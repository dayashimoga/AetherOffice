use crate::error::PluginError;
use crate::wit_contract::{CellCoordinate, DocumentType, SelectionInfo};
use std::collections::HashMap;

/// Host query interface matching WIT `interface document-query`.
pub trait DocumentQueryHandler: Send + Sync {
    fn get_active_selection(&self, doc_id: &str) -> Result<SelectionInfo, PluginError>;
    fn get_cell_value(&self, coord: &CellCoordinate) -> Result<String, PluginError>;
    fn get_paragraph_text(&self, doc_id: &str, para_index: u32) -> Result<String, PluginError>;
    fn get_slide_shapes(&self, doc_id: &str, slide_index: u32) -> Result<Vec<String>, PluginError>;
}

/// Host mutation interface matching WIT `interface document-mutation`.
pub trait DocumentMutationHandler: Send + Sync {
    fn insert_text(&mut self, doc_id: &str, text: &str) -> Result<(), PluginError>;
    fn set_cell_value(&mut self, coord: &CellCoordinate, value: &str) -> Result<(), PluginError>;
    fn set_cell_formula(
        &mut self,
        coord: &CellCoordinate,
        formula: &str,
    ) -> Result<(), PluginError>;
    fn execute_ribbon_command(&mut self, action: &str, payload: &str) -> Result<bool, PluginError>;
}

/// Unified host interface combining queries and mutations.
pub trait DocumentHost: DocumentQueryHandler + DocumentMutationHandler {}
impl<T: DocumentQueryHandler + DocumentMutationHandler> DocumentHost for T {}

/// Mock in-memory host implementation for testing and headless execution.
#[derive(Debug, Default, Clone)]
pub struct MockDocumentHost {
    pub active_selection: Option<SelectionInfo>,
    pub cell_values: HashMap<CellCoordinate, String>,
    pub cell_formulas: HashMap<CellCoordinate, String>,
    pub paragraphs: HashMap<(String, u32), String>,
    pub slide_shapes: HashMap<(String, u32), Vec<String>>,
    pub executed_commands: Vec<(String, String)>,
}

impl MockDocumentHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_default_doc(mut self) -> Self {
        self.active_selection = Some(SelectionInfo {
            doc_id: "doc-1".to_string(),
            doc_type: DocumentType::Word,
            start_offset: 0,
            end_offset: 12,
            selected_text: "Hello World!".to_string(),
        });
        self.paragraphs.insert(
            ("doc-1".to_string(), 0),
            "AetherOffice document paragraph 0 content.".to_string(),
        );
        self.cell_values
            .insert(CellCoordinate::new("Sheet1", 1, 1), "4200".to_string());
        self.slide_shapes.insert(
            ("pres-1".to_string(), 0),
            vec!["TitleBox".to_string(), "ChartShape".to_string()],
        );
        self
    }
}

impl DocumentQueryHandler for MockDocumentHost {
    fn get_active_selection(&self, doc_id: &str) -> Result<SelectionInfo, PluginError> {
        if let Some(ref sel) = self.active_selection {
            if sel.doc_id == doc_id {
                return Ok(sel.clone());
            }
        }
        // Return default empty selection if not found
        Ok(SelectionInfo {
            doc_id: doc_id.to_string(),
            doc_type: DocumentType::Word,
            start_offset: 0,
            end_offset: 0,
            selected_text: String::new(),
        })
    }

    fn get_cell_value(&self, coord: &CellCoordinate) -> Result<String, PluginError> {
        Ok(self.cell_values.get(coord).cloned().unwrap_or_default())
    }

    fn get_paragraph_text(&self, doc_id: &str, para_index: u32) -> Result<String, PluginError> {
        self.paragraphs
            .get(&(doc_id.to_string(), para_index))
            .cloned()
            .ok_or_else(|| {
                PluginError::DocumentQueryError(format!(
                    "Paragraph {para_index} not found in document '{doc_id}'"
                ))
            })
    }

    fn get_slide_shapes(&self, doc_id: &str, slide_index: u32) -> Result<Vec<String>, PluginError> {
        Ok(self
            .slide_shapes
            .get(&(doc_id.to_string(), slide_index))
            .cloned()
            .unwrap_or_default())
    }
}

impl DocumentMutationHandler for MockDocumentHost {
    fn insert_text(&mut self, doc_id: &str, text: &str) -> Result<(), PluginError> {
        let entry = self.paragraphs.entry((doc_id.to_string(), 0)).or_default();
        entry.push_str(text);
        Ok(())
    }

    fn set_cell_value(&mut self, coord: &CellCoordinate, value: &str) -> Result<(), PluginError> {
        self.cell_values.insert(coord.clone(), value.to_string());
        Ok(())
    }

    fn set_cell_formula(
        &mut self,
        coord: &CellCoordinate,
        formula: &str,
    ) -> Result<(), PluginError> {
        self.cell_formulas
            .insert(coord.clone(), formula.to_string());
        Ok(())
    }

    fn execute_ribbon_command(&mut self, action: &str, payload: &str) -> Result<bool, PluginError> {
        self.executed_commands
            .push((action.to_string(), payload.to_string()));
        Ok(true)
    }
}
