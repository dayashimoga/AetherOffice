use serde::{Deserialize, Serialize};

/// Compiled prompt output ready for submission to the local inference backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledPrompt {
    pub system_role: String,
    pub user_prompt: String,
    pub expected_output_format: String,
}

/// Specialized prompt templates for Word document authoring and editing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WordPromptTemplate {
    SummarizeSection {
        section_name: String,
        focus_bullet_count: usize,
        target_length: String,
    },
    RewriteConcise {
        original_text: String,
        target_reduction_pct: u8,
        preserve_terms: Vec<String>,
    },
    FixToneAndGrammar {
        original_text: String,
        target_tone: String, // e.g. "Executive", "Formal", "Direct", "Technical"
    },
}

impl WordPromptTemplate {
    pub fn compile(&self) -> CompiledPrompt {
        match self {
            WordPromptTemplate::SummarizeSection {
                section_name,
                focus_bullet_count,
                target_length,
            } => CompiledPrompt {
                system_role: "You are an executive document editor specializing in synthesizing corporate reports."
                    .to_string(),
                user_prompt: format!(
                    "Please summarize the section titled '{}'.\n\
Provide exactly {} bullet points with high informational density.\n\
Target summary length: {}.\n\
Ensure every bullet ends with an explicit reference to the source passage [1], [2].",
                    section_name, focus_bullet_count, target_length
                ),
                expected_output_format: "Markdown bulleted list with source citations".to_string(),
            },
            WordPromptTemplate::RewriteConcise {
                original_text,
                target_reduction_pct,
                preserve_terms,
            } => CompiledPrompt {
                system_role: "You are a professional editor. Rewrite text with zero fluff while preserving all core technical and quantitative substance."
                    .to_string(),
                user_prompt: format!(
                    "Rewrite the following text to be approximately {}% more concise:\n\n\"{}\"\n\n\
Key technical terms that MUST be preserved unchanged: {}.",
                    target_reduction_pct,
                    original_text,
                    if preserve_terms.is_empty() {
                        "None specified".to_string()
                    } else {
                        preserve_terms.join(", ")
                    }
                ),
                expected_output_format: "Concise plain text".to_string(),
            },
            WordPromptTemplate::FixToneAndGrammar {
                original_text,
                target_tone,
            } => CompiledPrompt {
                system_role: "You are an expert copy editor ensuring grammatical excellence and tone alignment."
                    .to_string(),
                user_prompt: format!(
                    "Fix all grammatical, punctuation, and structural issues in the following text, and adapt the tone to be '{}':\n\n\"{}\"",
                    target_tone, original_text
                ),
                expected_output_format: "Polished replacement text with brief explanation of key corrections".to_string(),
            },
        }
    }
}

/// Specialized prompt templates for Calc spreadsheet modeling and formulas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CalcPromptTemplate {
    ExplainFormula {
        formula: String,
        cell_ref: String,
        referenced_cells: Vec<(String, String)>,
    },
    SynthesizeFormula {
        intent: String,
        available_columns: Vec<String>,
        target_cell: String,
    },
    DetectDataAnomalies {
        range_ref: String,
        headers: Vec<String>,
        sample_rows: Vec<Vec<String>>,
    },
}

impl CalcPromptTemplate {
    pub fn compile(&self) -> CompiledPrompt {
        match self {
            CalcPromptTemplate::ExplainFormula {
                formula,
                cell_ref,
                referenced_cells,
            } => {
                let context_lines = referenced_cells
                    .iter()
                    .map(|(c, val)| format!("- Cell {}: {}", c, val))
                    .collect::<Vec<_>>()
                    .join("\n");

                CompiledPrompt {
                    system_role: "You are an Excel / Calc formula engineer explaining complex functions clearly and step-by-step."
                        .to_string(),
                    user_prompt: format!(
                        "Explain the formula '{}' in cell {}.\n\
Referenced cell values:\n{}\n\
Provide:\n1. A one-sentence plain English summary of what the formula computes.\n2. Step-by-step calculation trace.\n3. Potential edge cases (e.g. division by zero, blank inputs, #N/A).",
                        formula, cell_ref, if context_lines.is_empty() { "No sample values provided".to_string() } else { context_lines }
                    ),
                    expected_output_format: "Structured breakdown: Summary, Calculation Steps, Edge Cases".to_string(),
                }
            }
            CalcPromptTemplate::SynthesizeFormula {
                intent,
                available_columns,
                target_cell,
            } => CompiledPrompt {
                system_role: "You are a spreadsheet formula synthesizer. Generate valid, efficient Excel/Calc formulas using standard function syntax."
                    .to_string(),
                user_prompt: format!(
                    "Synthesize an Excel/Calc formula for target cell {}.\n\
User Intent: {}\n\
Available Columns: {}\n\
Respond with:\n- The exact formula starting with '='.\n- Explanation of why this formula satisfies the requirements.",
                    target_cell,
                    intent,
                    available_columns.join(", ")
                ),
                expected_output_format: "Code block containing `=FORMULA(...)` followed by explanation".to_string(),
            },
            CalcPromptTemplate::DetectDataAnomalies {
                range_ref,
                headers,
                sample_rows,
            } => {
                let mut table_lines = vec![headers.join(" | ")];
                for row in sample_rows {
                    table_lines.push(row.join(" | "));
                }

                CompiledPrompt {
                    system_role: "You are a data quality auditor scanning tabular data for outliers, inconsistencies, and formatting discrepancies."
                        .to_string(),
                    user_prompt: format!(
                        "Analyze the spreadsheet data in range {}:\n\n{}\n\n\
Detect and flag:\n1. Outlier values or negative numbers where unexpected.\n2. Inconsistent date, currency, or text formatting.\n3. Missing or null values.\n4. Recommended corrections.",
                        range_ref, table_lines.join("\n")
                    ),
                    expected_output_format: "Audit table: Row/Cell, Anomaly Type, Description, Suggested Fix".to_string(),
                }
            }
        }
    }
}

/// Specialized prompt templates for Presentation slide design, outlines, and speaker notes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlidesPromptTemplate {
    GenerateSlideFromOutline {
        outline_title: String,
        outline_points: Vec<String>,
        target_shape_count: usize,
    },
    SuggestLayout {
        slide_title: String,
        bullet_count: usize,
        has_media: bool,
        has_table: bool,
    },
    CreateSpeakerNotes {
        slide_title: String,
        slide_bullets: Vec<String>,
        target_duration_secs: u32,
    },
}

impl SlidesPromptTemplate {
    pub fn compile(&self) -> CompiledPrompt {
        match self {
            SlidesPromptTemplate::GenerateSlideFromOutline {
                outline_title,
                outline_points,
                target_shape_count,
            } => CompiledPrompt {
                system_role: "You are a presentation designer crafting concise, visually impactful slides."
                    .to_string(),
                user_prompt: format!(
                    "Generate a presentation slide based on this outline:\n\
Title: {}\n\
Outline Points:\n- {}\n\
Format into {} distinct visual cards/shapes. For each shape provide: Headline, 1-2 sentence supporting body, and suggested DrawingML icon or accent color.",
                    outline_title,
                    outline_points.join("\n- "),
                    target_shape_count
                ),
                expected_output_format: "Slide Card List: Headline, Body, Visual Cue".to_string(),
            },
            SlidesPromptTemplate::SuggestLayout {
                slide_title,
                bullet_count,
                has_media,
                has_table,
            } => CompiledPrompt {
                system_role: "You are an information architect and visual layout advisor for presentation software."
                    .to_string(),
                user_prompt: format!(
                    "Recommend the optimal slide layout for:\n\
Slide Title: '{}'\n\
Bullet Count: {}\n\
Media Attached: {}\n\
Table Present: {}\n\
Recommend either: TitleAndContent, TwoColumnComparison, CardGrid, HeroMediaSplit, or BigStatCallout, with rationale.",
                    slide_title, bullet_count, has_media, has_table
                ),
                expected_output_format: "Recommended Layout Name, Coordinate Grid Rationale, Typography Sizing".to_string(),
            },
            SlidesPromptTemplate::CreateSpeakerNotes {
                slide_title,
                slide_bullets,
                target_duration_secs,
            } => CompiledPrompt {
                system_role: "You are a professional speechwriter crafting natural, engaging presenter talk tracks."
                    .to_string(),
                user_prompt: format!(
                    "Create conversational teleprompter speaker notes for slide '{}':\n\
Slide Content:\n- {}\n\
Target presentation time for this slide: ~{} seconds.\n\
Include verbal transition into this slide and hook leading to the next slide.",
                    slide_title,
                    slide_bullets.join("\n- "),
                    target_duration_secs
                ),
                expected_output_format: "Teleprompter script with timing cues and emphasis marks".to_string(),
            },
        }
    }
}
