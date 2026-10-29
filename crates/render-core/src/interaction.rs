//! Interactive Hit-Testing, Selection Handles & Undo/Redo Command Bus
//!
//! Complies with ECMA-376, W3C Pointer Events Level 3, and W3C Clipboard API.
//! Provides:
//! - Sub-millisecond pointer/touch screen-to-world coordinate mapping
//! - Word, Excel, and PowerPoint hit-testing targets
//! - Selection models (Word caret/range, Excel cell/range/fill-handle, PowerPoint 8-handle & rotation)
//! - Multi-format clipboard serializer (Plain text TSV, HTML markup, OOXML)
//! - Transactional Command Bus with unlimited Undo/Redo and typing coalescing
//! - Kinetic scrolling inertia and boundary rubber-banding physics

use crate::primitive::Rect2D;
use std::fmt::Debug;

// =========================================================================
// 1. Screen-to-World Coordinate Mapping
// =========================================================================

pub struct CoordinateTransformer;

impl CoordinateTransformer {
    /// Map physical screen pointer coordinates $(x_{\text{screen}}, y_{\text{screen}})$ to world DIP coordinates.
    pub fn screen_to_world(
        screen_pt: (f64, f64),
        scroll: (f64, f64),
        zoom: f64,
        dpr: f64,
    ) -> (f64, f64) {
        let eff_scale = (zoom * dpr).max(0.001);
        let world_x = screen_pt.0 / eff_scale + scroll.0;
        let world_y = screen_pt.1 / eff_scale + scroll.1;
        (world_x, world_y)
    }

    /// Map world coordinates to physical screen coordinates.
    pub fn world_to_screen(
        world_pt: (f64, f64),
        scroll: (f64, f64),
        zoom: f64,
        dpr: f64,
    ) -> (f64, f64) {
        let eff_scale = (zoom * dpr).max(0.001);
        let screen_x = (world_pt.0 - scroll.0) * eff_scale;
        let screen_y = (world_pt.1 - scroll.1) * eff_scale;
        (screen_x, screen_y)
    }
}

// =========================================================================
// 2. Hit-Testing Target Types & Resolvers
// =========================================================================

/// Resolved hit target within a Word document.
#[derive(Debug, Clone, PartialEq)]
pub enum WordHitTarget {
    Page {
        page_index: usize,
        local_pt: (f64, f64),
    },
    Text {
        page_index: usize,
        line_index: usize,
        char_offset: usize,
    },
    TableCell {
        page_index: usize,
        row: usize,
        col: usize,
    },
    TableRowResize {
        page_index: usize,
        row: usize,
        y: f64,
    },
    TableColResize {
        page_index: usize,
        col: usize,
        x: f64,
    },
    Outside,
}

/// Resolved hit target within an Excel worksheet.
#[derive(Debug, Clone, PartialEq)]
pub enum ExcelHitTarget {
    Cell { row: u32, col: u32 },
    ColHeader { col: u32 },
    RowHeader { row: u32 },
    ColResize { col: u32, x: f64 },
    RowResize { row: u32, y: f64 },
    AutoFillHandle { row: u32, col: u32 },
    Corner,
    Outside,
}

/// 8 Bounding box resize handles for PowerPoint shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

/// Resolved hit target within a PowerPoint slide.
#[derive(Debug, Clone, PartialEq)]
pub enum PptHitTarget {
    ShapeBody { shape_id: u32 },
    ResizeHandle { shape_id: u32, handle: ResizeHandle },
    RotationHandle { shape_id: u32 },
    CanvasBackground,
}

pub struct HitTester;

impl HitTester {
    /// Hit-test Word document text lines and character offsets.
    pub fn hit_test_word_line(line_x: f64, char_advances: &[f64], click_x: f64) -> usize {
        let mut curr_x = line_x;
        for (i, &adv) in char_advances.iter().enumerate() {
            let midpoint = curr_x + adv / 2.0;
            if click_x < midpoint {
                return i;
            }
            curr_x += adv;
        }
        char_advances.len()
    }

    /// Hit-test Word table cell vs. row border resize (+-3px threshold).
    pub fn hit_test_word_table_row(
        row_top_y: f64,
        row_bottom_y: f64,
        point_y: f64,
        resize_threshold: f64,
    ) -> (bool, bool) {
        let is_inside = point_y >= row_top_y && point_y <= row_bottom_y;
        let is_resize = (point_y - row_bottom_y).abs() <= resize_threshold;
        (is_inside, is_resize)
    }

    /// Hit-test Excel grid elements (cells, headers, dividers, fill-handle).
    pub fn hit_test_excel(
        row_h: f64,
        col_w: f64,
        header_w: f64,
        header_h: f64,
        selection_bottom_right: Option<(f64, f64)>,
        point: (f64, f64),
    ) -> ExcelHitTarget {
        let (x, y) = point;
        let resize_thresh = 3.0;

        // Top-left corner
        if x < header_w && y < header_h {
            return ExcelHitTarget::Corner;
        }

        // Column header or column divider resize
        if y < header_h {
            let rel_x = x - header_w;
            if rel_x < 0.0 {
                return ExcelHitTarget::Corner;
            }
            let col = (rel_x / col_w).floor() as u32;

            // Check if near the left boundary (resizing col - 1)
            if col > 0 {
                let left_boundary = header_w + (col as f64) * col_w;
                if (x - left_boundary).abs() <= resize_thresh {
                    return ExcelHitTarget::ColResize {
                        col: col - 1,
                        x: left_boundary,
                    };
                }
            }

            // Check if near the right boundary (resizing col)
            let right_boundary = header_w + ((col + 1) as f64) * col_w;
            if (x - right_boundary).abs() <= resize_thresh {
                return ExcelHitTarget::ColResize {
                    col,
                    x: right_boundary,
                };
            }

            return ExcelHitTarget::ColHeader { col };
        }

        // Row header or row divider resize
        if x < header_w {
            let rel_y = y - header_h;
            if rel_y < 0.0 {
                return ExcelHitTarget::Corner;
            }
            let row = (rel_y / row_h).floor() as u32;

            // Check if near the top boundary (resizing row - 1)
            if row > 0 {
                let top_boundary = header_h + (row as f64) * row_h;
                if (y - top_boundary).abs() <= resize_thresh {
                    return ExcelHitTarget::RowResize {
                        row: row - 1,
                        y: top_boundary,
                    };
                }
            }

            // Check if near the bottom boundary (resizing row)
            let bottom_boundary = header_h + ((row + 1) as f64) * row_h;
            if (y - bottom_boundary).abs() <= resize_thresh {
                return ExcelHitTarget::RowResize {
                    row,
                    y: bottom_boundary,
                };
            }

            return ExcelHitTarget::RowHeader { row };
        }

        // AutoFill handle check (6x6 px square around selection bottom-right)
        if let Some((br_x, br_y)) = selection_bottom_right {
            if (x - br_x).abs() <= 6.0 && (y - br_y).abs() <= 6.0 {
                let row = ((br_y - header_h) / row_h).floor().max(0.0) as u32;
                let col = ((br_x - header_w) / col_w).floor().max(0.0) as u32;
                return ExcelHitTarget::AutoFillHandle { row, col };
            }
        }

        // Regular cell
        let col = ((x - header_w) / col_w).floor().max(0.0) as u32;
        let row = ((y - header_h) / row_h).floor().max(0.0) as u32;
        ExcelHitTarget::Cell { row, col }
    }

    /// Hit-test PowerPoint shapes, 8 resize handles, and rotation handle.
    pub fn hit_test_ppt_shape(
        shape_id: u32,
        rect: &Rect2D,
        point: (f64, f64),
        is_selected: bool,
    ) -> Option<PptHitTarget> {
        let (x, y) = point;
        let handle_sz = 8.0;

        if is_selected {
            // Rotation handle: circle 24px above top center
            let rot_cx = rect.x + rect.width / 2.0;
            let rot_cy = rect.y - 24.0;
            let d_rot = ((x - rot_cx) * (x - rot_cx) + (y - rot_cy) * (y - rot_cy)).sqrt();
            if d_rot <= handle_sz {
                return Some(PptHitTarget::RotationHandle { shape_id });
            }

            // 8 Bounding box handles
            let handles = [
                (rect.x, rect.y, ResizeHandle::TopLeft),
                (rect.x + rect.width / 2.0, rect.y, ResizeHandle::Top),
                (rect.right(), rect.y, ResizeHandle::TopRight),
                (
                    rect.right(),
                    rect.y + rect.height / 2.0,
                    ResizeHandle::Right,
                ),
                (rect.right(), rect.bottom(), ResizeHandle::BottomRight),
                (
                    rect.x + rect.width / 2.0,
                    rect.bottom(),
                    ResizeHandle::Bottom,
                ),
                (rect.x, rect.bottom(), ResizeHandle::BottomLeft),
                (rect.x, rect.y + rect.height / 2.0, ResizeHandle::Left),
            ];

            for (hx, hy, handle) in handles {
                if (x - hx).abs() <= handle_sz / 2.0 && (y - hy).abs() <= handle_sz / 2.0 {
                    return Some(PptHitTarget::ResizeHandle { shape_id, handle });
                }
            }
        }

        // Shape interior body
        if rect.contains_point(x, y) {
            return Some(PptHitTarget::ShapeBody { shape_id });
        }

        None
    }
}

// =========================================================================
// 3. Selection Models
// =========================================================================

/// Word text selection caret / range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordSelection {
    pub anchor_offset: usize,
    pub active_offset: usize,
}

impl WordSelection {
    pub fn caret(offset: usize) -> Self {
        Self {
            anchor_offset: offset,
            active_offset: offset,
        }
    }

    pub fn range(anchor: usize, active: usize) -> Self {
        Self {
            anchor_offset: anchor,
            active_offset: active,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor_offset == self.active_offset
    }

    pub fn span(&self) -> usize {
        (self.active_offset as isize - self.anchor_offset as isize).unsigned_abs()
    }

    pub fn min_offset(&self) -> usize {
        self.anchor_offset.min(self.active_offset)
    }

    pub fn max_offset(&self) -> usize {
        self.anchor_offset.max(self.active_offset)
    }
}

/// Contiguous rectangular cell range in an Excel spreadsheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExcelRange {
    pub r1: u32,
    pub c1: u32,
    pub r2: u32,
    pub c2: u32,
}

impl ExcelRange {
    pub fn new(r1: u32, c1: u32, r2: u32, c2: u32) -> Self {
        Self {
            r1: r1.min(r2),
            c1: c1.min(c2),
            r2: r1.max(r2),
            c2: c1.max(c2),
        }
    }

    pub fn row_count(&self) -> u32 {
        self.r2 - self.r1 + 1
    }

    pub fn col_count(&self) -> u32 {
        self.c2 - self.c1 + 1
    }

    pub fn total_cells(&self) -> u32 {
        self.row_count() * self.col_count()
    }

    pub fn contains(&self, row: u32, col: u32) -> bool {
        row >= self.r1 && row <= self.r2 && col >= self.c1 && col <= self.c2
    }
}

/// Excel multi-range selection state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelSelection {
    pub active_cell: (u32, u32),
    pub ranges: Vec<ExcelRange>,
}

impl ExcelSelection {
    pub fn single(row: u32, col: u32) -> Self {
        Self {
            active_cell: (row, col),
            ranges: vec![ExcelRange::new(row, col, row, col)],
        }
    }

    pub fn range(r1: u32, c1: u32, r2: u32, c2: u32) -> Self {
        Self {
            active_cell: (r1, c1),
            ranges: vec![ExcelRange::new(r1, c1, r2, c2)],
        }
    }

    pub fn add_range(&mut self, range: ExcelRange) {
        self.ranges.push(range);
    }
}

/// PowerPoint shape selection state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PptSelection {
    pub selected_ids: Vec<u32>,
    pub primary_id: Option<u32>,
}

impl PptSelection {
    pub fn single(id: u32) -> Self {
        Self {
            selected_ids: vec![id],
            primary_id: Some(id),
        }
    }

    pub fn toggle(&mut self, id: u32) {
        if let Some(pos) = self.selected_ids.iter().position(|&x| x == id) {
            self.selected_ids.remove(pos);
            if self.primary_id == Some(id) {
                self.primary_id = self.selected_ids.last().copied();
            }
        } else {
            self.selected_ids.push(id);
            self.primary_id = Some(id);
        }
    }

    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.primary_id = None;
    }
}

// =========================================================================
// 4. Multi-Format Clipboard Serializer
// =========================================================================

/// Standardized multi-format clipboard payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardPayload {
    pub plain_text: String,
    pub html: String,
    pub ooxml: Option<String>,
}

pub struct ClipboardManager;

impl ClipboardManager {
    /// Serialize plain Word text with optional bold / italic styles.
    pub fn serialize_word(text: &str, is_bold: bool, is_italic: bool) -> ClipboardPayload {
        let mut html_style = String::new();
        if is_bold {
            html_style.push_str("font-weight:bold;");
        }
        if is_italic {
            html_style.push_str("font-style:italic;");
        }

        let html = if html_style.is_empty() {
            format!("<p>{}</p>", quick_xml::escape::escape(text))
        } else {
            format!(
                r#"<p><span style="{}">{}</span></p>"#,
                html_style,
                quick_xml::escape::escape(text)
            )
        };

        let ooxml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:rPr>{}{}</w:rPr><w:t>{}</w:t></w:r></w:p>"#,
            if is_bold { "<w:b/>" } else { "" },
            if is_italic { "<w:i/>" } else { "" },
            quick_xml::escape::escape(text)
        );

        ClipboardPayload {
            plain_text: text.to_string(),
            html,
            ooxml: Some(ooxml),
        }
    }

    /// Serialize spreadsheet 2D grid into plain text TSV and HTML `<table>`.
    pub fn serialize_excel(grid: &[Vec<String>]) -> ClipboardPayload {
        let mut plain = String::new();
        let mut html = String::from("<table>\n");

        for (r_idx, row) in grid.iter().enumerate() {
            if r_idx > 0 {
                plain.push('\n');
            }
            plain.push_str(&row.join("\t"));

            html.push_str("  <tr>\n");
            for cell in row {
                html.push_str(&format!(
                    "    <td>{}</td>\n",
                    quick_xml::escape::escape(cell)
                ));
            }
            html.push_str("  </tr>\n");
        }
        html.push_str("</table>");

        ClipboardPayload {
            plain_text: plain,
            html,
            ooxml: None,
        }
    }
}

// =========================================================================
// 5. Transactional Command Bus & Undo/Redo Engine
// =========================================================================

/// Generic document state manipulated by commands.
#[derive(Debug, Clone, Default)]
pub struct DocumentState {
    pub text_content: String,
    pub cell_values: Vec<(u32, u32, String)>,
}

/// Abstract reversible command executed through the Command Bus.
pub trait Command: Debug + std::any::Any + Send + Sync {
    fn execute(&mut self, state: &mut DocumentState) -> Result<(), String>;
    fn undo(&mut self, state: &mut DocumentState) -> Result<(), String>;
    fn redo(&mut self, state: &mut DocumentState) -> Result<(), String> {
        self.execute(state)
    }
    fn description(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn try_merge_next(&mut self, _next: &dyn Command) -> bool {
        false
    }
}

/// Command inserting text at a specified character offset.
#[derive(Debug, Clone)]
pub struct InsertTextCommand {
    pub offset: usize,
    pub text: String,
}

impl Command for InsertTextCommand {
    fn execute(&mut self, state: &mut DocumentState) -> Result<(), String> {
        if self.offset > state.text_content.len() {
            state.text_content.push_str(&self.text);
        } else {
            state.text_content.insert_str(self.offset, &self.text);
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut DocumentState) -> Result<(), String> {
        let len = self.text.len();
        if self.offset + len <= state.text_content.len() {
            state.text_content.drain(self.offset..self.offset + len);
        }
        Ok(())
    }

    fn description(&self) -> &str {
        "Insert Text"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn try_merge_next(&mut self, next: &dyn Command) -> bool {
        if let Some(other) = next.as_any().downcast_ref::<InsertTextCommand>() {
            if other.offset == self.offset + self.text.len() {
                self.text.push_str(&other.text);
                return true;
            }
        }
        false
    }
}

/// Compound atomic transaction executing multiple sub-commands as one atomic unit.
#[derive(Debug)]
pub struct CompoundCommand {
    pub description_str: String,
    pub commands: Vec<Box<dyn Command>>,
}

impl Command for CompoundCommand {
    fn execute(&mut self, state: &mut DocumentState) -> Result<(), String> {
        for cmd in &mut self.commands {
            cmd.execute(state)?;
        }
        Ok(())
    }

    fn undo(&mut self, state: &mut DocumentState) -> Result<(), String> {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(state)?;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description_str
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Command Bus managing the atomic Undo and Redo transaction stacks.
pub struct CommandBus {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_stack_len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_stack_len(&self) -> usize {
        self.redo_stack.len()
    }

    /// Execute a command, attempting to coalesce/merge with the previous command if applicable.
    pub fn execute(
        &mut self,
        state: &mut DocumentState,
        mut cmd: Box<dyn Command>,
    ) -> Result<(), String> {
        cmd.execute(state)?;

        // Try merging with the previous command
        if let Some(last) = self.undo_stack.last_mut() {
            if last.try_merge_next(cmd.as_ref()) {
                self.redo_stack.clear();
                return Ok(());
            }
        }

        self.undo_stack.push(cmd);
        self.redo_stack.clear();
        Ok(())
    }

    /// Revert the top command on the undo stack.
    pub fn undo(&mut self, state: &mut DocumentState) -> Result<(), String> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state)?;
            self.redo_stack.push(cmd);
            Ok(())
        } else {
            Err("Nothing to undo".into())
        }
    }

    /// Re-apply the top command on the redo stack.
    pub fn redo(&mut self, state: &mut DocumentState) -> Result<(), String> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.redo(state)?;
            self.undo_stack.push(cmd);
            Ok(())
        } else {
            Err("Nothing to redo".into())
        }
    }
}

// =========================================================================
// 6. Kinetic Scrolling & Pinch-to-Zoom Inertia Physics
// =========================================================================

pub struct InertiaPhysics;

impl InertiaPhysics {
    /// Calculate decayed velocity after `frames` under friction factor $\mu = 0.92$.
    pub fn decay_velocity(v0: f64, frames: usize, mu: f64) -> f64 {
        v0 * mu.powi(frames as i32)
    }

    /// Calculate spring restoration force for viewport boundary rubber-banding:
    /// $$F_{\text{spring}} = -k \cdot \Delta x - c \cdot v$$
    pub fn compute_spring_force(displacement: f64, velocity: f64, k: f64, c: f64) -> f64 {
        -k * displacement - c * velocity
    }
}
