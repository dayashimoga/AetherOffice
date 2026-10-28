//! OpenOfficeRS Excel Calculation Engine & Dependency DAG.
//! Features clean-room topological dependency graph evaluation, dirty-flag propagation,
//! Tarjan's cycle detection, iterative convergence (calcPr iterate, iterateCount, iterateDelta),
//! and exact Excel IEEE-754 subtraction snap-to-zero parity.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).

use crate::formula::ast::{AstNode, BinaryOperator, Literal, UnaryOperator};
use crate::formula::reference::Reference;
use crate::functions::{dispatch_function, EvalContext};
use crate::grid::{cell_to_coords, coords_to_cell, CellError, CellValue, CompressedSparsePageGrid};
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Cursor;

/// Excel IEEE-754 subtraction cancellation snap-to-zero relative threshold (~15-16 decimal digits).
/// If `|result| <= scale * IEEE_SNAP_THRESHOLD`, Excel snaps the subtraction/addition result to 0.0,
/// unless suppressed by top-level parenthesis or non-additive expressions.
pub const IEEE_SNAP_THRESHOLD: f64 = 2.842_170_943_040_401e-14;

/// Applies clean-room reverse-engineered Excel IEEE-754 snap-to-zero heuristic for addition/subtraction.
#[inline]
pub fn apply_ieee_snap_to_zero(result: f64, left: f64, right: f64) -> f64 {
    let scale = left.abs().max(right.abs());
    if scale > 0.0 && result.abs() <= scale * IEEE_SNAP_THRESHOLD {
        0.0
    } else {
        result
    }
}

/// Global cell identifier across sheets in a workbook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CellKey {
    /// 1-based sheet index (matches ECMA-376 calcChain `i` attribute)
    pub sheet_idx: u32,
    /// 0-based row index (0..1,048,575)
    pub row: u32,
    /// 0-based column index (0..16,383)
    pub col: u32,
}

impl CellKey {
    pub fn new(sheet_idx: u32, row: u32, col: u32) -> Self {
        Self {
            sheet_idx,
            row,
            col,
        }
    }

    /// Formats cell key as A1 notation (e.g. `A1`).
    pub fn to_a1(&self) -> String {
        coords_to_cell(self.row, self.col).unwrap_or_else(|_| "A1".to_string())
    }

    /// Parses cell key from sheet index and A1 notation string.
    pub fn parse_a1(sheet_idx: u32, s: &str) -> Result<Self> {
        let (row, col) = cell_to_coords(s)?;
        Ok(Self::new(sheet_idx, row, col))
    }
}

/// Configuration for circular reference iterative calculation (ECMA-376 calcPr).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IterativeConfig {
    /// Whether iterative calculation is enabled (`iterate="1"`)
    pub enabled: bool,
    /// Maximum number of iterations (default 100, `iterateCount="100"`)
    pub max_iterations: u32,
    /// Maximum change threshold for convergence (default 0.001, `iterateDelta="0.001"`)
    pub max_change: f64,
}

impl Default for IterativeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_iterations: 100,
            max_change: 0.001,
        }
    }
}

#[derive(Default)]
struct TarjanState {
    index: usize,
    stack: Vec<CellKey>,
    in_stack: HashSet<CellKey>,
    indices: HashMap<CellKey, usize>,
    lowlink: HashMap<CellKey, usize>,
    cycles: Vec<Vec<CellKey>>,
}

/// Topological Dependency Graph & Calculation Chain Engine for Excel workbooks.
#[derive(Debug, Clone, Default)]
pub struct CalculationDag {
    /// Forward dependencies: Predecessor -> Set of Dependents
    dependents: HashMap<CellKey, HashSet<CellKey>>,
    /// Reverse dependencies: Dependent -> Set of Predecessors
    predecessors: HashMap<CellKey, HashSet<CellKey>>,
    /// Set of cells marked dirty requiring recalculation
    dirty_cells: HashSet<CellKey>,
    /// Pre-sorted calculation sequence cache (calcChain.xml)
    calc_chain: Vec<CellKey>,
    /// Iterative calculation parameters
    pub iter_config: IterativeConfig,
}

impl CalculationDag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a directional dependency: `dependent` depends on `predecessor`.
    pub fn add_dependency(&mut self, dependent: CellKey, predecessor: CellKey) {
        self.dependents
            .entry(predecessor)
            .or_default()
            .insert(dependent);
        self.predecessors
            .entry(dependent)
            .or_default()
            .insert(predecessor);
    }

    /// Registers a formula cell and extracts all cell and range dependencies from its AST.
    pub fn register_formula(&mut self, cell: CellKey, ast: &AstNode) {
        self.unregister_cell(cell);
        let mut refs = Vec::new();
        extract_references_from_ast(ast, &mut refs);

        for r in refs {
            match r {
                Reference::Cell {
                    sheet: _,
                    cell: addr,
                } => {
                    let pred = CellKey::new(cell.sheet_idx, addr.row, addr.col);
                    self.add_dependency(cell, pred);
                }
                Reference::Range { sheet: _, range } => {
                    let r_start = range.start.row.min(range.end.row);
                    let r_end = range.start.row.max(range.end.row);
                    let c_start = range.start.col.min(range.end.col);
                    let c_end = range.start.col.max(range.end.col);

                    // Add dependencies for cells within bounded range
                    let max_r = r_end.min(r_start + 1024);
                    let max_c = c_end.min(c_start + 256);
                    for r in r_start..=max_r {
                        for c in c_start..=max_c {
                            let pred = CellKey::new(cell.sheet_idx, r, c);
                            self.add_dependency(cell, pred);
                        }
                    }
                }
                Reference::Spill(inner) => {
                    if let Reference::Cell { cell: addr, .. } = &**inner {
                        let pred = CellKey::new(cell.sheet_idx, addr.row, addr.col);
                        self.add_dependency(cell, pred);
                    }
                }
                _ => {}
            }
        }

        self.mark_dirty(cell);
    }

    /// Unregisters all existing predecessor dependencies for a cell.
    pub fn unregister_cell(&mut self, cell: CellKey) {
        self.dirty_cells.remove(&cell);
        if let Some(preds) = self.predecessors.remove(&cell) {
            for pred in preds {
                if let Some(deps) = self.dependents.get_mut(&pred) {
                    deps.remove(&cell);
                }
            }
        }
    }

    /// Marks a cell dirty and propagates dirty flags to all transitive dependents.
    pub fn mark_dirty(&mut self, cell: CellKey) {
        let mut queue = VecDeque::new();
        queue.push_back(cell);
        self.dirty_cells.insert(cell);

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = self.dependents.get(&current) {
                for &dep in deps {
                    if self.dirty_cells.insert(dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    /// Marks all registered formula cells dirty.
    pub fn mark_all_dirty(&mut self) {
        for &dep in self.predecessors.keys() {
            self.dirty_cells.insert(dep);
        }
    }

    /// Clears all dirty flags.
    pub fn clear_dirty_cells(&mut self) {
        self.dirty_cells.clear();
    }

    /// Checks if a cell is currently marked dirty.
    pub fn is_dirty(&self, cell: CellKey) -> bool {
        self.dirty_cells.contains(&cell)
    }

    /// Detects circular references using Tarjan's Strongly Connected Components (SCC) algorithm.
    /// Returns a list of strongly connected components containing 2 or more cells, or self-cycles.
    pub fn detect_cycles(&self) -> Vec<Vec<CellKey>> {
        let mut state = TarjanState::default();

        for &node in self.predecessors.keys() {
            if !state.indices.contains_key(&node) {
                self.strongconnect(node, &mut state);
            }
        }

        state.cycles
    }

    fn strongconnect(&self, u: CellKey, state: &mut TarjanState) {
        state.indices.insert(u, state.index);
        state.lowlink.insert(u, state.index);
        state.index += 1;
        state.stack.push(u);
        state.in_stack.insert(u);

        if let Some(deps) = self.dependents.get(&u) {
            for &v in deps {
                if !state.indices.contains_key(&v) {
                    self.strongconnect(v, state);
                    let v_low = state.lowlink.get(&v).copied().unwrap_or(0);
                    let u_low = state.lowlink.get(&u).copied().unwrap_or(0);
                    state.lowlink.insert(u, u_low.min(v_low));
                } else if state.in_stack.contains(&v) {
                    let v_idx = state.indices.get(&v).copied().unwrap_or(0);
                    let u_low = state.lowlink.get(&u).copied().unwrap_or(0);
                    state.lowlink.insert(u, u_low.min(v_idx));
                }
            }
        }

        if state.lowlink.get(&u) == state.indices.get(&u) {
            let mut scc = Vec::new();
            while let Some(w) = state.stack.pop() {
                state.in_stack.remove(&w);
                scc.push(w);
                if w == u {
                    break;
                }
            }

            // A component is a cycle if it contains more than 1 cell or a self-loop
            if scc.len() > 1
                || (scc.len() == 1
                    && self
                        .dependents
                        .get(&u)
                        .map(|d| d.contains(&u))
                        .unwrap_or(false))
            {
                state.cycles.push(scc);
            }
        }
    }

    /// Computes topological evaluation order using Kahn's algorithm for all dirty cells.
    /// If cycles exist and iteration is disabled, circular cells are omitted from the sequence
    /// and marked with error.
    pub fn topological_sort(&self) -> Result<Vec<CellKey>> {
        // Collect in-degrees for all dirty cells
        let mut in_degree: HashMap<CellKey, usize> = HashMap::new();
        for &cell in &self.dirty_cells {
            let preds_count = self
                .predecessors
                .get(&cell)
                .map(|preds| {
                    preds
                        .iter()
                        .filter(|p| self.dirty_cells.contains(p))
                        .count()
                })
                .unwrap_or(0);
            in_degree.insert(cell, preds_count);
        }

        let mut queue = VecDeque::new();
        for (&cell, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(cell);
            }
        }

        let mut sorted = Vec::new();
        while let Some(u) = queue.pop_front() {
            sorted.push(u);
            if let Some(deps) = self.dependents.get(&u) {
                for &v in deps {
                    if self.dirty_cells.contains(&v) {
                        if let Some(deg) = in_degree.get_mut(&v) {
                            if *deg > 0 {
                                *deg -= 1;
                                if *deg == 0 {
                                    queue.push_back(v);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If cycle exists, sorted.len() < dirty_cells.len()
        if sorted.len() < self.dirty_cells.len() && !self.iter_config.enabled {
            let cycles = self.detect_cycles();
            if !cycles.is_empty() {
                let first_cycle_cells: Vec<String> = cycles[0].iter().map(|c| c.to_a1()).collect();
                return Err(EngineError::FormulaError(format!(
                    "Circular reference detected in formula chain: {}",
                    first_cycle_cells.join(" -> ")
                )));
            }
        }

        Ok(sorted)
    }

    /// Groups dirty cells into topological dependency levels (batches of mutually independent cells)
    /// suitable for parallel evaluation across DAG levels.
    pub fn topological_levels(&self) -> Result<Vec<Vec<CellKey>>> {
        let mut in_degree: HashMap<CellKey, usize> = HashMap::new();
        for &cell in &self.dirty_cells {
            let preds_count = self
                .predecessors
                .get(&cell)
                .map(|preds| {
                    preds
                        .iter()
                        .filter(|p| self.dirty_cells.contains(p))
                        .count()
                })
                .unwrap_or(0);
            in_degree.insert(cell, preds_count);
        }

        let mut current_level = Vec::new();
        for (&cell, &degree) in &in_degree {
            if degree == 0 {
                current_level.push(cell);
            }
        }

        let mut levels = Vec::new();
        let mut total_count = 0;

        while !current_level.is_empty() {
            current_level.sort();
            total_count += current_level.len();
            let mut next_level = Vec::new();

            for &u in &current_level {
                if let Some(deps) = self.dependents.get(&u) {
                    for &v in deps {
                        if self.dirty_cells.contains(&v) {
                            if let Some(deg) = in_degree.get_mut(&v) {
                                if *deg > 0 {
                                    *deg -= 1;
                                    if *deg == 0 {
                                        next_level.push(v);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            levels.push(current_level);
            current_level = next_level;
        }

        if total_count < self.dirty_cells.len() && !self.iter_config.enabled {
            let cycles = self.detect_cycles();
            if !cycles.is_empty() {
                let first_cycle_cells: Vec<String> = cycles[0].iter().map(|c| c.to_a1()).collect();
                return Err(EngineError::FormulaError(format!(
                    "Circular reference detected in formula chain: {}",
                    first_cycle_cells.join(" -> ")
                )));
            }
        }

        Ok(levels)
    }

    /// Evaluates all dirty cells in parallel topological levels against a grid, applying exact IEEE-754 snap.
    pub fn evaluate_dirty(&mut self, grid: &mut CompressedSparsePageGrid) -> Result<usize> {
        let cycles = self.detect_cycles();
        if !cycles.is_empty() && self.iter_config.enabled {
            return self.evaluate_with_iterations(grid, &cycles);
        }

        let levels = self.topological_levels()?;
        let mut count = 0;

        for level in levels {
            // Parallel evaluation of mutually independent cells at this DAG level via Rayon
            let eval_results: Result<Vec<(CellKey, Option<CellValue>)>> = level
                .par_iter()
                .map(|&cell| {
                    if let Some(formula_str) = grid.get_formula_str(cell.row, cell.col) {
                        let ast = crate::formula::parse_formula(formula_str)?;
                        let ctx = EvalContext::new(Some(grid), false);
                        let evaluated = evaluate_ast_node_with_ctx(&ast, &ctx, true)?;
                        Ok((cell, Some(evaluated)))
                    } else {
                        Ok((cell, None))
                    }
                })
                .collect();

            let eval_results = eval_results?;
            for (cell, maybe_evaluated) in eval_results {
                if let Some(evaluated) = maybe_evaluated {
                    count += 1;
                    match evaluated {
                        CellValue::Number(num) => {
                            grid.set_formula_cached_value(cell.row, cell.col, num)?;
                        }
                        CellValue::Boolean(b) => {
                            grid.set_boolean(cell.row, cell.col, b)?;
                        }
                        CellValue::Error(e) => {
                            grid.set_error(cell.row, cell.col, e)?;
                        }
                        CellValue::InlineString(s) => {
                            grid.set_inline_string(cell.row, cell.col, &s)?;
                        }
                        CellValue::Array(matrix) => {
                            if let Some(first_cell) = matrix.first().and_then(|r| r.first()) {
                                match first_cell {
                                    CellValue::Number(num) => {
                                        grid.set_formula_cached_value(cell.row, cell.col, *num)?;
                                    }
                                    CellValue::Boolean(b) => {
                                        grid.set_boolean(cell.row, cell.col, *b)?;
                                    }
                                    CellValue::Error(e) => {
                                        grid.set_error(cell.row, cell.col, *e)?;
                                    }
                                    CellValue::InlineString(s) => {
                                        grid.set_inline_string(cell.row, cell.col, s)?;
                                    }
                                    _ => {}
                                }
                            }
                            // Dynamic array spill into adjacent grid cells
                            for (r_off, row) in matrix.iter().enumerate() {
                                for (c_off, val) in row.iter().enumerate() {
                                    if r_off == 0 && c_off == 0 {
                                        continue;
                                    }
                                    let target_r = cell.row + r_off as u32;
                                    let target_c = cell.col + c_off as u32;
                                    match val {
                                        CellValue::Number(n) => {
                                            grid.set_number(target_r, target_c, *n)?;
                                        }
                                        CellValue::Boolean(b) => {
                                            grid.set_boolean(target_r, target_c, *b)?;
                                        }
                                        CellValue::Error(e) => {
                                            grid.set_error(target_r, target_c, *e)?;
                                        }
                                        CellValue::InlineString(s) => {
                                            grid.set_inline_string(target_r, target_c, s)?;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                self.dirty_cells.remove(&cell);
            }
        }

        Ok(count)
    }

    /// Evaluates circular formula dependencies iteratively until convergence or max iterations.
    fn evaluate_with_iterations(
        &mut self,
        grid: &mut CompressedSparsePageGrid,
        cycles: &[Vec<CellKey>],
    ) -> Result<usize> {
        let mut cyclic_set = HashSet::new();
        for cycle in cycles {
            for &c in cycle {
                cyclic_set.insert(c);
            }
        }

        let non_cyclic_order: Vec<CellKey> = self
            .dirty_cells
            .iter()
            .copied()
            .filter(|c| !cyclic_set.contains(c))
            .collect();

        // 1. Evaluate independent non-cyclic cells first
        for cell in non_cyclic_order {
            if let Some(formula_str) = grid.get_formula_str(cell.row, cell.col) {
                let formula_owned = formula_str.to_string();
                let ast = crate::formula::parse_formula(&formula_owned)?;
                let evaluated = evaluate_ast_node(&ast, grid, true)?;
                if let CellValue::Number(num) = evaluated {
                    grid.set_formula_cached_value(cell.row, cell.col, num)?;
                }
            }
            self.dirty_cells.remove(&cell);
        }

        // 2. Perform Gauss-Seidel iterations on cyclic set
        let mut iter = 0;
        let mut max_delta = f64::MAX;

        while iter < self.iter_config.max_iterations && max_delta > self.iter_config.max_change {
            max_delta = 0.0;
            iter += 1;

            for &cell in &cyclic_set {
                let old_val = match grid.get_value(cell.row, cell.col) {
                    CellValue::Number(n) => n,
                    CellValue::Formula { cached_value, .. } => cached_value.unwrap_or(0.0),
                    _ => 0.0,
                };

                if let Some(formula_str) = grid.get_formula_str(cell.row, cell.col) {
                    let formula_owned = formula_str.to_string();
                    let ast = crate::formula::parse_formula(&formula_owned)?;
                    let evaluated = evaluate_ast_node(&ast, grid, true)?;
                    if let CellValue::Number(new_val) = evaluated {
                        let delta = (new_val - old_val).abs();
                        if delta > max_delta {
                            max_delta = delta;
                        }
                        grid.set_formula_cached_value(cell.row, cell.col, new_val)?;
                    }
                }
            }
        }

        for &c in &cyclic_set {
            self.dirty_cells.remove(&c);
        }

        Ok(cyclic_set.len())
    }

    /// Parses an ECMA-376 `xl/calcChain.xml` part into this calculation chain.
    pub fn parse_calc_chain_xml(&mut self, xml: &[u8]) -> Result<()> {
        let mut reader = Reader::from_reader(Cursor::new(xml));
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        self.calc_chain.clear();

        let mut current_sheet_id: u32 = 1;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    if e.name().as_ref() == b"c" {
                        let mut cell_ref = String::new();
                        for attr in e.attributes() {
                            let attr = attr.map_err(EngineError::XmlAttr)?;
                            match attr.key.as_ref() {
                                b"r" => {
                                    cell_ref = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"i" => {
                                    let s = String::from_utf8_lossy(&attr.value);
                                    if let Ok(id) = s.parse::<u32>() {
                                        current_sheet_id = id;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if !cell_ref.is_empty() {
                            let key = CellKey::parse_a1(current_sheet_id, &cell_ref)?;
                            self.calc_chain.push(key);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(EngineError::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    /// Serializes current calculation sequence into ECMA-376 `xl/calcChain.xml`.
    pub fn serialize_calc_chain_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                Some("yes"),
            )))
            .map_err(EngineError::Io)?;

        let mut root = BytesStart::new("calcChain");
        root.push_attribute((
            "xmlns",
            "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        ));
        writer
            .write_event(Event::Start(root))
            .map_err(EngineError::Io)?;

        let mut last_sheet_id: Option<u32> = None;
        for cell in &self.calc_chain {
            let mut elem = BytesStart::new("c");
            let cell_ref = cell.to_a1();
            elem.push_attribute(("r", cell_ref.as_str()));

            if last_sheet_id != Some(cell.sheet_idx) {
                let sheet_str = cell.sheet_idx.to_string();
                elem.push_attribute(("i", sheet_str.as_str()));
                last_sheet_id = Some(cell.sheet_idx);
            }

            writer
                .write_event(Event::Empty(elem))
                .map_err(EngineError::Io)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("calcChain")))
            .map_err(EngineError::Io)?;

        Ok(writer.into_inner().into_inner())
    }
}

fn extract_references_from_ast<'a>(node: &'a AstNode, refs: &mut Vec<&'a Reference>) {
    match node {
        AstNode::Reference(r) => refs.push(r),
        AstNode::UnaryOp { expr, .. } => extract_references_from_ast(expr, refs),
        AstNode::BinaryOp { left, right, .. } => {
            extract_references_from_ast(left, refs);
            extract_references_from_ast(right, refs);
        }
        AstNode::FunctionCall { args, .. } => {
            for arg in args {
                extract_references_from_ast(arg, refs);
            }
        }
        AstNode::ArrayLiteral { rows } => {
            for row in rows {
                for cell in row {
                    extract_references_from_ast(cell, refs);
                }
            }
        }
        AstNode::Parenthesized(inner) => extract_references_from_ast(inner, refs),
        AstNode::Literal(_) => {}
    }
}

/// Evaluates an AST node with exact clean-room IEEE-754 subtraction snap heuristic.
pub fn evaluate_ast_node(
    node: &AstNode,
    grid: &CompressedSparsePageGrid,
    is_root: bool,
) -> Result<CellValue> {
    let ctx = EvalContext::new(Some(grid), false);
    evaluate_ast_node_with_ctx(node, &ctx, is_root)
}

/// Evaluates an AST node with evaluation context (scope bindings and grid).
pub fn evaluate_ast_node_with_ctx(
    node: &AstNode,
    ctx: &EvalContext,
    is_root: bool,
) -> Result<CellValue> {
    match node {
        AstNode::Literal(lit) => match lit {
            Literal::Number(n) => Ok(CellValue::Number(*n)),
            Literal::String(s) => Ok(CellValue::InlineString(s.clone())),
            Literal::Boolean(b) => Ok(CellValue::Boolean(*b)),
            Literal::Error(e) => Ok(CellValue::Error(*e)),
            Literal::Empty => Ok(CellValue::Number(0.0)),
        },
        AstNode::Reference(r) => resolve_reference_value_with_ctx(r, ctx),
        AstNode::Parenthesized(inner) => {
            // Parenthesized expression suppresses snap-to-zero heuristic at root level
            evaluate_ast_node_with_ctx(inner, ctx, false)
        }
        AstNode::UnaryOp { op, expr } => {
            let val = evaluate_ast_node_with_ctx(expr, ctx, false)?;
            if let CellValue::Error(_) = val {
                return Ok(val);
            }
            match op {
                UnaryOperator::Plus => Ok(val),
                UnaryOperator::Neg => match val {
                    CellValue::Number(n) => Ok(CellValue::Number(-n)),
                    other => Ok(other),
                },
                UnaryOperator::Percent => match val {
                    CellValue::Number(n) => Ok(CellValue::Number(n / 100.0)),
                    other => Ok(other),
                },
                UnaryOperator::ImplicitIntersection | UnaryOperator::Spill => Ok(val),
            }
        }
        AstNode::BinaryOp { op, left, right } => {
            let l_val = evaluate_ast_node_with_ctx(left, ctx, false)?;
            if let CellValue::Error(_) = l_val {
                return Ok(l_val);
            }
            let r_val = evaluate_ast_node_with_ctx(right, ctx, false)?;
            if let CellValue::Error(_) = r_val {
                return Ok(r_val);
            }

            match op {
                BinaryOperator::Add => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    let mut res = l_num + r_num;
                    if is_root {
                        res = apply_ieee_snap_to_zero(res, l_num, r_num);
                    }
                    Ok(CellValue::Number(res))
                }
                BinaryOperator::Sub => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    let mut res = l_num - r_num;
                    if is_root {
                        res = apply_ieee_snap_to_zero(res, l_num, r_num);
                    }
                    Ok(CellValue::Number(res))
                }
                BinaryOperator::Mul => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Number(l_num * r_num))
                }
                BinaryOperator::Div => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    if r_num == 0.0 {
                        Ok(CellValue::Error(CellError::Div0))
                    } else {
                        Ok(CellValue::Number(l_num / r_num))
                    }
                }
                BinaryOperator::Pow => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Number(l_num.powf(r_num)))
                }
                BinaryOperator::Concat => {
                    let l_str = extract_string(&l_val);
                    let r_str = extract_string(&r_val);
                    Ok(CellValue::InlineString(format!("{l_str}{r_str}")))
                }
                BinaryOperator::Equal => {
                    let eq = match (&l_val, &r_val) {
                        (CellValue::Number(a), CellValue::Number(b)) => {
                            (a - b).abs() < f64::EPSILON
                        }
                        _ => l_val == r_val,
                    };
                    Ok(CellValue::Boolean(eq))
                }
                BinaryOperator::NotEqual => {
                    let ne = match (&l_val, &r_val) {
                        (CellValue::Number(a), CellValue::Number(b)) => {
                            (a - b).abs() >= f64::EPSILON
                        }
                        _ => l_val != r_val,
                    };
                    Ok(CellValue::Boolean(ne))
                }
                BinaryOperator::LessThan => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Boolean(l_num < r_num))
                }
                BinaryOperator::LessEqual => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Boolean(l_num <= r_num))
                }
                BinaryOperator::GreaterThan => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Boolean(l_num > r_num))
                }
                BinaryOperator::GreaterEqual => {
                    let l_num = extract_f64(&l_val)?;
                    let r_num = extract_f64(&r_val)?;
                    Ok(CellValue::Boolean(l_num >= r_num))
                }
                _ => Ok(l_val),
            }
        }
        AstNode::FunctionCall { name, args } => {
            dispatch_function(name, args, ctx, &|sub_node, sub_ctx| {
                evaluate_ast_node_with_ctx(sub_node, sub_ctx, false)
            })
        }
        AstNode::ArrayLiteral { rows } => {
            let mut matrix = Vec::new();
            for r in rows {
                let mut row = Vec::new();
                for cell in r {
                    row.push(evaluate_ast_node_with_ctx(cell, ctx, false)?);
                }
                matrix.push(row);
            }
            Ok(CellValue::Array(matrix))
        }
    }
}

fn resolve_reference_value_with_ctx(r: &Reference, ctx: &EvalContext) -> Result<CellValue> {
    match r {
        Reference::Cell { cell, .. } => {
            if let Some(grid) = ctx.grid {
                Ok(grid.get_value(cell.row, cell.col))
            } else {
                Ok(CellValue::Error(CellError::Ref))
            }
        }
        Reference::Spill(inner) => resolve_reference_value_with_ctx(inner, ctx),
        Reference::ImplicitIntersection(inner) => resolve_reference_value_with_ctx(inner, ctx),
        Reference::Range { range, .. } => {
            if let Some(grid) = ctx.grid {
                let r1 = range.start.row.min(range.end.row);
                let r2 = range.start.row.max(range.end.row);
                let c1 = range.start.col.min(range.end.col);
                let c2 = range.start.col.max(range.end.col);
                let mut matrix = Vec::new();
                for row_idx in r1..=r2 {
                    let mut row = Vec::new();
                    for col_idx in c1..=c2 {
                        row.push(grid.get_value(row_idx, col_idx));
                    }
                    matrix.push(row);
                }
                Ok(CellValue::Array(matrix))
            } else {
                Ok(CellValue::Error(CellError::Ref))
            }
        }
        Reference::Named(name) => {
            if let Some(val) = ctx.env.get(&name.to_ascii_uppercase()) {
                Ok(val.clone())
            } else {
                Ok(CellValue::Number(0.0))
            }
        }
        Reference::Table(_) => Ok(CellValue::Number(0.0)),
    }
}

fn extract_f64(val: &CellValue) -> Result<f64> {
    match val {
        CellValue::Number(n) => Ok(*n),
        CellValue::Formula { cached_value, .. } => Ok(cached_value.unwrap_or(0.0)),
        CellValue::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CellValue::InlineString(s) => s.trim().parse::<f64>().map_err(|_| {
            EngineError::FormulaError(format!("Cannot convert string '{s}' to number"))
        }),
        CellValue::Empty => Ok(0.0),
        CellValue::Error(e) => Err(EngineError::FormulaError(format!("Cell error: {e}"))),
        CellValue::Array(arr) => {
            if let Some(first) = arr.first().and_then(|r| r.first()) {
                extract_f64(first)
            } else {
                Ok(0.0)
            }
        }
        _ => Ok(0.0),
    }
}

fn extract_string(val: &CellValue) -> String {
    match val {
        CellValue::InlineString(s) => s.clone(),
        CellValue::Number(n) => n.to_string(),
        CellValue::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.into(),
        CellValue::Error(e) => e.to_string(),
        CellValue::Empty => String::new(),
        CellValue::Array(arr) => {
            if let Some(first) = arr.first().and_then(|r| r.first()) {
                extract_string(first)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}
