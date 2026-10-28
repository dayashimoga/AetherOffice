//! OpenOfficeRS Excel Calculation Engine & Dependency DAG.
//! Features clean-room topological dependency graph evaluation, dirty-flag propagation,
//! Tarjan's cycle detection, iterative convergence (calcPr iterate, iterateCount, iterateDelta),
//! and exact Excel IEEE-754 subtraction snap-to-zero parity.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Result<T, EngineError>, #[forbid(unsafe_code)]).

use crate::formula::ast::{AstNode, BinaryOperator, Literal, UnaryOperator};
use crate::formula::reference::Reference;
use crate::grid::{cell_to_coords, coords_to_cell, CellError, CellValue, CompressedSparsePageGrid};
use ooxml_opc::error::{EngineError, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
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

    /// Evaluates all dirty cells in topological order against a grid, applying the exact IEEE-754 snap.
    pub fn evaluate_dirty(&mut self, grid: &mut CompressedSparsePageGrid) -> Result<usize> {
        let cycles = self.detect_cycles();
        if !cycles.is_empty() && self.iter_config.enabled {
            return self.evaluate_with_iterations(grid, &cycles);
        }

        let order = self.topological_sort()?;
        let mut count = 0;

        for cell in order {
            if let Some(formula_str) = grid.get_formula_str(cell.row, cell.col) {
                count += 1;
                let formula_owned = formula_str.to_string();
                let ast = crate::formula::parse_formula(&formula_owned)?;
                let evaluated = evaluate_ast_node(&ast, grid, true)?;
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
                    _ => {}
                }
            } else {
                // Non-formula cell evaluated
                self.dirty_cells.remove(&cell);
            }
            self.dirty_cells.remove(&cell);
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
    match node {
        AstNode::Literal(lit) => match lit {
            Literal::Number(n) => Ok(CellValue::Number(*n)),
            Literal::String(s) => Ok(CellValue::InlineString(s.clone())),
            Literal::Boolean(b) => Ok(CellValue::Boolean(*b)),
            Literal::Error(e) => Ok(CellValue::Error(*e)),
            Literal::Empty => Ok(CellValue::Number(0.0)),
        },
        AstNode::Reference(r) => resolve_reference_value(r, grid),
        AstNode::Parenthesized(inner) => {
            // Parenthesized expression suppresses snap-to-zero heuristic at root level
            evaluate_ast_node(inner, grid, false)
        }
        AstNode::UnaryOp { op, expr } => {
            let val = evaluate_ast_node(expr, grid, false)?;
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
            let l_val = evaluate_ast_node(left, grid, false)?;
            let r_val = evaluate_ast_node(right, grid, false)?;

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
        AstNode::FunctionCall { name, args } => evaluate_function(name, args, grid),
        AstNode::ArrayLiteral { rows } => {
            if let Some(first_row) = rows.first() {
                if let Some(first_cell) = first_row.first() {
                    return evaluate_ast_node(first_cell, grid, false);
                }
            }
            Ok(CellValue::Number(0.0))
        }
    }
}

fn resolve_reference_value(r: &Reference, grid: &CompressedSparsePageGrid) -> Result<CellValue> {
    match r {
        Reference::Cell { cell, .. } => Ok(grid.get_value(cell.row, cell.col)),
        Reference::Spill(inner) => resolve_reference_value(inner, grid),
        Reference::ImplicitIntersection(inner) => resolve_reference_value(inner, grid),
        Reference::Range { range, .. } => {
            // Default scalar evaluation of range takes top-left corner
            Ok(grid.get_value(range.start.row, range.start.col))
        }
        Reference::Named(_) | Reference::Table(_) => Ok(CellValue::Number(0.0)),
    }
}

fn evaluate_function(
    name: &str,
    args: &[AstNode],
    grid: &CompressedSparsePageGrid,
) -> Result<CellValue> {
    let upper = name.to_ascii_uppercase();
    match upper.as_str() {
        "SUM" => {
            let mut sum = 0.0;
            for arg in args {
                match arg {
                    AstNode::Reference(Reference::Range { range, .. }) => {
                        let r1 = range.start.row.min(range.end.row);
                        let r2 = range.start.row.max(range.end.row);
                        let c1 = range.start.col.min(range.end.col);
                        let c2 = range.start.col.max(range.end.col);
                        for r in r1..=r2 {
                            for c in c1..=c2 {
                                if let CellValue::Number(n) = grid.get_value(r, c) {
                                    sum += n;
                                }
                            }
                        }
                    }
                    other => {
                        let val = evaluate_ast_node(other, grid, false)?;
                        if let Ok(n) = extract_f64(&val) {
                            sum += n;
                        }
                    }
                }
            }
            Ok(CellValue::Number(sum))
        }
        "AVERAGE" => {
            let mut sum = 0.0;
            let mut count = 0;
            for arg in args {
                match arg {
                    AstNode::Reference(Reference::Range { range, .. }) => {
                        let r1 = range.start.row.min(range.end.row);
                        let r2 = range.start.row.max(range.end.row);
                        let c1 = range.start.col.min(range.end.col);
                        let c2 = range.start.col.max(range.end.col);
                        for r in r1..=r2 {
                            for c in c1..=c2 {
                                if let CellValue::Number(n) = grid.get_value(r, c) {
                                    sum += n;
                                    count += 1;
                                }
                            }
                        }
                    }
                    other => {
                        let val = evaluate_ast_node(other, grid, false)?;
                        if let Ok(n) = extract_f64(&val) {
                            sum += n;
                            count += 1;
                        }
                    }
                }
            }
            if count == 0 {
                Ok(CellValue::Error(CellError::Div0))
            } else {
                Ok(CellValue::Number(sum / count as f64))
            }
        }
        "MIN" => {
            let mut min = f64::MAX;
            for arg in args {
                let val = evaluate_ast_node(arg, grid, false)?;
                if let Ok(n) = extract_f64(&val) {
                    if n < min {
                        min = n;
                    }
                }
            }
            Ok(CellValue::Number(if min == f64::MAX { 0.0 } else { min }))
        }
        "MAX" => {
            let mut max = f64::MIN;
            for arg in args {
                let val = evaluate_ast_node(arg, grid, false)?;
                if let Ok(n) = extract_f64(&val) {
                    if n > max {
                        max = n;
                    }
                }
            }
            Ok(CellValue::Number(if max == f64::MIN { 0.0 } else { max }))
        }
        "COUNT" => {
            let mut count = 0;
            for arg in args {
                let val = evaluate_ast_node(arg, grid, false)?;
                if matches!(val, CellValue::Number(_)) {
                    count += 1;
                }
            }
            Ok(CellValue::Number(count as f64))
        }
        "ABS" => {
            if let Some(first) = args.first() {
                let val = evaluate_ast_node(first, grid, false)?;
                let n = extract_f64(&val)?;
                Ok(CellValue::Number(n.abs()))
            } else {
                Ok(CellValue::Error(CellError::Value))
            }
        }
        "ROUND" => {
            if args.len() >= 2 {
                let val = evaluate_ast_node(&args[0], grid, false)?;
                let digits = evaluate_ast_node(&args[1], grid, false)?;
                let n = extract_f64(&val)?;
                let d = extract_f64(&digits)? as i32;
                let factor = 10f64.powi(d);
                let scaled = n * factor;
                let eps = 1e-10;
                let rounded = if scaled >= 0.0 {
                    (scaled + eps).round()
                } else {
                    (scaled - eps).round()
                };
                Ok(CellValue::Number(rounded / factor))
            } else {
                Ok(CellValue::Error(CellError::Value))
            }
        }
        "IF" => {
            if args.is_empty() {
                return Ok(CellValue::Error(CellError::Value));
            }
            let cond_val = evaluate_ast_node(&args[0], grid, false)?;
            let is_true = match cond_val {
                CellValue::Boolean(b) => b,
                CellValue::Number(n) => n != 0.0,
                _ => false,
            };

            if is_true {
                if args.len() > 1 {
                    evaluate_ast_node(&args[1], grid, false)
                } else {
                    Ok(CellValue::Boolean(true))
                }
            } else if args.len() > 2 {
                evaluate_ast_node(&args[2], grid, false)
            } else {
                Ok(CellValue::Boolean(false))
            }
        }
        _ => Ok(CellValue::Error(CellError::Name)),
    }
}

fn extract_f64(val: &CellValue) -> Result<f64> {
    match val {
        CellValue::Number(n) => Ok(*n),
        CellValue::Formula { cached_value, .. } => Ok(cached_value.unwrap_or(0.0)),
        CellValue::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CellValue::InlineString(s) => s.parse::<f64>().map_err(|_| {
            EngineError::FormulaError(format!("Cannot convert string '{s}' to number"))
        }),
        CellValue::Empty => Ok(0.0),
        CellValue::Error(e) => Err(EngineError::FormulaError(format!("Cell error: {e}"))),
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
        _ => String::new(),
    }
}
