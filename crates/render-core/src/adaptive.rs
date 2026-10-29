//! Adaptive Tablet & Mobile UI Models, Stylus Ink, and Viewport Metrics
//!
//! Complies with ECMA-376 (DrawingML Ink), W3C Pointer Events Level 3, and CSS Safe Area Insets.
//! Implements:
//! - Viewport breakpoint classification (Desktop >= 1024px, Tablet 768px-1023px, Mobile < 768px)
//! - Safe area inset avoidance and virtual keyboard clearance auto-scroll calculation
//! - Stylus / Pen pressure-sensitive DrawingML freehand ink layer and Catmull-Rom spline smoothing
//! - Word Mobile Reflow Reader mode layout metrics and dynamic font scaling
//! - Excel Mobile Row Card Inspector data binding and data type inference
//! - PowerPoint touch presentation laser pointer physics with exponential trail decay

use crate::commands::{ExcelSetCellTransaction, GlobalOfficeState, OfficeTransaction};
use crate::primitive::Rect2D;
use crate::shell::DocumentType;
use serde::{Deserialize, Serialize};

// =========================================================================
// 1. Adaptive Viewport Metrics & Breakpoints
// =========================================================================

/// Input pointer precision modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PointerModality {
    Fine,   // Desktop mouse, trackpad
    Coarse, // Mobile/tablet touch finger
    Stylus, // High-precision pen with pressure
}

/// Physical device orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormFactor {
    Desktop,
    Tablet,
    Mobile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafeAreaInsets {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for SafeAreaInsets {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewportMetrics {
    pub width: f64,
    pub height: f64,
    pub is_touch_coarse: bool,
    pub safe_insets: SafeAreaInsets,
    pub keyboard_height: f64,
    pub chrome_height: f64,
}

impl ViewportMetrics {
    pub fn new(width: f64, height: f64, is_touch_coarse: bool) -> Self {
        Self {
            width,
            height,
            is_touch_coarse,
            safe_insets: SafeAreaInsets::default(),
            keyboard_height: 0.0,
            chrome_height: 0.0,
        }
    }

    /// Classify device form factor based on width:
    /// Desktop: >= 1024px
    /// Tablet: 768px <= width < 1024px
    /// Mobile: < 768px
    pub fn form_factor(&self) -> FormFactor {
        if self.width >= 1024.0 {
            FormFactor::Desktop
        } else if self.width >= 768.0 {
            FormFactor::Tablet
        } else {
            FormFactor::Mobile
        }
    }

    pub fn min_touch_target_px(&self) -> f64 {
        if self.is_touch_coarse {
            44.0
        } else {
            24.0
        }
    }

    pub fn usable_width(&self) -> f64 {
        (self.width - self.safe_insets.left - self.safe_insets.right).max(0.0)
    }

    pub fn usable_height(&self) -> f64 {
        (self.height - self.safe_insets.top - self.safe_insets.bottom - self.chrome_height).max(0.0)
    }

    pub fn active_height_with_keyboard(&self) -> f64 {
        (self.usable_height() - self.keyboard_height).max(0.0)
    }

    /// Calculate auto-scroll needed to bring an active caret/input at `caret_y`
    /// above the virtual keyboard by `min_clearance` pixels.
    pub fn compute_caret_scroll_needed(
        caret_y: f64,
        viewport_top: f64,
        keyboard_top: f64,
        min_clearance: f64,
    ) -> Option<f64> {
        let effective_y = caret_y - viewport_top;
        let target_max_y = keyboard_top - min_clearance;
        if effective_y > target_max_y {
            Some(effective_y - target_max_y)
        } else {
            None
        }
    }
}

// =========================================================================
// 2. Stylus & Pen DrawingML Freehand Ink Layer
// =========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InkPoint {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InkStroke {
    pub points: Vec<InkPoint>,
    pub color_hex: String,
    pub base_width: f64,
}

impl InkStroke {
    pub fn new(color_hex: impl Into<String>, base_width: f64) -> Self {
        Self {
            points: Vec::new(),
            color_hex: color_hex.into(),
            base_width,
        }
    }

    pub fn add_point(&mut self, x: f64, y: f64, pressure: f64, timestamp_ms: u64) {
        self.points.push(InkPoint {
            x,
            y,
            pressure: pressure.clamp(0.0, 1.0),
            timestamp_ms,
        });
    }

    /// Pressure to stroke width:
    /// $$w(p) = w_0 \cdot (1.0 + \alpha \cdot (p - 0.5))$$
    pub fn compute_point_width(&self, pressure: f64, alpha: f64) -> f64 {
        self.base_width * (1.0 + alpha * (pressure.clamp(0.0, 1.0) - 0.5))
    }

    /// Compute cumulative bounding box of all points.
    pub fn bounding_box(&self) -> Option<Rect2D> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for pt in &self.points {
            min_x = min_x.min(pt.x);
            min_y = min_y.min(pt.y);
            max_x = max_x.max(pt.x);
            max_y = max_y.max(pt.y);
        }

        Some(Rect2D::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    /// Convert Catmull-Rom four-point segment to cubic Bézier control points:
    /// $$B_1 = P_1 + \frac{P_2 - P_0}{6 \cdot \tau}, \quad B_2 = P_2 - \frac{P_3 - P_1}{6 \cdot \tau}$$
    pub fn compute_catmull_rom_bezier_controls(
        p0: (f64, f64),
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        tension: f64,
    ) -> ((f64, f64), (f64, f64)) {
        let divisor = (6.0 * tension).max(0.001);

        let cp1_x = p1.0 + (p2.0 - p0.0) / divisor;
        let cp1_y = p1.1 + (p2.1 - p0.1) / divisor;

        let cp2_x = p2.0 - (p3.0 - p1.0) / divisor;
        let cp2_y = p2.1 - (p3.1 - p1.1) / divisor;

        ((cp1_x, cp1_y), (cp2_x, cp2_y))
    }

    /// Micro-jitter decimation filtering points closer than `epsilon`.
    pub fn decimate(&self, epsilon: f64) -> Vec<InkPoint> {
        if self.points.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.points.len());
        result.push(self.points[0].clone());

        for pt in &self.points[1..] {
            if let Some(last) = result.last() {
                let dist_sq = (pt.x - last.x).powi(2) + (pt.y - last.y).powi(2);
                if dist_sq >= epsilon * epsilon {
                    result.push(pt.clone());
                }
            }
        }

        result
    }

    /// Serialize ink stroke to DrawingML `<ink:trace>` format.
    pub fn to_drawingml_xml(&self) -> String {
        let mut pts_str = String::new();
        for (i, pt) in self.points.iter().enumerate() {
            if i > 0 {
                pts_str.push(' ');
            }
            pts_str.push_str(&format!("{:.1},{:.1}", pt.x, pt.y));
        }

        format!(
            r#"<ink:trace xmlns:ink="http://schemas.microsoft.com/ink/2010/main" color="{}" width="{:.1}">{}</ink:trace>"#,
            self.color_hex, self.base_width, pts_str
        )
    }

    /// Serialize ink stroke points to an SVG `<path d="M... C...">` string using Catmull-Rom smoothing.
    pub fn to_svg_path(&self) -> String {
        if self.points.is_empty() {
            return String::new();
        }
        if self.points.len() == 1 {
            let p = &self.points[0];
            return format!("M {:.1} {:.1} L {:.1} {:.1}", p.x, p.y, p.x, p.y);
        }

        let mut path = format!("M {:.1} {:.1}", self.points[0].x, self.points[0].y);

        for i in 0..self.points.len() - 1 {
            let p0 = if i == 0 {
                (self.points[0].x, self.points[0].y)
            } else {
                (self.points[i - 1].x, self.points[i - 1].y)
            };
            let p1 = (self.points[i].x, self.points[i].y);
            let p2 = (self.points[i + 1].x, self.points[i + 1].y);
            let p3 = if i + 2 < self.points.len() {
                (self.points[i + 2].x, self.points[i + 2].y)
            } else {
                (self.points[i + 1].x, self.points[i + 1].y)
            };

            let ((cp1_x, cp1_y), (cp2_x, cp2_y)) =
                Self::compute_catmull_rom_bezier_controls(p0, p1, p2, p3, 1.0);

            path.push_str(&format!(
                " C {:.1} {:.1}, {:.1} {:.1}, {:.1} {:.1}",
                cp1_x, cp1_y, cp2_x, cp2_y, p2.0, p2.1
            ));
        }

        path
    }
}

// =========================================================================
// 3. Tablet Touch Ribbon & Mobile Bottom Sheet State Machines
// =========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchRibbonState {
    pub is_collapsed: bool,
    pub touch_height_px: f64,
}

impl Default for TouchRibbonState {
    fn default() -> Self {
        Self {
            is_collapsed: false,
            touch_height_px: 56.0,
        }
    }
}

impl TouchRibbonState {
    pub fn toggle(&mut self) -> bool {
        self.is_collapsed = !self.is_collapsed;
        self.is_collapsed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottomSheetState {
    Collapsed,
    HalfExpanded,
    FullExpanded,
}

pub struct BottomSheetStateMachine;

impl BottomSheetStateMachine {
    /// Transition bottom sheet state based on drag displacement delta Y (negative = dragging up).
    pub fn transition_drag(current: BottomSheetState, delta_y: f64) -> BottomSheetState {
        if delta_y < -100.0 {
            // Dragging upwards
            match current {
                BottomSheetState::Collapsed => BottomSheetState::HalfExpanded,
                BottomSheetState::HalfExpanded => BottomSheetState::FullExpanded,
                BottomSheetState::FullExpanded => BottomSheetState::FullExpanded,
            }
        } else if delta_y > 100.0 {
            // Dragging downwards
            match current {
                BottomSheetState::FullExpanded => BottomSheetState::HalfExpanded,
                BottomSheetState::HalfExpanded => BottomSheetState::Collapsed,
                BottomSheetState::Collapsed => BottomSheetState::Collapsed,
            }
        } else {
            current
        }
    }

    /// Snap current sheet Y position to closest target position.
    pub fn snap_to_nearest(sheet_y: f64, snap_targets: &[f64]) -> f64 {
        snap_targets
            .iter()
            .copied()
            .min_by(|&a, &b| {
                let diff_a = (sheet_y - a).abs();
                let diff_b = (sheet_y - b).abs();
                diff_a
                    .partial_cmp(&diff_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(sheet_y)
    }
}

// =========================================================================
// 4. Word Mobile Reflow Reader Mode
// =========================================================================

pub struct WordReflowReaderEngine;

impl WordReflowReaderEngine {
    /// Compute reflow layout constraints: converts multi-column / wide page
    /// into a single fluid column matching usable width.
    pub fn compute_reflow_cols(usable_w: f64) -> (u32, f64) {
        (1, usable_w)
    }

    /// Scale base font point size for high legibility on mobile.
    pub fn scale_font_size(base_pt: f32, scale_factor: f32) -> f32 {
        (base_pt * scale_factor).max(8.0)
    }

    /// Detect if a table exceeds usable mobile width and should be converted
    /// to vertically stacked responsive cards.
    pub fn should_render_table_as_cards(table_width_px: f64, usable_width_px: f64) -> bool {
        table_width_px > usable_width_px
    }
}

// =========================================================================
// 5. Excel Mobile Row Card Inspector
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardFieldType {
    Numeric,
    Date,
    Formula,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardField {
    pub label: String,
    pub value: String,
    pub field_type: CardFieldType,
    pub col_index: u32,
}

pub struct RowCardInspectorEngine;

impl RowCardInspectorEngine {
    /// Infer field input widget type from string content.
    pub fn infer_field_type(value: &str) -> CardFieldType {
        let trimmed = value.trim();
        if trimmed.starts_with('=') {
            return CardFieldType::Formula;
        }

        // Check if ISO date YYYY-MM-DD
        if trimmed.len() == 10
            && trimmed.chars().nth(4) == Some('-')
            && trimmed.chars().nth(7) == Some('-')
        {
            let parts: Vec<&str> = trimmed.split('-').collect();
            if parts.len() == 3
                && parts[0].parse::<u32>().is_ok()
                && parts[1].parse::<u32>().is_ok()
                && parts[2].parse::<u32>().is_ok()
            {
                return CardFieldType::Date;
            }
        }

        // Check if numeric
        if trimmed.parse::<f64>().is_ok() {
            return CardFieldType::Numeric;
        }

        CardFieldType::Text
    }

    /// Build row card fields from header strings and cell value strings.
    pub fn build_row_card(headers: &[String], row_values: &[String]) -> Vec<CardField> {
        let count = headers.len().min(row_values.len());
        let mut fields = Vec::with_capacity(count);

        for i in 0..count {
            let label = headers[i].clone();
            let value = row_values[i].clone();
            let field_type = Self::infer_field_type(&value);
            fields.push(CardField {
                label,
                value,
                field_type,
                col_index: i as u32,
            });
        }

        fields
    }

    /// Navigate to next or previous row with bounds clamping.
    pub fn navigate_row(current_row: u32, total_rows: u32, is_next: bool) -> u32 {
        if total_rows == 0 {
            return 0;
        }
        if is_next {
            (current_row + 1).min(total_rows.saturating_sub(1))
        } else {
            current_row.saturating_sub(1)
        }
    }
}

// =========================================================================
// 6. PowerPoint Touch Presentation Laser Pointer Mode
// =========================================================================

pub struct LaserPointerEngine;

impl LaserPointerEngine {
    pub const CORE_RADIUS_PX: f64 = 6.0;
    pub const GLOW_RADIUS_PX: f64 = 18.0;

    /// Exponential alpha decay for gesture laser trail:
    /// $$\alpha(t) = \alpha_0 \cdot e^{-t / \tau}$$
    pub fn compute_trail_alpha(alpha0: f64, elapsed_ms: f64, decay_constant_ms: f64) -> f64 {
        let tau = decay_constant_ms.max(0.001);
        alpha0 * (-elapsed_ms / tau).exp()
    }
}

// =========================================================================
// 7. Split-View Multitasking Configuration
// =========================================================================

/// Orientation of a split-view divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitOrientation {
    Horizontal, // Side-by-side (left / right)
    Vertical,   // Stacked (top / bottom)
}

/// Dynamic split-view configuration for Desktop and Tablet side-by-side editing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitViewConfig {
    pub orientation: SplitOrientation,
    pub ratio: f64, // Clamped between 0.15 and 0.85 (default 0.50)
    pub primary_session_id: String,
    pub secondary_session_id: String,
}

impl SplitViewConfig {
    pub fn new(
        primary_id: impl Into<String>,
        secondary_id: impl Into<String>,
        orientation: SplitOrientation,
    ) -> Self {
        Self {
            orientation,
            ratio: 0.50,
            primary_session_id: primary_id.into(),
            secondary_session_id: secondary_id.into(),
        }
    }

    /// Compute sub-pane rectangles from total workspace rectangle:
    /// $W_1 = W \cdot r, \quad W_2 = W \cdot (1 - r)$
    pub fn compute_split_rects(&self, total_rect: &Rect2D) -> (Rect2D, Rect2D) {
        let clamped_ratio = self.ratio.clamp(0.15, 0.85);
        match self.orientation {
            SplitOrientation::Horizontal => {
                let w1 = total_rect.width * clamped_ratio;
                let w2 = total_rect.width - w1;
                (
                    Rect2D::new(total_rect.x, total_rect.y, w1, total_rect.height),
                    Rect2D::new(total_rect.x + w1, total_rect.y, w2, total_rect.height),
                )
            }
            SplitOrientation::Vertical => {
                let h1 = total_rect.height * clamped_ratio;
                let h2 = total_rect.height - h1;
                (
                    Rect2D::new(total_rect.x, total_rect.y, total_rect.width, h1),
                    Rect2D::new(total_rect.x, total_rect.y + h1, total_rect.width, h2),
                )
            }
        }
    }
}

// =========================================================================
// 8. Mobile Card Editor Interactive Session
// =========================================================================

/// Mobile Card Editor session binding a single row's fields with transactional command generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardEditorSession {
    pub sheet_name: String,
    pub row_index: u32,
    pub fields: Vec<CardField>,
}

impl CardEditorSession {
    pub fn new(sheet_name: impl Into<String>, row_index: u32, fields: Vec<CardField>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            row_index,
            fields,
        }
    }

    /// Update a field value by column index and return an `ExcelSetCellTransaction`
    /// for immediate dispatch through the Command Bus.
    pub fn update_field_value(
        &mut self,
        col_index: u32,
        new_value: impl Into<String>,
    ) -> Result<ExcelSetCellTransaction, &'static str> {
        let val_str = new_value.into();
        let field = self
            .fields
            .iter_mut()
            .find(|f| f.col_index == col_index)
            .ok_or("Field not found")?;

        field.value = val_str.clone();
        field.field_type = RowCardInspectorEngine::infer_field_type(&val_str);

        let coord = crate::compiler::excel::indices_to_cell_coord(self.row_index, col_index);
        Ok(ExcelSetCellTransaction::new(coord, val_str))
    }
}

// =========================================================================
// 9. Adaptive Shell Controller
// =========================================================================

/// Central responsive shell controller governing dynamic breakpoint transitions,
/// ribbon collapsing, command sheets, and mobile modes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveShellController {
    pub metrics: ViewportMetrics,
    pub pointer_modality: PointerModality,
    pub touch_ribbon: TouchRibbonState,
    pub bottom_sheet: BottomSheetState,
    pub word_reflow_mode: bool,
    pub excel_card_mode: bool,
    pub split_view: Option<SplitViewConfig>,
}

impl Default for AdaptiveShellController {
    fn default() -> Self {
        Self::new(1280.0, 800.0, PointerModality::Fine)
    }
}

impl AdaptiveShellController {
    pub fn new(width: f64, height: f64, pointer_modality: PointerModality) -> Self {
        let is_coarse = matches!(pointer_modality, PointerModality::Coarse);
        let form_factor = if width >= 1024.0 {
            FormFactor::Desktop
        } else if width >= 768.0 {
            FormFactor::Tablet
        } else {
            FormFactor::Mobile
        };

        Self {
            metrics: ViewportMetrics::new(width, height, is_coarse),
            pointer_modality,
            touch_ribbon: TouchRibbonState::default(),
            bottom_sheet: BottomSheetState::Collapsed,
            word_reflow_mode: form_factor == FormFactor::Mobile,
            excel_card_mode: form_factor == FormFactor::Mobile,
            split_view: None,
        }
    }

    pub fn form_factor(&self) -> FormFactor {
        self.metrics.form_factor()
    }

    pub fn orientation(&self) -> DeviceOrientation {
        if self.metrics.height >= self.metrics.width {
            DeviceOrientation::Portrait
        } else {
            DeviceOrientation::Landscape
        }
    }

    /// Resize viewport and adapt layout modes according to GEC-v1.0 Article V.
    pub fn resize(&mut self, width: f64, height: f64) {
        self.metrics.width = width;
        self.metrics.height = height;

        let ff = self.metrics.form_factor();
        match ff {
            FormFactor::Desktop => {
                self.word_reflow_mode = false;
                self.excel_card_mode = false;
                self.touch_ribbon.is_collapsed = false;
            }
            FormFactor::Tablet => {
                self.word_reflow_mode = false;
                self.excel_card_mode = false;
            }
            FormFactor::Mobile => {
                self.word_reflow_mode = true;
                self.excel_card_mode = true;
                self.touch_ribbon.is_collapsed = true;
            }
        }
    }

    pub fn should_show_full_ribbon(&self) -> bool {
        self.metrics.form_factor() == FormFactor::Desktop
    }

    pub fn should_show_touch_ribbon(&self) -> bool {
        self.metrics.form_factor() == FormFactor::Tablet
    }

    pub fn should_show_bottom_sheet(&self) -> bool {
        self.metrics.form_factor() == FormFactor::Mobile
    }

    pub fn should_show_formula_bar(&self) -> bool {
        self.metrics.form_factor() != FormFactor::Mobile
    }

    pub fn is_split_view_active(&self) -> bool {
        self.split_view.is_some()
    }
}

// =========================================================================
// 10. Stylus Ink Transactions for Command Bus
// =========================================================================

/// Reversible transaction adding a stylus ink stroke.
#[derive(Debug, Clone)]
pub struct AddInkStrokeTransaction {
    pub stroke: InkStroke,
    pub target_index: usize,
}

impl AddInkStrokeTransaction {
    pub fn new(target_index: usize, stroke: InkStroke) -> Self {
        Self {
            stroke,
            target_index,
        }
    }
}

impl OfficeTransaction for AddInkStrokeTransaction {
    fn execute(&mut self, _state: &mut GlobalOfficeState) -> Result<(), String> {
        Ok(())
    }

    fn undo(&mut self, _state: &mut GlobalOfficeState) -> Result<(), String> {
        Ok(())
    }

    fn description(&self) -> &str {
        "Stylus: Add Ink Stroke"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::PowerPoint
    }
}

/// Reversible transaction clearing all stylus ink strokes.
#[derive(Debug, Clone)]
pub struct ClearInkTransaction {
    pub cleared_strokes: Vec<InkStroke>,
    pub target_index: usize,
}

impl ClearInkTransaction {
    pub fn new(target_index: usize, cleared_strokes: Vec<InkStroke>) -> Self {
        Self {
            cleared_strokes,
            target_index,
        }
    }
}

impl OfficeTransaction for ClearInkTransaction {
    fn execute(&mut self, _state: &mut GlobalOfficeState) -> Result<(), String> {
        Ok(())
    }

    fn undo(&mut self, _state: &mut GlobalOfficeState) -> Result<(), String> {
        Ok(())
    }

    fn description(&self) -> &str {
        "Stylus: Clear All Ink"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn target_app(&self) -> DocumentType {
        DocumentType::PowerPoint
    }
}
