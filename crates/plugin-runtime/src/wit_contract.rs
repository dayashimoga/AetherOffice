use serde::{Deserialize, Serialize};

/// Core document types supported across the office suite.
/// Matches WIT `enum document-type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentType {
    Word,
    Excel,
    Powerpoint,
    Pdf,
}

impl DocumentType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Word => "word",
            Self::Excel => "excel",
            Self::Powerpoint => "powerpoint",
            Self::Pdf => "pdf",
        }
    }
}

/// Selection metadata returned by host queries.
/// Matches WIT `record selection-info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionInfo {
    pub doc_id: String,
    pub doc_type: DocumentType,
    pub start_offset: u32,
    pub end_offset: u32,
    pub selected_text: String,
}

/// Cell coordinate for spreadsheet operations.
/// Matches WIT `record cell-coordinate`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub sheet: String,
    pub row: u32,
    pub col: u32,
}

impl CellCoordinate {
    pub fn new(sheet: impl Into<String>, row: u32, col: u32) -> Self {
        Self {
            sheet: sheet.into(),
            row,
            col,
        }
    }
}

/// Lifecycle hook events dispatched by host to plugins.
/// Matches WIT `interface lifecycle-hooks`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEvent {
    OnInit {
        plugin_id: String,
    },
    OnDocumentOpened {
        doc_id: String,
        doc_type: DocumentType,
    },
    OnBeforeSave {
        doc_id: String,
    },
    OnSelectionChanged {
        selection: SelectionInfo,
    },
}

impl LifecycleEvent {
    pub fn hook_name(&self) -> &'static str {
        match self {
            Self::OnInit { .. } => "on-init",
            Self::OnDocumentOpened { .. } => "on-document-opened",
            Self::OnBeforeSave { .. } => "on-before-save",
            Self::OnSelectionChanged { .. } => "on-selection-changed",
        }
    }
}

