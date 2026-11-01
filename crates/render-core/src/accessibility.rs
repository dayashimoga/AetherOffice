//! accessibility.rs
//! GEC-v7.0 Stage 52 (Sub-Phase 52.2.1)
//!
//! Authoritative WCAG 2.1 AA accessibility models, virtualized ARIA Shadow DOM tree,
//! CJK IME composition bridge, Unicode BiDi / RTL analysis, and spatial keyboard navigation map.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Semantic ARIA landmark and widget roles for document and canvas elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AriaRole {
    Document,
    Heading,
    Paragraph,
    Grid,
    Row,
    Cell,
    Shape,
    Button,
    Textbox,
    List,
    ListItem,
    Image,
    Region,
}

impl AriaRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Heading => "heading",
            Self::Paragraph => "paragraph",
            Self::Grid => "grid",
            Self::Row => "row",
            Self::Cell => "gridcell",
            Self::Shape => "img",
            Self::Button => "button",
            Self::Textbox => "textbox",
            Self::List => "list",
            Self::ListItem => "listitem",
            Self::Image => "img",
            Self::Region => "region",
        }
    }
}

/// 2D bounding rectangle in document viewport coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AriaBoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl AriaBoundingBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// Semantic node within the accessibility tree reflecting canvas document structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AriaNode {
    pub id: String,
    pub role: AriaRole,
    pub label: String,
    pub value: Option<String>,
    pub level: Option<u32>,
    pub selected: bool,
    pub checked: Option<bool>,
    pub disabled: bool,
    pub expanded: Option<bool>,
    pub live: Option<String>,
    pub bounds: Option<AriaBoundingBox>,
    pub children: Vec<AriaNode>,
}

impl AriaNode {
    pub fn new(id: impl Into<String>, role: AriaRole, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role,
            label: label.into(),
            value: None,
            level: None,
            selected: false,
            checked: None,
            disabled: false,
            expanded: None,
            live: None,
            bounds: None,
            children: Vec::new(),
        }
    }

    pub fn with_bounds(mut self, bounds: AriaBoundingBox) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn add_child(&mut self, child: AriaNode) {
        self.children.push(child);
    }

    pub fn node_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(AriaNode::node_count)
            .sum::<usize>()
    }

    pub fn find_node(&self, target_id: &str) -> Option<&AriaNode> {
        if self.id == target_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_node(target_id) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_node_at_point(&self, px: f32, py: f32) -> Option<&AriaNode> {
        for child in self.children.iter().rev() {
            if let Some(found) = child.find_node_at_point(px, py) {
                return Some(found);
            }
        }
        if let Some(bounds) = &self.bounds {
            if bounds.contains_point(px, py) {
                return Some(self);
            }
        }
        None
    }

    /// Renders semantic HTML markup for screen reader Shadow DOM virtual overlays.
    pub fn to_html_overlay(&self) -> String {
        let mut html = String::new();
        self.render_html_recursive(&mut html, 0);
        html
    }

    fn render_html_recursive(&self, out: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        let role_attr = format!("role=\"{}\"", self.role.as_str());
        let id_attr = format!("id=\"aria-{}\"", self.id);
        let label_attr = format!("aria-label=\"{}\"", self.label);

        let mut extra_attrs = String::new();
        if let Some(lvl) = self.level {
            extra_attrs.push_str(&format!(" aria-level=\"{}\"", lvl));
        }
        if self.selected {
            extra_attrs.push_str(" aria-selected=\"true\"");
        }
        if self.disabled {
            extra_attrs.push_str(" aria-disabled=\"true\"");
        }
        if let Some(live) = &self.live {
            extra_attrs.push_str(&format!(" aria-live=\"{}\"", live));
        }

        let tag = match self.role {
            AriaRole::Document => "div",
            AriaRole::Heading => "h2",
            AriaRole::Paragraph => "p",
            AriaRole::Grid => "table",
            AriaRole::Row => "tr",
            AriaRole::Cell => "td",
            AriaRole::Button => "button",
            AriaRole::Textbox => "input",
            AriaRole::List => "ul",
            AriaRole::ListItem => "li",
            AriaRole::Shape | AriaRole::Image => "div",
            AriaRole::Region => "section",
        };

        if self.children.is_empty() {
            let val = self.value.as_deref().unwrap_or(&self.label);
            out.push_str(&format!(
                "{}<{tag} {} {} {}{}>{}</{tag}>\n",
                indent, id_attr, role_attr, label_attr, extra_attrs, val
            ));
        } else {
            out.push_str(&format!(
                "{}<{tag} {} {} {}{}>\n",
                indent, id_attr, role_attr, label_attr, extra_attrs
            ));
            for child in &self.children {
                child.render_html_recursive(out, depth + 1);
            }
            out.push_str(&format!("{}</{tag}>\n", indent));
        }
    }
}

/// Virtualized Accessibility Tree covering an active editor canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AriaTree {
    pub root: AriaNode,
    pub total_nodes: usize,
}

impl AriaTree {
    pub fn new(root: AriaNode) -> Self {
        let total_nodes = root.node_count();
        Self { root, total_nodes }
    }

    pub fn find_node(&self, id: &str) -> Option<&AriaNode> {
        self.root.find_node(id)
    }

    pub fn find_node_at_point(&self, px: f32, py: f32) -> Option<&AriaNode> {
        self.root.find_node_at_point(px, py)
    }

    pub fn to_html_shadow_dom(&self) -> String {
        self.root.to_html_overlay()
    }
}

/// CJK Input Method Editor (IME) live composition state and candidate popup placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImeCompositionState {
    pub is_composing: bool,
    pub preedit_string: String,
    pub cursor_position: usize,
    pub selection_start: usize,
    pub selection_end: usize,
    pub caret_x: f32,
    pub caret_y: f32,
    pub caret_height: f32,
    pub candidate_box: AriaBoundingBox,
}

impl Default for ImeCompositionState {
    fn default() -> Self {
        Self {
            is_composing: false,
            preedit_string: String::new(),
            cursor_position: 0,
            selection_start: 0,
            selection_end: 0,
            caret_x: 0.0,
            caret_y: 0.0,
            caret_height: 18.0,
            candidate_box: AriaBoundingBox::new(0.0, 18.0, 200.0, 120.0),
        }
    }
}

impl ImeCompositionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, caret_x: f32, caret_y: f32, caret_h: f32) {
        self.is_composing = true;
        self.preedit_string.clear();
        self.cursor_position = 0;
        self.selection_start = 0;
        self.selection_end = 0;
        self.caret_x = caret_x;
        self.caret_y = caret_y;
        self.caret_height = caret_h;
        self.candidate_box = AriaBoundingBox::new(caret_x, caret_y + caret_h, 220.0, 130.0);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        text: &str,
        cursor: usize,
        sel_start: usize,
        sel_end: usize,
        caret_x: f32,
        caret_y: f32,
        caret_h: f32,
    ) {
        self.is_composing = true;
        self.preedit_string = text.to_string();
        self.cursor_position = cursor;
        self.selection_start = sel_start;
        self.selection_end = sel_end;
        self.caret_x = caret_x;
        self.caret_y = caret_y;
        self.caret_height = caret_h;
        self.candidate_box = AriaBoundingBox::new(caret_x, caret_y + caret_h, 220.0, 130.0);
    }

    pub fn finish(&mut self) -> String {
        self.is_composing = false;
        let committed = std::mem::take(&mut self.preedit_string);
        self.cursor_position = 0;
        self.selection_start = 0;
        self.selection_end = 0;
        committed
    }

    pub fn cancel(&mut self) {
        self.is_composing = false;
        self.preedit_string.clear();
        self.cursor_position = 0;
        self.selection_start = 0;
        self.selection_end = 0;
    }
}

/// Unicode text reading direction for Bidirectional (BiDi) paragraph analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BidiDirection {
    Ltr,
    Rtl,
    Neutral,
}

impl BidiDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
            Self::Neutral => "auto",
        }
    }
}

/// A contiguous run of text with uniform BiDi reading direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BidiRun {
    pub text: String,
    pub direction: BidiDirection,
    pub start_idx: usize,
    pub length: usize,
}

/// Evaluates Unicode Bidirectional (BiDi) characteristics for multi-language paragraphs.
#[derive(Debug, Clone, Default)]
pub struct BidiAnalyzer;

impl BidiAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Tests whether a Unicode scalar value is an RTL character (Arabic, Hebrew, Syriac, Thaana, Samaritan).
    pub fn is_rtl_codepoint(c: char) -> bool {
        matches!(
            c,
            '\u{0590}'..='\u{05FF}'   // Hebrew
            | '\u{0600}'..='\u{06FF}' // Arabic
            | '\u{0700}'..='\u{074F}' // Syriac
            | '\u{0750}'..='\u{077F}' // Arabic Supplement
            | '\u{0780}'..='\u{07BF}' // Thaana
            | '\u{0800}'..='\u{083F}' // Samaritan
            | '\u{08A0}'..='\u{08FF}' // Arabic Extended-A
            | '\u{FB1D}'..='\u{FB4F}' // Hebrew Presentation Forms
            | '\u{FB50}'..='\u{FDFF}' // Arabic Presentation Forms-A
            | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms-B
        )
    }

    /// Determines the base paragraph direction by examining the first strong directional character.
    pub fn detect_base_direction(text: &str) -> BidiDirection {
        for c in text.chars() {
            if Self::is_rtl_codepoint(c) {
                return BidiDirection::Rtl;
            }
            if c.is_alphabetic() {
                return BidiDirection::Ltr;
            }
        }
        BidiDirection::Neutral
    }

    /// Decomposes text into contiguous directional runs.
    pub fn analyze_runs(text: &str) -> Vec<BidiRun> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut runs = Vec::new();
        let mut current_dir: Option<BidiDirection> = None;
        let mut current_run_chars = String::new();
        let mut start_idx = 0;
        let mut current_idx = 0;

        for c in text.chars() {
            let char_dir = if Self::is_rtl_codepoint(c) {
                BidiDirection::Rtl
            } else if c.is_alphabetic() {
                BidiDirection::Ltr
            } else {
                BidiDirection::Neutral
            };

            match (current_dir, char_dir) {
                (None, dir) => {
                    current_dir = Some(dir);
                    current_run_chars.push(c);
                }
                (Some(cd), nd) if cd == nd || nd == BidiDirection::Neutral => {
                    current_run_chars.push(c);
                }
                (Some(BidiDirection::Neutral), nd) => {
                    current_dir = Some(nd);
                    current_run_chars.push(c);
                }
                (Some(cd), nd) => {
                    let len = current_run_chars.len();
                    runs.push(BidiRun {
                        text: std::mem::take(&mut current_run_chars),
                        direction: cd,
                        start_idx,
                        length: len,
                    });
                    start_idx = current_idx;
                    current_dir = Some(nd);
                    current_run_chars.push(c);
                }
            }
            current_idx += c.len_utf8();
        }

        if !current_run_chars.is_empty() {
            let dir = current_dir.unwrap_or(BidiDirection::Neutral);
            let len = current_run_chars.len();
            runs.push(BidiRun {
                text: current_run_chars,
                direction: dir,
                start_idx,
                length: len,
            });
        }

        runs
    }
}

/// Key paning targets for global F6 / Shift+F6 keyboard cycling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NavFocusPane {
    TitleBar,
    Ribbon,
    SidePane,
    Canvas,
    StatusBar,
}

impl NavFocusPane {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TitleBar => "title-bar",
            Self::Ribbon => "ribbon",
            Self::SidePane => "side-pane",
            Self::Canvas => "canvas",
            Self::StatusBar => "status-bar",
        }
    }
}

/// Global sequential keyboard navigation and paning map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardNavigationMap {
    pub active_pane: NavFocusPane,
}

impl Default for KeyboardNavigationMap {
    fn default() -> Self {
        Self {
            active_pane: NavFocusPane::Canvas,
        }
    }
}

impl KeyboardNavigationMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cycles forward to the next pane (F6 shortcut order).
    pub fn cycle_next_pane(&mut self) -> NavFocusPane {
        self.active_pane = match self.active_pane {
            NavFocusPane::TitleBar => NavFocusPane::Ribbon,
            NavFocusPane::Ribbon => NavFocusPane::SidePane,
            NavFocusPane::SidePane => NavFocusPane::Canvas,
            NavFocusPane::Canvas => NavFocusPane::StatusBar,
            NavFocusPane::StatusBar => NavFocusPane::TitleBar,
        };
        self.active_pane
    }

    /// Cycles backward to the previous pane (Shift+F6 shortcut order).
    pub fn cycle_prev_pane(&mut self) -> NavFocusPane {
        self.active_pane = match self.active_pane {
            NavFocusPane::TitleBar => NavFocusPane::StatusBar,
            NavFocusPane::Ribbon => NavFocusPane::TitleBar,
            NavFocusPane::SidePane => NavFocusPane::Ribbon,
            NavFocusPane::Canvas => NavFocusPane::SidePane,
            NavFocusPane::StatusBar => NavFocusPane::Canvas,
        };
        self.active_pane
    }

    pub fn set_pane(&mut self, pane: NavFocusPane) {
        self.active_pane = pane;
    }
}
