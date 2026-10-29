//! Interactive AcroForms Engine (ISO 32000-1 Clause 12.7).

use crate::catalog::PdfDocument;
use crate::error::PdfError;
use crate::types::{PdfObject, PdfObjectId, PdfRect, PdfStream};
use crate::writer::PdfIncrementalWriter;
use std::collections::BTreeMap;

/// Supported interactive form field types.
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldType {
    Text {
        multiline: bool,
        password: bool,
    },
    Checkbox {
        export_value: String,
    },
    RadioButton {
        export_value: String,
    },
    Choice {
        combo: bool,
        editable: bool,
        options: Vec<String>,
    },
    Signature,
}

/// Form field value representation.
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldValue {
    Text(String),
    Boolean(bool),
    Choice(String),
}

/// Resolved interactive form field with geometry and appearance state.
#[derive(Debug, Clone, PartialEq)]
pub struct FormField {
    pub id: PdfObjectId,
    pub name: String,
    pub full_name: String,
    pub field_type: FormFieldType,
    pub value: Option<String>,
    pub default_value: Option<String>,
    pub rect: PdfRect,
    pub page_index: Option<usize>,
    pub read_only: bool,
    pub required: bool,
    pub default_appearance: Option<String>,
    pub appearance_stream: Option<PdfStream>,
}

pub struct AcroFormEngine {
    fields: Vec<FormField>,
    acro_form_dict: BTreeMap<String, PdfObject>,
}

impl AcroFormEngine {
    /// Discovers and resolves all form fields defined in the document.
    pub fn parse(doc: &PdfDocument) -> Result<Self, PdfError> {
        let mut fields = Vec::new();

        let acro_form_dict = match doc.catalog().get("AcroForm") {
            Some(PdfObject::Dictionary(d)) => d.clone(),
            Some(PdfObject::Reference(ref_id)) => {
                if let Ok(PdfObject::Dictionary(d)) = doc.resolve_object(*ref_id) {
                    d
                } else {
                    BTreeMap::new()
                }
            }
            _ => BTreeMap::new(),
        };

        if let Some(fields_obj) = acro_form_dict.get("Fields") {
            let fields_arr = match fields_obj {
                PdfObject::Array(arr) => arr.clone(),
                PdfObject::Reference(ref_id) => {
                    if let Ok(PdfObject::Array(arr)) = doc.resolve_object(*ref_id) {
                        arr
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };

            for field_ref in fields_arr {
                if let Some(id) = field_ref.as_reference() {
                    Self::traverse_field_hierarchy(id, "", None, None, doc, &mut fields)?;
                }
            }
        }

        Ok(Self {
            fields,
            acro_form_dict,
        })
    }

    pub fn fields(&self) -> &[FormField] {
        &self.fields
    }

    pub fn acro_form_dict(&self) -> &BTreeMap<String, PdfObject> {
        &self.acro_form_dict
    }

    pub fn fields_mut(&mut self) -> &mut [FormField] {
        &mut self.fields
    }

    pub fn get_field(&self, full_name: &str) -> Option<&FormField> {
        self.fields.iter().find(|f| f.full_name == full_name)
    }

    pub fn get_field_mut(&mut self, full_name: &str) -> Option<&mut FormField> {
        self.fields.iter_mut().find(|f| f.full_name == full_name)
    }

    /// Mutates a form field value and regenerates its visual appearance stream.
    pub fn set_field_value(
        &mut self,
        full_name: &str,
        new_value: FormFieldValue,
    ) -> Result<(), PdfError> {
        let field = self
            .get_field_mut(full_name)
            .ok_or_else(|| PdfError::SyntaxError {
                offset: 0,
                message: format!("AcroForm field '{}' not found", full_name),
            })?;

        match (&field.field_type, &new_value) {
            (FormFieldType::Text { .. }, FormFieldValue::Text(s)) => {
                field.value = Some(s.clone());
                let da = field
                    .default_appearance
                    .as_deref()
                    .unwrap_or("/Helv 12 Tf 0 g");
                field.appearance_stream = Some(Self::generate_text_appearance(&field.rect, s, da));
            }
            (FormFieldType::Checkbox { export_value }, FormFieldValue::Boolean(checked)) => {
                let val_str = if *checked {
                    export_value.clone()
                } else {
                    "Off".into()
                };
                field.value = Some(val_str);
                field.appearance_stream =
                    Some(Self::generate_checkbox_appearance(&field.rect, *checked));
            }
            (FormFieldType::Choice { .. }, FormFieldValue::Choice(s)) => {
                field.value = Some(s.clone());
                let da = field
                    .default_appearance
                    .as_deref()
                    .unwrap_or("/Helv 12 Tf 0 g");
                field.appearance_stream = Some(Self::generate_text_appearance(&field.rect, s, da));
            }
            _ => {
                return Err(PdfError::InvalidType {
                    expected: "compatible form field value",
                    found: "mismatched value type",
                });
            }
        }

        Ok(())
    }

    /// Generates appearance stream for single-line or multi-line text input.
    pub fn generate_text_appearance(rect: &PdfRect, text: &str, da: &str) -> PdfStream {
        let mut content = Vec::new();
        // Background and border
        let header = format!(
            "q\n0 0 {:.2} {:.2} re W n\nBT\n{}\n2 3 Td\n({}) Tj\nET\nQ\n",
            rect.width(),
            rect.height(),
            da,
            text
        );
        content.extend_from_slice(header.as_bytes());

        let mut dict = BTreeMap::new();
        dict.insert("Type".into(), PdfObject::Name("XObject".into()));
        dict.insert("Subtype".into(), PdfObject::Name("Form".into()));
        dict.insert("FormType".into(), PdfObject::Integer(1));
        dict.insert(
            "BBox".into(),
            PdfObject::Array(vec![
                PdfObject::Real(0.0),
                PdfObject::Real(0.0),
                PdfObject::Real(rect.width()),
                PdfObject::Real(rect.height()),
            ]),
        );

        PdfStream::new(dict, content)
    }

    /// Generates appearance stream for an interactive checkbox with checkmark.
    pub fn generate_checkbox_appearance(rect: &PdfRect, checked: bool) -> PdfStream {
        let mut content = Vec::new();
        let w = rect.width();
        let h = rect.height();

        if checked {
            // Draw square border with checkmark 'X' or tick
            let check_cmds = format!(
                "q\n1 w 0 0 0 RG\n0.5 0.5 {:.2} {:.2} re S\n{:.2} {:.2} m {:.2} {:.2} l S\n{:.2} {:.2} m {:.2} {:.2} l S\nQ\n",
                w - 1.0, h - 1.0,
                w * 0.2, h * 0.5, w * 0.45, h * 0.2,
                w * 0.45, h * 0.2, w * 0.8, h * 0.8
            );
            content.extend_from_slice(check_cmds.as_bytes());
        } else {
            // Unchecked empty border
            let empty_cmds = format!(
                "q\n1 w 0 0 0 RG\n0.5 0.5 {:.2} {:.2} re S\nQ\n",
                w - 1.0,
                h - 1.0
            );
            content.extend_from_slice(empty_cmds.as_bytes());
        }

        let mut dict = BTreeMap::new();
        dict.insert("Type".into(), PdfObject::Name("XObject".into()));
        dict.insert("Subtype".into(), PdfObject::Name("Form".into()));
        dict.insert("FormType".into(), PdfObject::Integer(1));
        dict.insert(
            "BBox".into(),
            PdfObject::Array(vec![
                PdfObject::Real(0.0),
                PdfObject::Real(0.0),
                PdfObject::Real(w),
                PdfObject::Real(h),
            ]),
        );

        PdfStream::new(dict, content)
    }

    /// Appends updated form field dictionaries into the incremental update writer.
    pub fn export_incremental_update(
        &self,
        writer: &mut PdfIncrementalWriter,
    ) -> Result<(), PdfError> {
        for field in &self.fields {
            if let Some(ref ap) = field.appearance_stream {
                let ap_id = writer.add_object(PdfObject::Stream(ap.clone()));

                let mut field_dict = BTreeMap::new();
                field_dict.insert("Type".into(), PdfObject::Name("Annot".into()));
                field_dict.insert("Subtype".into(), PdfObject::Name("Widget".into()));
                field_dict.insert("Rect".into(), field.rect.to_array());
                field_dict.insert(
                    "T".into(),
                    PdfObject::LiteralString(field.name.as_bytes().to_vec()),
                );

                let ft_name = match field.field_type {
                    FormFieldType::Text { .. } => "Tx",
                    FormFieldType::Checkbox { .. } | FormFieldType::RadioButton { .. } => "Btn",
                    FormFieldType::Choice { .. } => "Ch",
                    FormFieldType::Signature => "Sig",
                };
                field_dict.insert("FT".into(), PdfObject::Name(ft_name.into()));

                if let Some(ref da) = field.default_appearance {
                    field_dict.insert(
                        "DA".into(),
                        PdfObject::LiteralString(da.as_bytes().to_vec()),
                    );
                }

                let mut ff: u32 = 0;
                if field.read_only {
                    ff |= 1;
                }
                if field.required {
                    ff |= 2;
                }
                match &field.field_type {
                    FormFieldType::Text {
                        multiline,
                        password,
                    } => {
                        if *multiline {
                            ff |= 1 << 12;
                        }
                        if *password {
                            ff |= 1 << 13;
                        }
                    }
                    FormFieldType::RadioButton { .. } => {
                        ff |= 1 << 15;
                    }
                    FormFieldType::Choice {
                        combo,
                        editable,
                        ref options,
                    } => {
                        if *combo {
                            ff |= 1 << 17;
                        }
                        if *editable {
                            ff |= 1 << 18;
                        }
                        if !options.is_empty() {
                            let opt_arr = options
                                .iter()
                                .map(|opt| PdfObject::LiteralString(opt.as_bytes().to_vec()))
                                .collect();
                            field_dict.insert("Opt".into(), PdfObject::Array(opt_arr));
                        }
                    }
                    _ => {}
                }
                if ff != 0 {
                    field_dict.insert("Ff".into(), PdfObject::Integer(ff as i64));
                }

                if let Some(ref v) = field.value {
                    match field.field_type {
                        FormFieldType::Checkbox { .. } | FormFieldType::RadioButton { .. } => {
                            field_dict.insert("V".into(), PdfObject::Name(v.clone()));
                            field_dict.insert("AS".into(), PdfObject::Name(v.clone()));
                        }
                        _ => {
                            field_dict.insert(
                                "V".into(),
                                PdfObject::LiteralString(v.as_bytes().to_vec()),
                            );
                        }
                    }
                }

                let mut ap_dict = BTreeMap::new();
                ap_dict.insert("N".into(), PdfObject::Reference(ap_id));
                field_dict.insert("AP".into(), PdfObject::Dictionary(ap_dict));

                writer.set_object(field.id, PdfObject::Dictionary(field_dict));
            }
        }
        Ok(())
    }

    fn traverse_field_hierarchy(
        id: PdfObjectId,
        parent_name: &str,
        inherited_ft: Option<String>,
        inherited_da: Option<String>,
        doc: &PdfDocument,
        fields: &mut Vec<FormField>,
    ) -> Result<(), PdfError> {
        let field_obj = doc.resolve_object(id)?;
        let dict = match field_obj {
            PdfObject::Dictionary(d) => d,
            _ => return Ok(()),
        };

        let partial_name = dict
            .get("T")
            .and_then(|v| v.to_string_lossy())
            .unwrap_or_default();
        let full_name = if parent_name.is_empty() {
            partial_name.clone()
        } else if partial_name.is_empty() {
            parent_name.to_string()
        } else {
            format!("{}.{}", parent_name, partial_name)
        };

        let ft = dict
            .get("FT")
            .and_then(|v| v.as_name())
            .map(String::from)
            .or(inherited_ft);

        let da = dict
            .get("DA")
            .and_then(|v| v.to_string_lossy())
            .or(inherited_da);

        let ff = dict.get("Ff").and_then(|v| v.as_i64()).unwrap_or(0).max(0) as u32;
        let read_only = (ff & 1) != 0;
        let required = (ff & 2) != 0;

        let rect = dict
            .get("Rect")
            .and_then(|v| v.as_array())
            .and_then(|arr| PdfRect::from_array(arr).ok())
            .unwrap_or_else(|| PdfRect::new(0.0, 0.0, 150.0, 30.0));

        let value = dict.get("V").and_then(|v| v.to_string_lossy());
        let default_value = dict.get("DV").and_then(|v| v.to_string_lossy());

        // Check if there are children in /Kids
        if let Some(kids_obj) = dict.get("Kids") {
            let kids_arr = match kids_obj {
                PdfObject::Array(arr) => arr.clone(),
                PdfObject::Reference(ref_id) => {
                    if let Ok(PdfObject::Array(arr)) = doc.resolve_object(*ref_id) {
                        arr
                    } else {
                        Vec::new()
                    }
                }
                _ => Vec::new(),
            };

            for kid_ref in kids_arr {
                if let Some(kid_id) = kid_ref.as_reference() {
                    Self::traverse_field_hierarchy(
                        kid_id,
                        &full_name,
                        ft.clone(),
                        da.clone(),
                        doc,
                        fields,
                    )?;
                }
            }
        }

        // If this node represents a leaf field or widget
        if let Some(ref field_type_name) = ft {
            let field_type = match field_type_name.as_str() {
                "Tx" => {
                    let multiline = (ff & (1 << 12)) != 0;
                    let password = (ff & (1 << 13)) != 0;
                    FormFieldType::Text {
                        multiline,
                        password,
                    }
                }
                "Btn" => {
                    let is_radio = (ff & (1 << 15)) != 0;
                    let is_push = (ff & (1 << 16)) != 0;
                    if is_radio {
                        FormFieldType::RadioButton {
                            export_value: "Yes".into(),
                        }
                    } else if is_push {
                        FormFieldType::Text {
                            multiline: false,
                            password: false,
                        }
                    } else {
                        FormFieldType::Checkbox {
                            export_value: "Yes".into(),
                        }
                    }
                }
                "Ch" => {
                    let combo = (ff & (1 << 17)) != 0;
                    let editable = (ff & (1 << 18)) != 0;
                    let mut options = Vec::new();
                    if let Some(opt_arr) = dict.get("Opt").and_then(|v| v.as_array()) {
                        for item in opt_arr {
                            if let Some(s) = item.to_string_lossy() {
                                options.push(s);
                            }
                        }
                    }
                    FormFieldType::Choice {
                        combo,
                        editable,
                        options,
                    }
                }
                "Sig" => FormFieldType::Signature,
                _ => FormFieldType::Text {
                    multiline: false,
                    password: false,
                },
            };

            fields.push(FormField {
                id,
                name: partial_name,
                full_name,
                field_type,
                value,
                default_value,
                rect,
                page_index: None,
                read_only,
                required,
                default_appearance: da,
                appearance_stream: None,
            });
        }

        Ok(())
    }
}
