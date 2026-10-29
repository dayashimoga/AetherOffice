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

use crate::interaction::CommandBus;
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
    pub font_family: String,
    pub font_size_pt: f32,
}

impl Default for FormattingState {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            font_family: "Calibri".into(),
            font_size_pt: 11.0,
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

    pub fn font_size_half_pt(&self) -> u32 {
        (self.font_size_pt * 2.0).round().max(1.0) as u32
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
