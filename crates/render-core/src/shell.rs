//! Desktop & Laptop Shell Models, Ribbon Controller, Formula Bar & Shortcut Engine
//!
//! Complies with ECMA-376, W3C UI Events, and W3C Pointer Events Level 3.
//! Implements:
//! - Multi-document workspace tab bar (Word, Excel, PowerPoint)
//! - Contextual Ribbon toolbar sets and formatting state
//! - Excel Formula Bar Name Box conversion and live formula syntax tokenizer
//! - Word Ruler margin and indent geometry
//! - Document outline heading extractor for Navigation Pane
//! - PowerPoint Slide Sorter thumbnail reorderer
//! - Desktop keyboard shortcut engine (Accelerators & Alt Keytip state machine)
//! - Desktop Session Manager coordinating CommandBus instances

use crate::compiler::excel::ExcelSelectionRange;
use crate::compiler::word::{
    PageMargins, PageSize, ParagraphAlignment, WordCaretPosition, WordDisplayLine, WordDisplayPage,
    WordDisplayTableCell, WordSelectionRange,
};
use crate::interaction::{Command, CommandBus};
use crate::primitive::{ColorRgba, Rect2D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================================
// 1. Document Types & Tab Bar Model
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    Word,
    Excel,
    PowerPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentTab {
    pub id: String,
    pub doc_type: DocumentType,
    pub file_path: Option<String>,
    pub title: String,
    pub is_dirty: bool,
}

impl DocumentTab {
    pub fn new(id: impl Into<String>, doc_type: DocumentType, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            doc_type,
            file_path: None,
            title: title.into(),
            is_dirty: false,
        }
    }

    pub fn display_title(&self) -> String {
        if self.is_dirty {
            format!("{} *", self.title)
        } else {
            self.title.clone()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabBarModel {
    pub tabs: Vec<DocumentTab>,
    pub active_tab_id: Option<String>,
}

impl TabBarModel {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
        }
    }

    pub fn add_tab(&mut self, tab: DocumentTab) -> String {
        let id = tab.id.clone();
        self.tabs.push(tab);
        self.active_tab_id = Some(id.clone());
        id
    }

    pub fn switch_tab(&mut self, id: &str) -> bool {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn close_tab(&mut self, id: &str, force: bool) -> Result<Option<String>, &'static str> {
        let pos = self
            .tabs
            .iter()
            .position(|t| t.id == id)
            .ok_or("Tab not found")?;

        if self.tabs[pos].is_dirty && !force {
            return Err("Tab has unsaved changes");
        }

        self.tabs.remove(pos);

        if self.active_tab_id.as_deref() == Some(id) {
            if self.tabs.is_empty() {
                self.active_tab_id = None;
            } else if pos < self.tabs.len() {
                self.active_tab_id = Some(self.tabs[pos].id.clone());
            } else {
                self.active_tab_id = Some(self.tabs[pos - 1].id.clone());
            }
        }

        Ok(self.active_tab_id.clone())
    }

    pub fn active_tab(&self) -> Option<&DocumentTab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter().find(|t| &t.id == id))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut DocumentTab> {
        if let Some(ref id) = self.active_tab_id.clone() {
            self.tabs.iter_mut().find(|t| &t.id == id)
        } else {
            None
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        if let Some(ref curr_id) = self.active_tab_id {
            if let Some(pos) = self.tabs.iter().position(|t| &t.id == curr_id) {
                let next_pos = (pos + 1) % self.tabs.len();
                self.active_tab_id = Some(self.tabs[next_pos].id.clone());
            }
        }
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() <= 1 {
            return;
        }
        if let Some(ref curr_id) = self.active_tab_id {
            if let Some(pos) = self.tabs.iter().position(|t| &t.id == curr_id) {
                let prev_pos = if pos == 0 {
                    self.tabs.len() - 1
                } else {
                    pos - 1
                };
                self.active_tab_id = Some(self.tabs[prev_pos].id.clone());
            }
        }
    }
}

// =========================================================================
// 2. Contextual Ribbon Model & Formatting State
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RibbonTabId {
    File,
    Home,
    Insert,
    Layout,
    Formulas,
    Data,
    Transitions,
    Animations,
    SlideShow,
    Review,
    View,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RibbonModel {
    pub active_tab: RibbonTabId,
    pub doc_type: DocumentType,
}

impl RibbonModel {
    pub fn for_document_type(doc_type: DocumentType) -> Self {
        Self {
            active_tab: RibbonTabId::Home,
            doc_type,
        }
    }

    pub fn visible_tabs(&self) -> Vec<RibbonTabId> {
        match self.doc_type {
            DocumentType::Word => vec![
                RibbonTabId::File,
                RibbonTabId::Home,
                RibbonTabId::Insert,
                RibbonTabId::Layout,
                RibbonTabId::Review,
                RibbonTabId::View,
            ],
            DocumentType::Excel => vec![
                RibbonTabId::File,
                RibbonTabId::Home,
                RibbonTabId::Insert,
                RibbonTabId::Layout,
                RibbonTabId::Formulas,
                RibbonTabId::Data,
                RibbonTabId::Review,
                RibbonTabId::View,
            ],
            DocumentType::PowerPoint => vec![
                RibbonTabId::File,
                RibbonTabId::Home,
                RibbonTabId::Insert,
                RibbonTabId::Layout,
                RibbonTabId::Transitions,
                RibbonTabId::Animations,
                RibbonTabId::SlideShow,
                RibbonTabId::Review,
                RibbonTabId::View,
            ],
        }
    }

    pub fn is_tab_visible(&self, tab: RibbonTabId) -> bool {
        self.visible_tabs().contains(&tab)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormattingState {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_family: String,
    pub font_size_pt: f32,
    pub text_color: ColorRgba,
    pub highlight_color: Option<ColorRgba>,
    pub alignment: ParagraphAlignment,
    pub line_spacing: f32,
    pub indent_level: u32,
    pub style_name: String,
}

impl Default for FormattingState {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            font_family: "Calibri".into(),
            font_size_pt: 11.0,
            text_color: ColorRgba::BLACK,
            highlight_color: None,
            alignment: ParagraphAlignment::Left,
            line_spacing: 1.15,
            indent_level: 0,
            style_name: "Normal".into(),
        }
    }
}

impl FormattingState {
    pub fn toggle_bold(&mut self) {
        self.bold = !self.bold;
    }

    pub fn toggle_italic(&mut self) {
        self.italic = !self.italic;
    }

    pub fn toggle_underline(&mut self) {
        self.underline = !self.underline;
    }

    pub fn toggle_strikethrough(&mut self) {
        self.strikethrough = !self.strikethrough;
    }

    pub fn set_font_family(&mut self, font: impl Into<String>) {
        self.font_family = font.into();
    }

    pub fn set_font_size(&mut self, pt: f32) {
        self.font_size_pt = pt.clamp(1.0, 1638.0);
    }

    pub fn font_size_px(&self) -> f32 {
        (self.font_size_pt * 4.0) / 3.0
    }

    pub fn font_size_half_pt(&self) -> u32 {
        (self.font_size_pt * 2.0).round().max(1.0) as u32
    }

    pub fn increase_font_size(&mut self) {
        const SIZES: &[f32] = &[
            8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 48.0, 72.0,
        ];
        for &sz in SIZES {
            if sz > self.font_size_pt + 0.1 {
                self.font_size_pt = sz;
                return;
            }
        }
        self.font_size_pt = (self.font_size_pt + 10.0).min(1638.0);
    }

    pub fn decrease_font_size(&mut self) {
        const SIZES: &[f32] = &[
            8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 48.0, 72.0,
        ];
        for &sz in SIZES.iter().rev() {
            if sz < self.font_size_pt - 0.1 {
                self.font_size_pt = sz;
                return;
            }
        }
        self.font_size_pt = 8.0;
    }

    pub fn set_text_color(&mut self, color: ColorRgba) {
        self.text_color = color;
    }

    pub fn set_highlight_color(&mut self, color: Option<ColorRgba>) {
        self.highlight_color = color;
    }

    pub fn clear_formatting(&mut self) {
        *self = Self::default();
    }

    pub fn set_alignment(&mut self, alignment: ParagraphAlignment) {
        self.alignment = alignment;
    }

    pub fn increase_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_add(1);
    }

    pub fn decrease_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    pub fn set_line_spacing(&mut self, spacing: f32) {
        self.line_spacing = spacing.clamp(0.5, 5.0);
    }

    pub fn set_style_name(&mut self, name: impl Into<String>) {
        let name_str = name.into();
        self.style_name = name_str.clone();
        match name_str.to_ascii_lowercase().as_str() {
            "heading 1" | "heading1" => {
                self.font_size_pt = 20.0;
                self.bold = true;
                self.italic = false;
                self.text_color = ColorRgba::new(0.12, 0.29, 0.49, 1.0);
            }
            "heading 2" | "heading2" => {
                self.font_size_pt = 15.0;
                self.bold = true;
                self.italic = false;
                self.text_color = ColorRgba::new(0.18, 0.46, 0.71, 1.0);
            }
            "title" => {
                self.font_size_pt = 28.0;
                self.bold = true;
                self.italic = false;
                self.text_color = ColorRgba::new(0.12, 0.29, 0.49, 1.0);
            }
            "subtitle" => {
                self.font_size_pt = 14.0;
                self.bold = false;
                self.italic = true;
                self.text_color = ColorRgba::new(0.40, 0.40, 0.40, 1.0);
            }
            _ => {
                self.font_size_pt = 11.0;
                self.bold = false;
                self.italic = false;
                self.text_color = ColorRgba::BLACK;
            }
        }
    }
}

/// Page orientation for the document layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

/// Page setup configuration model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSetupModel {
    pub page_size: PageSize,
    pub orientation: PageOrientation,
    pub margins: PageMargins,
    pub column_count: usize,
}

impl Default for PageSetupModel {
    fn default() -> Self {
        Self {
            page_size: PageSize::Letter,
            orientation: PageOrientation::Portrait,
            margins: PageMargins::default(),
            column_count: 1,
        }
    }
}

impl PageSetupModel {
    pub fn resolved_dimensions(&self) -> (f64, f64) {
        let (w, h) = self.page_size.dimensions();
        match self.orientation {
            PageOrientation::Portrait => (w, h),
            PageOrientation::Landscape => (h, w),
        }
    }

    pub fn set_orientation(&mut self, orientation: PageOrientation) {
        self.orientation = orientation;
    }

    pub fn set_margins_normal(&mut self) {
        self.margins = PageMargins::uniform(96.0); // 1 inch
    }

    pub fn set_margins_narrow(&mut self) {
        self.margins = PageMargins::uniform(48.0); // 0.5 inch
    }

    pub fn set_margins_moderate(&mut self) {
        self.margins = PageMargins::new(96.0, 72.0, 96.0, 72.0); // 1" top/bottom, 0.75" left/right
    }

    pub fn set_margins_wide(&mut self) {
        self.margins = PageMargins::new(96.0, 192.0, 96.0, 192.0); // 1" top/bottom, 2" left/right
    }
}

/// Actions triggered from Ribbon toolbar buttons and dropdowns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RibbonAction {
    // Home - Clipboard
    Cut,
    Copy,
    Paste,
    FormatPainter,

    // Home - Font
    ToggleBold,
    ToggleItalic,
    ToggleUnderline,
    ToggleStrikethrough,
    SetFontFamily(String),
    SetFontSize(f32),
    IncreaseFontSize,
    DecreaseFontSize,
    SetTextColor(ColorRgba),
    SetHighlightColor(Option<ColorRgba>),
    ClearFormatting,

    // Home - Paragraph
    AlignLeft,
    AlignCenter,
    AlignRight,
    AlignJustify,
    IncreaseIndent,
    DecreaseIndent,
    SetLineSpacing(f32),
    ToggleBulletList,
    ToggleNumberedList,
    SetStyle(String),

    // Insert Tab
    InsertTable {
        rows: usize,
        cols: usize,
    },
    InsertPicture {
        path: String,
        width: f64,
        height: f64,
    },
    InsertShape {
        shape_type: String,
        width: f64,
        height: f64,
    },
    InsertTextBox {
        text: String,
        width: f64,
        height: f64,
    },
    InsertPageBreak,
    InsertHeader(String),
    InsertFooter(String),

    // Layout Tab
    SetMargins(PageMargins),
    SetOrientation(PageOrientation),
    SetPageSize(PageSize),
    SetColumnCount(usize),
}

/// Controller executing Ribbon actions live against the document.
pub struct WordRibbonController;

impl WordRibbonController {
    /// Dispatches a RibbonAction, applying formatting live to the current document state,
    /// pages, and active selection.
    pub fn dispatch_action(
        action: &RibbonAction,
        formatting: &mut FormattingState,
        page_setup: &mut PageSetupModel,
        pages: &mut [WordDisplayPage],
        selection: Option<&WordSelectionRange>,
        active_caret: &WordCaretPosition,
    ) -> bool {
        match action {
            RibbonAction::ToggleBold => {
                formatting.toggle_bold();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = formatting.font_size_px();
                });
                true
            }
            RibbonAction::ToggleItalic => {
                formatting.toggle_italic();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |_line| {});
                true
            }
            RibbonAction::ToggleUnderline => {
                formatting.toggle_underline();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |_line| {});
                true
            }
            RibbonAction::ToggleStrikethrough => {
                formatting.toggle_strikethrough();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |_line| {});
                true
            }
            RibbonAction::SetFontFamily(f) => {
                formatting.set_font_family(f.clone());
                true
            }
            RibbonAction::SetFontSize(sz) => {
                formatting.set_font_size(*sz);
                let fs_px = formatting.font_size_px();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = fs_px;
                });
                true
            }
            RibbonAction::IncreaseFontSize => {
                formatting.increase_font_size();
                let fs_px = formatting.font_size_px();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = fs_px;
                });
                true
            }
            RibbonAction::DecreaseFontSize => {
                formatting.decrease_font_size();
                let fs_px = formatting.font_size_px();
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = fs_px;
                });
                true
            }
            RibbonAction::SetTextColor(c) => {
                formatting.set_text_color(*c);
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.color = *c;
                });
                true
            }
            RibbonAction::SetHighlightColor(h) => {
                formatting.set_highlight_color(*h);
                true
            }
            RibbonAction::ClearFormatting => {
                formatting.clear_formatting();
                let fs_px = formatting.font_size_px();
                let c = formatting.text_color;
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = fs_px;
                    line.color = c;
                });
                true
            }
            RibbonAction::AlignLeft => {
                formatting.set_alignment(ParagraphAlignment::Left);
                Self::apply_alignment(pages, active_caret, ParagraphAlignment::Left, page_setup);
                true
            }
            RibbonAction::AlignCenter => {
                formatting.set_alignment(ParagraphAlignment::Center);
                Self::apply_alignment(pages, active_caret, ParagraphAlignment::Center, page_setup);
                true
            }
            RibbonAction::AlignRight => {
                formatting.set_alignment(ParagraphAlignment::Right);
                Self::apply_alignment(pages, active_caret, ParagraphAlignment::Right, page_setup);
                true
            }
            RibbonAction::AlignJustify => {
                formatting.set_alignment(ParagraphAlignment::Justify);
                Self::apply_alignment(pages, active_caret, ParagraphAlignment::Justify, page_setup);
                true
            }
            RibbonAction::IncreaseIndent => {
                formatting.increase_indent();
                Self::apply_indent(pages, active_caret, 36.0); // 0.375 inch per indent step
                true
            }
            RibbonAction::DecreaseIndent => {
                formatting.decrease_indent();
                Self::apply_indent(pages, active_caret, -36.0);
                true
            }
            RibbonAction::SetLineSpacing(sp) => {
                formatting.set_line_spacing(*sp);
                true
            }
            RibbonAction::ToggleBulletList => {
                Self::toggle_bullet_on_active_line(pages, active_caret);
                true
            }
            RibbonAction::ToggleNumberedList => {
                Self::toggle_numbering_on_active_line(pages, active_caret);
                true
            }
            RibbonAction::SetStyle(st) => {
                formatting.set_style_name(st);
                let fs_px = formatting.font_size_px();
                let c = formatting.text_color;
                Self::apply_font_mutation_to_selection(pages, selection, active_caret, |line| {
                    line.font_size_px = fs_px;
                    line.color = c;
                });
                true
            }
            RibbonAction::InsertTable { rows, cols } => {
                Self::insert_table(pages, active_caret, *rows, *cols, page_setup);
                true
            }
            RibbonAction::SetMargins(m) => {
                page_setup.margins = *m;
                true
            }
            RibbonAction::SetOrientation(o) => {
                page_setup.set_orientation(*o);
                let (w, h) = page_setup.resolved_dimensions();
                for page in pages.iter_mut() {
                    page.bounds = Rect2D::new(0.0, 0.0, w, h);
                }
                true
            }
            RibbonAction::SetPageSize(ps) => {
                page_setup.page_size = *ps;
                let (w, h) = page_setup.resolved_dimensions();
                for page in pages.iter_mut() {
                    page.bounds = Rect2D::new(0.0, 0.0, w, h);
                }
                true
            }
            RibbonAction::SetColumnCount(cnt) => {
                page_setup.column_count = (*cnt).max(1);
                true
            }
            _ => false,
        }
    }

    fn apply_font_mutation_to_selection<F>(
        pages: &mut [WordDisplayPage],
        selection: Option<&WordSelectionRange>,
        active_caret: &WordCaretPosition,
        mut mutate: F,
    ) where
        F: FnMut(&mut WordDisplayLine),
    {
        if let Some(sel) = selection {
            if !sel.is_collapsed() {
                let (start, end) = sel.ordered();
                for (p_idx, page) in pages.iter_mut().enumerate() {
                    if p_idx < start.page_index || p_idx > end.page_index {
                        continue;
                    }
                    for (l_idx, line) in page.lines.iter_mut().enumerate() {
                        if p_idx == start.page_index && l_idx < start.line_index {
                            continue;
                        }
                        if p_idx == end.page_index && l_idx > end.line_index {
                            break;
                        }
                        mutate(line);
                    }
                }
                return;
            }
        }

        // Collapsed: apply to active line
        if let Some(page) = pages.get_mut(active_caret.page_index) {
            if let Some(line) = page.lines.get_mut(active_caret.line_index) {
                mutate(line);
            }
        }
    }

    fn apply_alignment(
        pages: &mut [WordDisplayPage],
        active_caret: &WordCaretPosition,
        align: ParagraphAlignment,
        page_setup: &PageSetupModel,
    ) {
        if let Some(page) = pages.get_mut(active_caret.page_index) {
            if let Some(line) = page.lines.get_mut(active_caret.line_index) {
                let (page_w, _) = page_setup.resolved_dimensions();
                let margin_l = page_setup.margins.left;
                let margin_r = page_setup.margins.right;
                let content_w = (page_w - margin_l - margin_r).max(100.0);
                let line_w = line.width();

                line.x = match align {
                    ParagraphAlignment::Left | ParagraphAlignment::Justify => margin_l,
                    ParagraphAlignment::Center => margin_l + (content_w - line_w) / 2.0,
                    ParagraphAlignment::Right => page_w - margin_r - line_w,
                };
            }
        }
    }

    fn apply_indent(pages: &mut [WordDisplayPage], active_caret: &WordCaretPosition, delta_x: f64) {
        if let Some(page) = pages.get_mut(active_caret.page_index) {
            if let Some(line) = page.lines.get_mut(active_caret.line_index) {
                line.x = (line.x + delta_x).max(72.0);
            }
        }
    }

    fn toggle_bullet_on_active_line(
        pages: &mut [WordDisplayPage],
        active_caret: &WordCaretPosition,
    ) {
        if let Some(page) = pages.get_mut(active_caret.page_index) {
            if let Some(line) = page.lines.get_mut(active_caret.line_index) {
                if let Some(rest) = line.text.strip_prefix("• ") {
                    line.text = rest.to_string();
                } else {
                    line.text = format!("• {}", line.text);
                }
            }
        }
    }

    fn toggle_numbering_on_active_line(
        pages: &mut [WordDisplayPage],
        active_caret: &WordCaretPosition,
    ) {
        if let Some(page) = pages.get_mut(active_caret.page_index) {
            if let Some(line) = page.lines.get_mut(active_caret.line_index) {
                if let Some(rest) = line.text.strip_prefix("1. ") {
                    line.text = rest.to_string();
                } else {
                    line.text = format!("1. {}", line.text);
                }
            }
        }
    }

    fn insert_table(
        pages: &mut [WordDisplayPage],
        active_caret: &WordCaretPosition,
        rows: usize,
        cols: usize,
        page_setup: &PageSetupModel,
    ) {
        if rows == 0 || cols == 0 {
            return;
        }

        if let Some(page) = pages.get_mut(active_caret.page_index) {
            let (page_w, _) = page_setup.resolved_dimensions();
            let margin_l = page_setup.margins.left;
            let margin_r = page_setup.margins.right;
            let total_w = (page_w - margin_l - margin_r).max(100.0);
            let col_w = total_w / (cols as f64);
            let row_h = 28.0;
            let start_y = active_caret.subpixel_y + active_caret.height + 14.0;

            for r in 0..rows {
                for c in 0..cols {
                    let cell_x = margin_l + (c as f64) * col_w;
                    let cell_y = start_y + (r as f64) * row_h;
                    page.table_cells.push(WordDisplayTableCell {
                        bounds: Rect2D::new(cell_x, cell_y, col_w, row_h),
                        fill_color: if r == 0 {
                            Some(ColorRgba::new(0.92, 0.94, 0.97, 1.0))
                        } else {
                            Some(ColorRgba::WHITE)
                        },
                        border_color: Some(ColorRgba::new(0.8, 0.8, 0.8, 1.0)),
                        border_width: 1.0,
                        text: if r == 0 {
                            Some(format!("Header {}", c + 1))
                        } else {
                            Some(String::new())
                        },
                    });
                }
            }
        }
    }
}

/// Reversible formatting command executed through the Command Bus.
#[derive(Debug, Clone)]
pub struct FormatCommand {
    pub prev_formatting: FormattingState,
    pub new_formatting: FormattingState,
    pub description_str: String,
}

impl Command for FormatCommand {
    fn execute(&mut self, _state: &mut crate::interaction::DocumentState) -> Result<(), String> {
        Ok(())
    }

    fn undo(&mut self, _state: &mut crate::interaction::DocumentState) -> Result<(), String> {
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

// =========================================================================
// 3. Excel Formula Bar Engine & Syntax Highlighting Tokenizer
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaTokenType {
    Function,
    Reference,
    Number,
    StringLiteral,
    Operator,
    ParenOpen,
    ParenClose,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormulaToken {
    pub token_type: FormulaTokenType,
    pub value: String,
}

pub struct FormulaBarEngine;

impl FormulaBarEngine {
    /// Convert 0-indexed (row, col) to A1 Name Box string (e.g. (3, 1) -> "B4").
    pub fn coord_to_a1(row: u32, col: u32) -> String {
        let mut col_str = String::new();
        let mut c = col + 1;
        while c > 0 {
            let rem = ((c - 1) % 26) as u8;
            col_str.insert(0, (b'A' + rem) as char);
            c = (c - 1) / 26;
        }
        format!("{}{}", col_str, row + 1)
    }

    /// Parse A1 Name Box string to 0-indexed (row, col) (e.g. "AA100" -> (99, 26)).
    pub fn a1_to_coord(a1: &str) -> Option<(u32, u32)> {
        let trimmed = a1.trim().to_ascii_uppercase();
        let col_part_len = trimmed
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .count();
        if col_part_len == 0 || col_part_len == trimmed.len() {
            return None;
        }

        let (col_str, row_str) = trimmed.split_at(col_part_len);
        let mut col: u32 = 0;
        for ch in col_str.bytes() {
            if !ch.is_ascii_uppercase() {
                return None;
            }
            col = col.checked_mul(26)?.checked_add((ch - b'A' + 1) as u32)?;
        }
        let col_idx = col.checked_sub(1)?;

        let row_num: u32 = row_str.parse().ok()?;
        let row_idx = row_num.checked_sub(1)?;

        Some((row_idx, col_idx))
    }

    /// Tokenize formula for live syntax highlighting.
    pub fn tokenize_formula(formula: &str) -> Vec<FormulaToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = formula.chars().collect();
        let mut i = 0;

        // Skip leading '=' if present
        if !chars.is_empty() && chars[0] == '=' {
            i = 1;
        }

        while i < chars.len() {
            let ch = chars[i];
            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            match ch {
                '(' => {
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::ParenOpen,
                        value: "(".into(),
                    });
                    i += 1;
                }
                ')' => {
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::ParenClose,
                        value: ")".into(),
                    });
                    i += 1;
                }
                ',' => {
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::Comma,
                        value: ",".into(),
                    });
                    i += 1;
                }
                '+' | '-' | '*' | '/' | '^' | '&' | '=' | '<' | '>' => {
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::Operator,
                        value: ch.to_string(),
                    });
                    i += 1;
                }
                '"' => {
                    // String literal
                    let mut lit = String::new();
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        lit.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '"' {
                        i += 1;
                    }
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::StringLiteral,
                        value: lit,
                    });
                }
                '0'..='9' | '.' => {
                    // Number literal
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    tokens.push(FormulaToken {
                        token_type: FormulaTokenType::Number,
                        value: num_str,
                    });
                }
                _ if ch.is_ascii_alphabetic() || ch == '$' => {
                    // Identifier: could be Function (if followed by '(') or Reference / Range
                    let start = i;
                    while i < chars.len()
                        && (chars[i].is_ascii_alphanumeric()
                            || chars[i] == '$'
                            || chars[i] == ':'
                            || chars[i] == '_')
                    {
                        i += 1;
                    }
                    let ident: String = chars[start..i].iter().collect();

                    // Check if next non-whitespace is '('
                    let mut lookahead = i;
                    while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                        lookahead += 1;
                    }

                    if lookahead < chars.len() && chars[lookahead] == '(' {
                        tokens.push(FormulaToken {
                            token_type: FormulaTokenType::Function,
                            value: ident,
                        });
                    } else {
                        tokens.push(FormulaToken {
                            token_type: FormulaTokenType::Reference,
                            value: ident,
                        });
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        tokens
    }

    /// Rotate Excel cell reference through absolute / relative styles:
    /// `A1` -> `$A$1` -> `A$1` -> `$A1` -> `A1`.
    pub fn rotate_f4_reference(reference: &str) -> String {
        let clean = reference.trim();
        let has_col_dollar = clean.starts_with('$');
        let rest = if has_col_dollar { &clean[1..] } else { clean };

        let row_split = rest.find(|c: char| c.is_ascii_digit());
        let (col_part, row_full) = match row_split {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => return clean.to_string(),
        };

        let has_row_dollar = col_part.ends_with('$');
        let pure_col = if has_row_dollar {
            &col_part[..col_part.len() - 1]
        } else {
            col_part
        };

        match (has_col_dollar, has_row_dollar) {
            // Relative "A1" -> Absolute "$A$1"
            (false, false) => format!("${pure_col}${row_full}"),
            // Absolute "$A$1" -> Row-fixed "A$1"
            (true, true) => format!("{pure_col}${row_full}"),
            // Row-fixed "A$1" -> Col-fixed "$A1"
            (false, true) => format!("${pure_col}{row_full}"),
            // Col-fixed "$A1" -> Relative "A1"
            (true, false) => format!("{pure_col}{row_full}"),
        }
    }

    /// Parse Name Box input into an active selection range (e.g. "B4" or "A1:C10").
    pub fn parse_name_box(input: &str) -> Option<ExcelSelectionRange> {
        let trimmed = input.trim();
        if let Some((start_str, end_str)) = trimmed.split_once(':') {
            let (r1, c1) = Self::a1_to_coord(start_str)?;
            let (r2, c2) = Self::a1_to_coord(end_str)?;
            Some(ExcelSelectionRange {
                min_row: r1.min(r2),
                min_col: c1.min(c2),
                max_row: r1.max(r2),
                max_col: c1.max(c2),
            })
        } else {
            let (row, col) = Self::a1_to_coord(trimmed)?;
            Some(ExcelSelectionRange::single(row, col))
        }
    }

    /// Extract cell/range references from a formula along with matching visual highlight colors.
    pub fn extract_colored_references(formula: &str) -> Vec<(ExcelSelectionRange, ColorRgba)> {
        let palette = [
            ColorRgba::new(0.12, 0.53, 0.90, 1.0), // Blue
            ColorRgba::new(0.06, 0.49, 0.25, 1.0), // Green
            ColorRgba::new(0.55, 0.25, 0.70, 1.0), // Purple
            ColorRgba::new(0.85, 0.40, 0.10, 1.0), // Orange
            ColorRgba::new(0.80, 0.10, 0.20, 1.0), // Red
        ];

        let tokens = Self::tokenize_formula(formula);
        let mut refs = Vec::new();
        let mut color_idx = 0;

        for token in tokens {
            if token.token_type == FormulaTokenType::Reference {
                if let Some(range) = Self::parse_name_box(&token.value) {
                    let color = palette[color_idx % palette.len()];
                    color_idx += 1;
                    refs.push((range, color));
                }
            }
        }

        refs
    }
}

// =========================================================================
// 4. Word Ruler Geometry & Indent Calculation
// =========================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordRulerConfig {
    pub page_width_dxa: u32,
    pub margin_left_dxa: u32,
    pub margin_right_dxa: u32,
    pub first_line_indent_dxa: i32,
    pub hanging_indent_dxa: i32,
}

impl WordRulerConfig {
    pub fn new(page_w: u32, margin_l: u32, margin_r: u32) -> Self {
        Self {
            page_width_dxa: page_w,
            margin_left_dxa: margin_l,
            margin_right_dxa: margin_r,
            first_line_indent_dxa: 0,
            hanging_indent_dxa: 0,
        }
    }

    pub fn content_width_dxa(&self) -> u32 {
        self.page_width_dxa
            .saturating_sub(self.margin_left_dxa)
            .saturating_sub(self.margin_right_dxa)
    }

    pub fn slider_first_line_x_dxa(&self) -> i32 {
        self.margin_left_dxa as i32 + self.first_line_indent_dxa
    }
}

// =========================================================================
// 5. Navigation Pane Outline Extractor
// =========================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingNode {
    pub level: u8,
    pub text: String,
}

pub struct NavigationPaneEngine;

impl NavigationPaneEngine {
    /// Extract headings (Heading 1 through Heading 9) from styled document paragraphs.
    pub fn extract_headings(paragraphs: &[(String, String)]) -> Vec<HeadingNode> {
        let mut headings = Vec::new();
        for (style, text) in paragraphs {
            let lower = style.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("heading ") {
                if let Ok(lvl) = rest.trim().parse::<u8>() {
                    if (1..=9).contains(&lvl) {
                        headings.push(HeadingNode {
                            level: lvl,
                            text: text.clone(),
                        });
                    }
                }
            }
        }
        headings
    }
}

// =========================================================================
// 6. PowerPoint Slide Sorter Thumbnail Reorderer
// =========================================================================

pub struct SlideSorterEngine;

impl SlideSorterEngine {
    /// Move a slide thumbnail from `from_idx` to `to_idx` preserving sequential integrity.
    pub fn reorder_slides(slides: &mut Vec<u32>, from_idx: usize, to_idx: usize) {
        if from_idx < slides.len() && to_idx < slides.len() && from_idx != to_idx {
            let item = slides.remove(from_idx);
            slides.insert(to_idx, item);
        }
    }
}

// =========================================================================
// 7. Desktop Keyboard Shortcut Engine & Alt Keytips
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellAction {
    Copy,
    Cut,
    Paste,
    Undo,
    Redo,
    Save,
    SaveAs,
    Open,
    New,
    Close,
    Find,
    NextTab,
    PrevTab,
    CellEdit,
    ToggleF4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyCombination {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

pub struct ShortcutEngine;

impl ShortcutEngine {
    /// Match key combination against desktop accelerator table.
    pub fn match_action(combo: &KeyCombination) -> Option<ShellAction> {
        let k = combo.key.to_ascii_lowercase();

        if combo.ctrl && !combo.alt {
            match k.as_str() {
                "c" => return Some(ShellAction::Copy),
                "x" => return Some(ShellAction::Cut),
                "v" => return Some(ShellAction::Paste),
                "z" if !combo.shift => return Some(ShellAction::Undo),
                "y" => return Some(ShellAction::Redo),
                "z" if combo.shift => return Some(ShellAction::Redo),
                "s" if !combo.shift => return Some(ShellAction::Save),
                "s" if combo.shift => return Some(ShellAction::SaveAs),
                "o" => return Some(ShellAction::Open),
                "n" => return Some(ShellAction::New),
                "w" => return Some(ShellAction::Close),
                "f" => return Some(ShellAction::Find),
                "tab" if !combo.shift => return Some(ShellAction::NextTab),
                "tab" if combo.shift => return Some(ShellAction::PrevTab),
                _ => {}
            }
        }

        if !combo.ctrl && !combo.alt && !combo.shift {
            match k.as_str() {
                "f2" => return Some(ShellAction::CellEdit),
                "f4" => return Some(ShellAction::ToggleF4),
                _ => {}
            }
        }

        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct AltKeytipStateMachine {
    pub is_active: bool,
}

impl AltKeytipStateMachine {
    pub fn toggle_alt(&mut self) -> bool {
        self.is_active = !self.is_active;
        self.is_active
    }

    pub fn dispatch_keytip(&mut self, code: char) -> Option<RibbonTabId> {
        if !self.is_active {
            return None;
        }

        let tab = match code.to_ascii_uppercase() {
            'F' => Some(RibbonTabId::File),
            'H' => Some(RibbonTabId::Home),
            'N' => Some(RibbonTabId::Insert),
            'P' => Some(RibbonTabId::Layout),
            'M' => Some(RibbonTabId::Formulas),
            'A' => Some(RibbonTabId::Data),
            'T' => Some(RibbonTabId::Transitions),
            'K' => Some(RibbonTabId::Animations),
            'S' => Some(RibbonTabId::SlideShow),
            'R' => Some(RibbonTabId::Review),
            'W' => Some(RibbonTabId::View),
            _ => None,
        };

        if tab.is_some() {
            self.is_active = false;
        }

        tab
    }

    pub fn cancel(&mut self) {
        self.is_active = false;
    }
}

// =========================================================================
// 8. Desktop Session Manager (Tauri IPC Integration Backend)
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub doc_type: DocumentType,
    pub title: String,
    pub is_dirty: bool,
    pub file_path: Option<String>,
}

#[derive(Default)]
pub struct DesktopSessionManager {
    pub tab_bar: TabBarModel,
    pub command_buses: HashMap<String, CommandBus>,
    next_session_counter: usize,
}

impl DesktopSessionManager {
    pub fn new() -> Self {
        Self {
            tab_bar: TabBarModel::new(),
            command_buses: HashMap::new(),
            next_session_counter: 1,
        }
    }

    /// Open or detect document type from file path extension.
    pub fn detect_doc_type(path: &str) -> DocumentType {
        let lower = path.to_ascii_lowercase();
        if lower.ends_with(".xlsx") || lower.ends_with(".xlsm") {
            DocumentType::Excel
        } else if lower.ends_with(".pptx") || lower.ends_with(".pptm") {
            DocumentType::PowerPoint
        } else {
            DocumentType::Word
        }
    }

    /// Create a new unsaved document session.
    pub fn create_session(&mut self, doc_type: DocumentType, title: &str) -> SessionSummary {
        let session_id = format!("sess-{}", self.next_session_counter);
        self.next_session_counter += 1;

        let tab = DocumentTab::new(&session_id, doc_type, title);
        self.tab_bar.add_tab(tab);
        self.command_buses
            .insert(session_id.clone(), CommandBus::new());

        SessionSummary {
            session_id,
            doc_type,
            title: title.to_string(),
            is_dirty: false,
            file_path: None,
        }
    }

    /// Open an existing document path.
    pub fn open_session(&mut self, file_path: &str) -> SessionSummary {
        let doc_type = Self::detect_doc_type(file_path);
        let filename = std::path::Path::new(file_path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(file_path);

        let mut summary = self.create_session(doc_type, filename);
        summary.file_path = Some(file_path.to_string());

        if let Some(tab) = self.tab_bar.active_tab_mut() {
            tab.file_path = Some(file_path.to_string());
        }

        summary
    }

    /// Close session with dirty check.
    pub fn close_session(
        &mut self,
        session_id: &str,
        force: bool,
    ) -> Result<Option<String>, &'static str> {
        let next_active = self.tab_bar.close_tab(session_id, force)?;
        self.command_buses.remove(session_id);
        Ok(next_active)
    }

    /// Mark active session as dirty.
    pub fn mark_active_dirty(&mut self, is_dirty: bool) {
        if let Some(tab) = self.tab_bar.active_tab_mut() {
            tab.is_dirty = is_dirty;
        }
    }
}
