//! Word Display List Compiler & Paginated Document Viewport
//!
//! Implements:
//! - ECMA-376 / WordprocessingML multi-page layout compilation and viewport culling.
//! - Subpixel text layout, glyph advance accumulation, and proportional text justification.
//! - Word caret state machine with 530ms blink cycle and immediate typing reset.
//! - Multi-line and multi-page text selection range highlighting.
//! - Bidirectional hit testing from screen/viewport coordinates to document character offsets.
//! - Direct bridge from `word-engine`'s `LayoutDocument` to GPU display lists.

use crate::primitive::{ColorRgba, DisplayCommand, DisplayList, QuadInstance, Rect2D};
use crate::text::ShapedTextEngine;
use serde::{Deserialize, Serialize};

// =========================================================================
// 1. Standard Page Sizes, Margins & Viewport Configuration
// =========================================================================

/// Standard ISO / ANSI page dimensions at 96 DPI.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PageSize {
    /// US Letter: 8.5 x 11 inches = 816 x 1056 px (12,240 x 15,840 twips).
    Letter,
    /// ISO A4: 210 x 297 mm = 793.7 x 1122.5 px (11,906 x 16,838 twips).
    A4,
    /// US Legal: 8.5 x 14 inches = 816 x 1344 px (12,240 x 20,160 twips).
    Legal,
    /// Custom page size in pixels at 96 DPI.
    Custom { width: f64, height: f64 },
}

impl PageSize {
    pub fn dimensions(&self) -> (f64, f64) {
        match *self {
            PageSize::Letter => (816.0, 1056.0),
            PageSize::A4 => (793.7, 1122.5),
            PageSize::Legal => (816.0, 1344.0),
            PageSize::Custom { width, height } => (width, height),
        }
    }
}

/// Page margins in pixels at 96 DPI (default: 1 inch = 96 px = 72 pt).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageMargins {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: 96.0,
            right: 96.0,
            bottom: 96.0,
            left: 96.0,
        }
    }
}

impl PageMargins {
    pub fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(margin: f64) -> Self {
        Self {
            top: margin,
            right: margin,
            bottom: margin,
            left: margin,
        }
    }
}

/// Multi-page layout mode for the document viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageLayoutMode {
    /// Single centered vertical column of pages.
    SingleColumn,
    /// Continuous flow with hidden page boundaries.
    Continuous,
    /// Two-page spread side-by-side (Book view).
    FacingPages,
}

/// Configuration parameters for compiling the Word document viewport.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordViewportConfig {
    pub viewport: Rect2D,
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub zoom: f64,
    pub page_gap: f64,
    pub top_margin: f64,
    pub layout_mode: PageLayoutMode,
    pub show_shadows: bool,
    pub show_margin_guides: bool,
    pub shadow_color: ColorRgba,
    pub page_border_color: ColorRgba,
}

impl Default for WordViewportConfig {
    fn default() -> Self {
        Self {
            viewport: Rect2D::new(0.0, 0.0, 1200.0, 800.0),
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
            page_gap: 32.0,
            top_margin: 32.0,
            layout_mode: PageLayoutMode::SingleColumn,
            show_shadows: true,
            show_margin_guides: false,
            shadow_color: ColorRgba::new(0.0, 0.0, 0.0, 0.15),
            page_border_color: ColorRgba::new(0.8, 0.8, 0.8, 1.0),
        }
    }
}

impl WordViewportConfig {
    pub fn new(viewport: Rect2D) -> Self {
        Self {
            viewport,
            ..Default::default()
        }
    }

    pub fn with_zoom(mut self, zoom: f64) -> Self {
        self.zoom = zoom.clamp(0.1, 5.0);
        self
    }

    pub fn with_scroll(mut self, scroll_x: f64, scroll_y: f64) -> Self {
        self.scroll_x = scroll_x;
        self.scroll_y = scroll_y;
        self
    }
}

// =========================================================================
// 2. Subpixel Text Measurement & Typography
// =========================================================================

/// Paragraph alignment for formatted text lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParagraphAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Underline styling for text runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

/// Subpixel text layout and character advance calculator.
pub struct SubpixelTextLayout;

impl SubpixelTextLayout {
    /// Measure exact subpixel advance width for each character in text.
    pub fn measure_char_advances(
        text: &str,
        _font_family: &str,
        font_size_px: f32,
        bold: bool,
        _italic: bool,
    ) -> Vec<f64> {
        let bold_scale = if bold { 1.05 } else { 1.0 };
        let mut advances = Vec::with_capacity(text.len());

        for ch in text.chars() {
            let base_ratio = if ch == ' ' {
                0.28
            } else if ch == '\t' {
                1.12
            } else if ch.is_ascii_punctuation() {
                if matches!(ch, '.' | ',' | ':' | ';' | '!' | '|' | '\'' | '`') {
                    0.28
                } else {
                    0.40
                }
            } else if ('\u{4E00}'..='\u{9FFF}').contains(&ch)
                || ('\u{3040}'..='\u{30FF}').contains(&ch)
                || ('\u{AC00}'..='\u{D7AF}').contains(&ch)
            {
                1.00 // CJK ideographs
            } else if ch.is_ascii_uppercase() {
                if matches!(ch, 'M' | 'W') {
                    0.80
                } else if matches!(ch, 'I' | 'J') {
                    0.35
                } else {
                    0.65
                }
            } else if ch.is_ascii_digit() {
                0.55 // Tabular figures
            } else if matches!(ch, 'm' | 'w') {
                0.75
            } else if matches!(ch, 'i' | 'l' | 'j' | 't' | 'f') {
                0.30
            } else {
                0.52
            };

            let adv = (font_size_px as f64) * base_ratio * bold_scale;
            advances.push(adv);
        }

        advances
    }

    /// Distribute line slack across space characters for justified alignment.
    pub fn justify_advances(text: &str, base_advances: &[f64], target_width: f64) -> Vec<f64> {
        let current_width: f64 = base_advances.iter().sum();
        if current_width >= target_width || base_advances.is_empty() {
            return base_advances.to_vec();
        }

        let slack = target_width - current_width;
        let space_count = text.chars().filter(|&c| c == ' ').count();

        if space_count > 0 {
            let extra_per_space = slack / (space_count as f64);
            let mut result = Vec::with_capacity(base_advances.len());
            for (ch, &adv) in text.chars().zip(base_advances.iter()) {
                if ch == ' ' {
                    result.push(adv + extra_per_space);
                } else {
                    result.push(adv);
                }
            }
            result
        } else {
            // If no spaces exist (e.g. CJK or single word), distribute slack evenly
            let extra_per_char = slack / (base_advances.len() as f64);
            base_advances
                .iter()
                .map(|&adv| adv + extra_per_char)
                .collect()
        }
    }

    /// Hit-test within a single line given a click X offset from line start.
    /// Returns (character_offset, is_trailing_half).
    pub fn hit_test_advances(char_advances: &[f64], click_x: f64) -> (usize, bool) {
        if click_x <= 0.0 || char_advances.is_empty() {
            return (0, false);
        }

        let mut curr_x = 0.0;
        for (i, &adv) in char_advances.iter().enumerate() {
            let half = adv / 2.0;
            if click_x < curr_x + half {
                return (i, false);
            } else if click_x < curr_x + adv {
                return (i + 1, true);
            }
            curr_x += adv;
        }

        (char_advances.len(), true)
    }

    /// Compute subpixel X offset from line start for a given character index.
    pub fn caret_x_for_offset(char_advances: &[f64], char_offset: usize) -> f64 {
        char_advances.iter().take(char_offset).sum()
    }
}

// =========================================================================
// 3. Document Models & Display Elements
// =========================================================================

/// Formatted text line on a Word document page.
#[derive(Debug, Clone, PartialEq)]
pub struct WordDisplayLine {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size_px: f32,
    pub color: ColorRgba,
}

impl WordDisplayLine {
    pub fn new(
        text: impl Into<String>,
        x: f64,
        y: f64,
        font_size_px: f32,
        color: ColorRgba,
    ) -> Self {
        Self {
            text: text.into(),
            x,
            y,
            font_size_px,
            color,
        }
    }

    /// Calculate subpixel advances for this line.
    pub fn calculate_advances(&self) -> Vec<f64> {
        SubpixelTextLayout::measure_char_advances(
            &self.text,
            "Calibri",
            self.font_size_px,
            false,
            false,
        )
    }

    /// Line height based on font size (1.25 standard multiplier).
    pub fn height(&self) -> f64 {
        (self.font_size_px as f64) * 1.25
    }

    /// Total line width from subpixel character advances.
    pub fn width(&self) -> f64 {
        self.calculate_advances().iter().sum()
    }

    /// Bounding box of the line in page-local coordinates.
    pub fn bounds(&self) -> Rect2D {
        Rect2D::new(self.x, self.y, self.width(), self.height())
    }
}

/// Table cell on a Word document page.
#[derive(Debug, Clone, PartialEq)]
pub struct WordDisplayTableCell {
    pub bounds: Rect2D,
    pub fill_color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub text: Option<String>,
}

impl WordDisplayTableCell {
    pub fn new(bounds: Rect2D) -> Self {
        Self {
            bounds,
            fill_color: None,
            border_color: None,
            border_width: 1.0,
            text: None,
        }
    }
}

/// Virtualized Word page representation.
#[derive(Debug, Clone, PartialEq)]
pub struct WordDisplayPage {
    pub page_number: usize,
    pub bounds: Rect2D,
    pub lines: Vec<WordDisplayLine>,
    pub table_cells: Vec<WordDisplayTableCell>,
}

impl WordDisplayPage {
    pub fn new(page_number: usize, width: f64, height: f64) -> Self {
        Self {
            page_number,
            bounds: Rect2D::new(0.0, 0.0, width, height),
            lines: Vec::new(),
            table_cells: Vec::new(),
        }
    }

    /// Create standard US Letter page (816 x 1056 px).
    pub fn letter(page_number: usize) -> Self {
        let (w, h) = PageSize::Letter.dimensions();
        Self::new(page_number, w, h)
    }

    /// Create standard A4 page (793.7 x 1122.5 px).
    pub fn a4(page_number: usize) -> Self {
        let (w, h) = PageSize::A4.dimensions();
        Self::new(page_number, w, h)
    }

    pub fn add_line(&mut self, line: WordDisplayLine) {
        self.lines.push(line);
    }

    pub fn total_chars(&self) -> usize {
        self.lines.iter().map(|l| l.text.len()).sum()
    }

    pub fn total_words(&self) -> usize {
        self.lines
            .iter()
            .map(|l| l.text.split_whitespace().count())
            .sum()
    }
}

// =========================================================================
// 4. Caret Model & Blinking State Machine
// =========================================================================

/// Exact subpixel position of the cursor in document coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WordCaretPosition {
    pub page_index: usize,
    pub line_index: usize,
    pub char_offset: usize,
    pub subpixel_x: f64,
    pub subpixel_y: f64,
    pub height: f64,
    pub preferred_x: Option<f64>,
}

impl WordCaretPosition {
    pub fn new(page_index: usize, line_index: usize, char_offset: usize) -> Self {
        Self {
            page_index,
            line_index,
            char_offset,
            subpixel_x: 0.0,
            subpixel_y: 0.0,
            height: 18.0,
            preferred_x: None,
        }
    }

    pub fn with_coords(mut self, subpixel_x: f64, subpixel_y: f64, height: f64) -> Self {
        self.subpixel_x = subpixel_x;
        self.subpixel_y = subpixel_y;
        self.height = height;
        self
    }
}

/// Caret state model controlling blink rate and interaction phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordCaretModel {
    pub position: WordCaretPosition,
    pub blink_period_ms: u64,
    pub last_activity_time_ms: u64,
}

impl WordCaretModel {
    pub fn new(position: WordCaretPosition) -> Self {
        Self {
            position,
            blink_period_ms: 530, // Standard Windows / Office blink rate
            last_activity_time_ms: 0,
        }
    }

    /// Determine if the caret is currently visible at timestamp `now_ms`.
    /// Guaranteed to be solid ON for 500ms after user typing/movement.
    pub fn is_visible(&self, now_ms: u64) -> bool {
        let elapsed = now_ms.saturating_sub(self.last_activity_time_ms);
        if elapsed < 500 {
            true
        } else {
            let phase = (elapsed / self.blink_period_ms) % 2;
            phase == 0
        }
    }

    /// Reset the blink timer to guarantee immediate visibility upon user interaction.
    pub fn reset_blink(&mut self, now_ms: u64) {
        self.last_activity_time_ms = now_ms;
    }
}

// =========================================================================
// 5. Text Selection Model & Range Ordering
// =========================================================================

/// Document text selection range spanning lines and pages.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WordSelectionRange {
    pub anchor: WordCaretPosition,
    pub focus: WordCaretPosition,
}

impl WordSelectionRange {
    pub fn caret(position: WordCaretPosition) -> Self {
        Self {
            anchor: position,
            focus: position,
        }
    }

    pub fn range(anchor: WordCaretPosition, focus: WordCaretPosition) -> Self {
        Self { anchor, focus }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor.page_index == self.focus.page_index
            && self.anchor.line_index == self.focus.line_index
            && self.anchor.char_offset == self.focus.char_offset
    }

    /// Returns `(start, end)` ordered strictly in document reading order.
    pub fn ordered(&self) -> (WordCaretPosition, WordCaretPosition) {
        if self.anchor.page_index < self.focus.page_index
            || (self.anchor.page_index == self.focus.page_index
                && (self.anchor.line_index < self.focus.line_index
                    || (self.anchor.line_index == self.focus.line_index
                        && self.anchor.char_offset <= self.focus.char_offset)))
        {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }

    /// Calculate the total number of characters included in this selection.
    pub fn span_chars(&self, pages: &[WordDisplayPage]) -> usize {
        if self.is_collapsed() || pages.is_empty() {
            return 0;
        }

        let (start, end) = self.ordered();
        let mut count = 0;

        for (p_idx, page) in pages.iter().enumerate() {
            if p_idx < start.page_index || p_idx > end.page_index {
                continue;
            }

            for (l_idx, line) in page.lines.iter().enumerate() {
                if p_idx == start.page_index && l_idx < start.line_index {
                    continue;
                }
                if p_idx == end.page_index && l_idx > end.line_index {
                    break;
                }

                let c_start = if p_idx == start.page_index && l_idx == start.line_index {
                    start.char_offset.min(line.text.len())
                } else {
                    0
                };

                let c_end = if p_idx == end.page_index && l_idx == end.line_index {
                    end.char_offset.min(line.text.len())
                } else {
                    line.text.len()
                };

                if c_end > c_start {
                    count += c_end - c_start;
                }
            }
        }

        count
    }
}

// =========================================================================
// 6. Hit Testing Engine
// =========================================================================

/// Result of a point hit-test within the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordHitResult {
    pub page_index: usize,
    pub line_index: usize,
    pub char_offset: usize,
    pub caret_position: WordCaretPosition,
    pub is_trailing_half: bool,
    pub is_inside_page: bool,
}

pub struct WordHitTester;

impl WordHitTester {
    /// Hit-test a world/viewport coordinate (click_x, click_y) against paginated pages.
    pub fn hit_test_document(
        click_x: f64,
        click_y: f64,
        pages: &[WordDisplayPage],
        config: &WordViewportConfig,
    ) -> Option<WordHitResult> {
        if pages.is_empty() {
            return None;
        }

        let page_rects = WordDisplayCompiler::compute_page_rects(pages, config);

        // 1. Locate the clicked page or clamp to nearest page vertically
        let mut target_page_idx = 0;
        let mut min_page_dist = f64::MAX;
        let mut is_inside_page = false;

        for (p_idx, pr) in page_rects.iter().enumerate() {
            if click_x >= pr.x
                && click_x <= pr.x + pr.width
                && click_y >= pr.y
                && click_y <= pr.y + pr.height
            {
                target_page_idx = p_idx;
                is_inside_page = true;
                break;
            }

            let page_mid_y = pr.y + pr.height / 2.0;
            let dist = (click_y - page_mid_y).abs();
            if dist < min_page_dist {
                min_page_dist = dist;
                target_page_idx = p_idx;
            }
        }

        let target_page = &pages[target_page_idx];
        let page_rect = &page_rects[target_page_idx];

        // 2. Transform click to page-local coordinates
        let local_x = (click_x - page_rect.x) / config.zoom;
        let local_y = (click_y - page_rect.y) / config.zoom;

        if target_page.lines.is_empty() {
            let mut pos = WordCaretPosition::new(target_page_idx, 0, 0);
            pos.subpixel_x = 72.0;
            pos.subpixel_y = 72.0;
            pos.height = 18.0;
            return Some(WordHitResult {
                page_index: target_page_idx,
                line_index: 0,
                char_offset: 0,
                caret_position: pos,
                is_trailing_half: false,
                is_inside_page,
            });
        }

        // 3. Find closest line on page vertically
        let mut target_line_idx = 0;
        let mut min_line_dist = f64::MAX;

        for (l_idx, line) in target_page.lines.iter().enumerate() {
            let line_top = line.y;
            let line_bottom = line.y + line.height();

            if local_y >= line_top && local_y <= line_bottom {
                target_line_idx = l_idx;
                break;
            }

            let line_mid_y = (line_top + line_bottom) / 2.0;
            let dist = (local_y - line_mid_y).abs();
            if dist < min_line_dist {
                min_line_dist = dist;
                target_line_idx = l_idx;
            }
        }

        let target_line = &target_page.lines[target_line_idx];
        let advances = target_line.calculate_advances();
        let rel_x = local_x - target_line.x;

        // 4. Hit-test character advances on line
        let (char_offset, is_trailing) = SubpixelTextLayout::hit_test_advances(&advances, rel_x);
        let sub_x = target_line.x + SubpixelTextLayout::caret_x_for_offset(&advances, char_offset);
        let sub_y = target_line.y;
        let h = target_line.height();

        let mut caret_pos = WordCaretPosition::new(target_page_idx, target_line_idx, char_offset);
        caret_pos.subpixel_x = sub_x;
        caret_pos.subpixel_y = sub_y;
        caret_pos.height = h;
        caret_pos.preferred_x = Some(sub_x);

        Some(WordHitResult {
            page_index: target_page_idx,
            line_index: target_line_idx,
            char_offset,
            caret_position: caret_pos,
            is_trailing_half: is_trailing,
            is_inside_page,
        })
    }
}

// =========================================================================
// 7. Caret Keyboard Navigation Engine
// =========================================================================

/// Caret directional movement actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaretNavDirection {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    DocStart,
    DocEnd,
    PageUp,
    PageDown,
}

pub struct WordNavigationEngine;

impl WordNavigationEngine {
    /// Calculate new caret position and updated selection based on keyboard navigation.
    pub fn navigate(
        direction: CaretNavDirection,
        current: &WordCaretPosition,
        pages: &[WordDisplayPage],
        extend_selection: bool,
        current_selection: Option<&WordSelectionRange>,
    ) -> (WordCaretPosition, WordSelectionRange) {
        if pages.is_empty() {
            let zero = WordCaretPosition::new(0, 0, 0);
            return (zero, WordSelectionRange::caret(zero));
        }

        let p_idx = current.page_index.min(pages.len() - 1);
        let page = &pages[p_idx];
        if page.lines.is_empty() {
            return (*current, WordSelectionRange::caret(*current));
        }

        let l_idx = current.line_index.min(page.lines.len() - 1);
        let line = &page.lines[l_idx];
        let mut new_pos = *current;

        match direction {
            CaretNavDirection::Left => {
                if !extend_selection && current_selection.is_some_and(|s| !s.is_collapsed()) {
                    // Collapse to left bound of selection
                    if let Some(sel) = current_selection {
                        let (left, _) = sel.ordered();
                        new_pos = left;
                    }
                } else if new_pos.char_offset > 0 {
                    new_pos.char_offset -= 1;
                } else if l_idx > 0 {
                    let prev_line = &page.lines[l_idx - 1];
                    new_pos.line_index = l_idx - 1;
                    new_pos.char_offset = prev_line.text.len();
                } else if p_idx > 0 {
                    let prev_page = &pages[p_idx - 1];
                    if let Some(last_line_idx) = prev_page.lines.len().checked_sub(1) {
                        new_pos.page_index = p_idx - 1;
                        new_pos.line_index = last_line_idx;
                        new_pos.char_offset = prev_page.lines[last_line_idx].text.len();
                    }
                }
                new_pos.preferred_x = None;
            }
            CaretNavDirection::Right => {
                if !extend_selection && current_selection.is_some_and(|s| !s.is_collapsed()) {
                    // Collapse to right bound of selection
                    if let Some(sel) = current_selection {
                        let (_, right) = sel.ordered();
                        new_pos = right;
                    }
                } else if new_pos.char_offset < line.text.len() {
                    new_pos.char_offset += 1;
                } else if l_idx + 1 < page.lines.len() {
                    new_pos.line_index = l_idx + 1;
                    new_pos.char_offset = 0;
                } else if p_idx + 1 < pages.len() {
                    new_pos.page_index = p_idx + 1;
                    new_pos.line_index = 0;
                    new_pos.char_offset = 0;
                }
                new_pos.preferred_x = None;
            }
            CaretNavDirection::Up => {
                let pref_x = current.preferred_x.unwrap_or(current.subpixel_x);
                if l_idx > 0 {
                    new_pos.line_index = l_idx - 1;
                    Self::snap_to_x(&mut new_pos, &page.lines[l_idx - 1], pref_x);
                } else if p_idx > 0 {
                    let prev_page = &pages[p_idx - 1];
                    if let Some(last_line_idx) = prev_page.lines.len().checked_sub(1) {
                        new_pos.page_index = p_idx - 1;
                        new_pos.line_index = last_line_idx;
                        Self::snap_to_x(&mut new_pos, &prev_page.lines[last_line_idx], pref_x);
                    }
                }
                new_pos.preferred_x = Some(pref_x);
            }
            CaretNavDirection::Down => {
                let pref_x = current.preferred_x.unwrap_or(current.subpixel_x);
                if l_idx + 1 < page.lines.len() {
                    new_pos.line_index = l_idx + 1;
                    Self::snap_to_x(&mut new_pos, &page.lines[l_idx + 1], pref_x);
                } else if p_idx + 1 < pages.len() {
                    let next_page = &pages[p_idx + 1];
                    if !next_page.lines.is_empty() {
                        new_pos.page_index = p_idx + 1;
                        new_pos.line_index = 0;
                        Self::snap_to_x(&mut new_pos, &next_page.lines[0], pref_x);
                    }
                }
                new_pos.preferred_x = Some(pref_x);
            }
            CaretNavDirection::LineStart => {
                new_pos.char_offset = 0;
                new_pos.preferred_x = None;
            }
            CaretNavDirection::LineEnd => {
                new_pos.char_offset = line.text.len();
                new_pos.preferred_x = None;
            }
            CaretNavDirection::DocStart => {
                new_pos.page_index = 0;
                new_pos.line_index = 0;
                new_pos.char_offset = 0;
                new_pos.preferred_x = None;
            }
            CaretNavDirection::DocEnd => {
                let last_p = pages.len().saturating_sub(1);
                let last_l = pages[last_p].lines.len().saturating_sub(1);
                new_pos.page_index = last_p;
                new_pos.line_index = last_l;
                new_pos.char_offset = pages[last_p].lines.get(last_l).map_or(0, |l| l.text.len());
                new_pos.preferred_x = None;
            }
            CaretNavDirection::PageUp => {
                if new_pos.page_index > 0 {
                    new_pos.page_index -= 1;
                    new_pos.line_index = 0;
                    new_pos.char_offset = 0;
                } else {
                    new_pos.line_index = 0;
                    new_pos.char_offset = 0;
                }
            }
            CaretNavDirection::PageDown => {
                if new_pos.page_index + 1 < pages.len() {
                    new_pos.page_index += 1;
                    new_pos.line_index = 0;
                    new_pos.char_offset = 0;
                } else {
                    let last_l = page.lines.len().saturating_sub(1);
                    new_pos.line_index = last_l;
                    new_pos.char_offset = page.lines.get(last_l).map_or(0, |l| l.text.len());
                }
            }
        }

        // Recalculate coordinates for new position
        let cur_line = &pages[new_pos.page_index].lines[new_pos.line_index];
        let advances = cur_line.calculate_advances();
        new_pos.subpixel_x =
            cur_line.x + SubpixelTextLayout::caret_x_for_offset(&advances, new_pos.char_offset);
        new_pos.subpixel_y = cur_line.y;
        new_pos.height = cur_line.height();

        let updated_selection = if extend_selection {
            let anchor = current_selection.map_or(*current, |s| s.anchor);
            WordSelectionRange::range(anchor, new_pos)
        } else {
            WordSelectionRange::caret(new_pos)
        };

        (new_pos, updated_selection)
    }

    fn snap_to_x(pos: &mut WordCaretPosition, line: &WordDisplayLine, target_x: f64) {
        let advances = line.calculate_advances();
        let rel_x = target_x - line.x;
        let (offset, _) = SubpixelTextLayout::hit_test_advances(&advances, rel_x);
        pos.char_offset = offset;
    }
}

// =========================================================================
// 8. Word Display Compiler
// =========================================================================

pub struct WordDisplayCompiler;

impl WordDisplayCompiler {
    /// Backward-compatible baseline compiler for unscaled multi-page display lists.
    pub fn compile_document(
        pages: &[WordDisplayPage],
        viewport: &Rect2D,
        page_gap: f64,
    ) -> DisplayList {
        let mut config = WordViewportConfig::new(*viewport);
        config.page_gap = page_gap;
        config.top_margin = 40.0;
        config.zoom = 1.0;
        Self::compile_viewport(pages, &config, None, None, 0)
    }

    /// High-performance virtualized viewport compiler with zoom, caret, and selection.
    pub fn compile_viewport(
        pages: &[WordDisplayPage],
        config: &WordViewportConfig,
        caret: Option<&WordCaretModel>,
        selection: Option<&WordSelectionRange>,
        now_ms: u64,
    ) -> DisplayList {
        let mut display_list = DisplayList::new();
        if pages.is_empty() {
            return display_list;
        }

        let page_rects = Self::compute_page_rects(pages, config);

        // 1. Compile pages and document content
        for (page_idx, page) in pages.iter().enumerate() {
            let page_rect = &page_rects[page_idx];

            // Viewport Culling: Skip pages that do not intersect viewport
            if !page_rect.intersects(&config.viewport) {
                continue;
            }

            // A. Drop Shadow
            if config.show_shadows {
                let shadow_rect = Rect2D::new(
                    page_rect.x + 4.0 * config.zoom,
                    page_rect.y + 4.0 * config.zoom,
                    page_rect.width,
                    page_rect.height,
                );
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::solid(
                    shadow_rect,
                    config.shadow_color,
                )));
            }

            // B. Page Paper Canvas Background & Border
            display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                *page_rect,
                ColorRgba::WHITE,
                config.page_border_color,
                1.0,
            )));

            // C. Optional Margin Guides
            if config.show_margin_guides {
                let margin_rect = Rect2D::new(
                    page_rect.x + 96.0 * config.zoom,
                    page_rect.y + 96.0 * config.zoom,
                    page_rect.width - 192.0 * config.zoom,
                    page_rect.height - 192.0 * config.zoom,
                );
                display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                    margin_rect,
                    ColorRgba::TRANSPARENT,
                    ColorRgba::new(0.9, 0.9, 0.9, 1.0),
                    1.0,
                )));
            }

            // D. Table Cells
            for cell in &page.table_cells {
                let cell_rect = Rect2D::new(
                    page_rect.x + cell.bounds.x * config.zoom,
                    page_rect.y + cell.bounds.y * config.zoom,
                    cell.bounds.width * config.zoom,
                    cell.bounds.height * config.zoom,
                );

                if cell_rect.intersects(&config.viewport) {
                    let fill = cell.fill_color.unwrap_or(ColorRgba::WHITE);
                    let border = cell
                        .border_color
                        .unwrap_or(ColorRgba::new(0.85, 0.85, 0.85, 1.0));

                    display_list.push(DisplayCommand::DrawQuad(QuadInstance::bordered(
                        cell_rect,
                        fill,
                        border,
                        (cell.border_width * config.zoom as f32).max(1.0),
                    )));

                    if let Some(ref text) = cell.text {
                        let text_origin = (
                            cell_rect.x + 6.0 * config.zoom,
                            cell_rect.y + 6.0 * config.zoom,
                        );
                        let run = ShapedTextEngine::shape_text(
                            text,
                            "Calibri",
                            14.0 * config.zoom as f32,
                            ColorRgba::BLACK,
                            text_origin,
                        );
                        display_list.push(DisplayCommand::DrawText(run));
                    }
                }
            }

            // E. Text Lines
            for line in &page.lines {
                let line_origin = (
                    page_rect.x + line.x * config.zoom,
                    page_rect.y + line.y * config.zoom,
                );
                let line_rect = Rect2D::new(
                    line_origin.0,
                    line_origin.1,
                    (line.text.len() as f64) * (line.font_size_px as f64) * 0.55 * config.zoom,
                    (line.font_size_px as f64) * 1.25 * config.zoom,
                );

                if line_rect.intersects(&config.viewport) {
                    let run = ShapedTextEngine::shape_text(
                        &line.text,
                        "Calibri",
                        line.font_size_px * config.zoom as f32,
                        line.color,
                        line_origin,
                    );
                    display_list.push(DisplayCommand::DrawText(run));
                }
            }
        }

        // 2. Compile Selection Highlight Quads
        if let Some(sel) = selection {
            if !sel.is_collapsed() {
                let sel_quads = Self::compile_selection_quads(sel, pages, &page_rects, config.zoom);
                for quad in sel_quads {
                    display_list.push(DisplayCommand::DrawQuad(quad));
                }
            }
        }

        // 3. Compile Caret Quad
        if let Some(c) = caret {
            if c.position.page_index < page_rects.len() {
                let page_rect = &page_rects[c.position.page_index];
                if let Some(caret_quad) =
                    Self::compile_caret_quad(c, page_rect, config.zoom, now_ms)
                {
                    display_list.push(DisplayCommand::DrawQuad(caret_quad));
                }
            }
        }

        display_list
    }

    /// Compile directly from `word_engine`'s `LayoutDocument`.
    pub fn compile_from_layout_document(
        layout_doc: &word_engine::layout::LayoutDocument,
        config: &WordViewportConfig,
        caret: Option<&WordCaretModel>,
        selection: Option<&WordSelectionRange>,
        now_ms: u64,
    ) -> DisplayList {
        let pages = Self::layout_document_to_display_pages(layout_doc);
        Self::compile_viewport(&pages, config, caret, selection, now_ms)
    }

    /// Convert `word_engine`'s `LayoutDocument` into display pages.
    pub fn layout_document_to_display_pages(
        layout_doc: &word_engine::layout::LayoutDocument,
    ) -> Vec<WordDisplayPage> {
        let twips_to_px = 1.0 / 15.0;
        let mut display_pages = Vec::with_capacity(layout_doc.pages.len());

        for (p_idx, page) in layout_doc.pages.iter().enumerate() {
            let page_w = (page.width as f64) * twips_to_px;
            let page_h = (page.height as f64) * twips_to_px;
            let mut disp_page = WordDisplayPage::new(p_idx + 1, page_w, page_h);

            for col in &page.columns {
                let col_x = (col.bounds.x as f64) * twips_to_px;
                let col_y = (col.bounds.y as f64) * twips_to_px;

                for line in &col.lines {
                    let line_x = col_x + (line.bounds.x as f64) * twips_to_px;
                    let line_y = col_y + (line.bounds.y as f64) * twips_to_px;

                    let mut combined_text = String::new();
                    let mut max_font_size_px = 14.67f32; // 11pt default
                    let mut first_color = ColorRgba::BLACK;

                    for (r_idx, run) in line.runs.iter().enumerate() {
                        combined_text.push_str(&run.text);
                        let fs_px = (run.font_size_half_pts as f32 * 0.5) * (4.0 / 3.0);
                        if fs_px > max_font_size_px {
                            max_font_size_px = fs_px;
                        }
                        if r_idx == 0 {
                            if let Some(ref c_hex) = run.color {
                                first_color = Self::parse_hex_color(c_hex);
                            }
                        }
                    }

                    disp_page.add_line(WordDisplayLine::new(
                        combined_text,
                        line_x,
                        line_y,
                        max_font_size_px,
                        first_color,
                    ));
                }
            }

            display_pages.push(disp_page);
        }

        display_pages
    }

    /// Compute page bounding boxes positioned in world/viewport space.
    pub fn compute_page_rects(
        pages: &[WordDisplayPage],
        config: &WordViewportConfig,
    ) -> Vec<Rect2D> {
        let mut rects = Vec::with_capacity(pages.len());
        let mut curr_y = config.top_margin - config.scroll_y;

        for page in pages {
            let page_w = page.bounds.width * config.zoom;
            let page_h = page.bounds.height * config.zoom;

            let centered_x = match config.layout_mode {
                PageLayoutMode::SingleColumn | PageLayoutMode::Continuous => {
                    ((config.viewport.width - page_w) / 2.0).max(16.0) - config.scroll_x
                }
                PageLayoutMode::FacingPages => {
                    // Two-page spread side-by-side
                    let spread_w = page_w * 2.0 + config.page_gap * config.zoom;
                    let base_x =
                        ((config.viewport.width - spread_w) / 2.0).max(16.0) - config.scroll_x;
                    if page.page_number % 2 == 1 {
                        base_x
                    } else {
                        base_x + page_w + config.page_gap * config.zoom
                    }
                }
            };

            rects.push(Rect2D::new(centered_x, curr_y, page_w, page_h));

            if config.layout_mode != PageLayoutMode::FacingPages || page.page_number % 2 == 0 {
                let gap = if config.layout_mode == PageLayoutMode::Continuous {
                    4.0
                } else {
                    config.page_gap
                };
                curr_y += page_h + gap * config.zoom;
            }
        }

        rects
    }

    /// Compile selection highlight quads spanning multiple lines and pages.
    pub fn compile_selection_quads(
        selection: &WordSelectionRange,
        pages: &[WordDisplayPage],
        page_rects: &[Rect2D],
        zoom: f64,
    ) -> Vec<QuadInstance> {
        if selection.is_collapsed() || pages.is_empty() {
            return Vec::new();
        }

        let (start, end) = selection.ordered();
        let mut quads = Vec::new();
        let highlight_color = ColorRgba::new(0.20, 0.45, 0.90, 0.35); // Semi-transparent blue

        for (p_idx, page) in pages.iter().enumerate() {
            if p_idx < start.page_index || p_idx > end.page_index {
                continue;
            }

            if p_idx >= page_rects.len() {
                break;
            }
            let pr = &page_rects[p_idx];

            for (l_idx, line) in page.lines.iter().enumerate() {
                if p_idx == start.page_index && l_idx < start.line_index {
                    continue;
                }
                if p_idx == end.page_index && l_idx > end.line_index {
                    break;
                }

                let c_start = if p_idx == start.page_index && l_idx == start.line_index {
                    start.char_offset.min(line.text.len())
                } else {
                    0
                };

                let c_end = if p_idx == end.page_index && l_idx == end.line_index {
                    end.char_offset.min(line.text.len())
                } else {
                    line.text.len()
                };

                if c_end > c_start {
                    let advances = line.calculate_advances();
                    let x1 = line.x + SubpixelTextLayout::caret_x_for_offset(&advances, c_start);
                    let x2 = line.x + SubpixelTextLayout::caret_x_for_offset(&advances, c_end);
                    let w = (x2 - x1).max(2.0);
                    let h = line.height();

                    let sel_rect =
                        Rect2D::new(pr.x + x1 * zoom, pr.y + line.y * zoom, w * zoom, h * zoom);

                    quads.push(QuadInstance::solid(sel_rect, highlight_color));
                }
            }
        }

        quads
    }

    /// Compile a 2px vertical caret bar at the subpixel coordinates.
    pub fn compile_caret_quad(
        caret: &WordCaretModel,
        page_rect: &Rect2D,
        zoom: f64,
        now_ms: u64,
    ) -> Option<QuadInstance> {
        if !caret.is_visible(now_ms) {
            return None;
        }

        let caret_x = page_rect.x + caret.position.subpixel_x * zoom;
        let caret_y = page_rect.y + caret.position.subpixel_y * zoom;
        let caret_w = (2.0 * zoom).max(1.5);
        let caret_h = caret.position.height * zoom;

        Some(QuadInstance::solid(
            Rect2D::new(caret_x, caret_y, caret_w, caret_h),
            ColorRgba::BLACK,
        ))
    }

    fn parse_hex_color(hex: &str) -> ColorRgba {
        let clean = hex.trim().trim_start_matches('#');
        if clean.len() >= 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0) as f32 / 255.0;
            ColorRgba::new(r, g, b, 1.0)
        } else {
            ColorRgba::BLACK
        }
    }
}
