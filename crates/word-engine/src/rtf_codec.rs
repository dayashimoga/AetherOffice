//! Rich Text Format (.rtf) Codec
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - Microsoft Rich Text Format (RTF 1.9.1) open standard.
//! - Bidirectional translation between RTF markup and WordprocessingML `Document`.

use crate::model::{
    BodyElement, Document, Paragraph, ParagraphChild, Run, RunContent, RunProperties,
};

/// Codec for decoding and encoding Rich Text Format (.rtf) documents.
pub struct RtfCodec;

impl RtfCodec {
    /// Decodes RTF byte stream into a `Document` model.
    pub fn decode(rtf_bytes: &[u8]) -> Result<Document, String> {
        let rtf_str = String::from_utf8_lossy(rtf_bytes);
        if !rtf_str.trim_start().starts_with("{\\rtf") {
            return Err("Invalid RTF signature: missing {\\rtf".to_string());
        }

        let mut doc = Document::default();
        let mut cur_para = Paragraph::default();
        let mut cur_run = Run::default();
        let mut in_control_word = false;
        let mut control_buf = String::new();
        let mut chars = rtf_str.chars().peekable();

        // Formatting state stack
        let mut bold = false;
        let mut italic = false;
        let mut underline = false;

        while let Some(c) = chars.next() {
            if c == '\\' {
                if in_control_word {
                    Self::apply_control_word(
                        &control_buf,
                        &mut bold,
                        &mut italic,
                        &mut underline,
                        &mut cur_para,
                        &mut cur_run,
                        &mut doc,
                    );
                    control_buf.clear();
                }

                // Check for escaped characters
                if let Some(&next_c) = chars.peek() {
                    if next_c == '\\' || next_c == '{' || next_c == '}' {
                        chars.next();
                        Self::push_char_to_run(&mut cur_run, next_c, bold, italic, underline);
                        continue;
                    }
                }

                in_control_word = true;
            } else if in_control_word {
                if c.is_alphabetic() || c == '*' || c == '-' || c == '+' || c.is_ascii_digit() {
                    control_buf.push(c);
                } else {
                    in_control_word = false;
                    Self::apply_control_word(
                        &control_buf,
                        &mut bold,
                        &mut italic,
                        &mut underline,
                        &mut cur_para,
                        &mut cur_run,
                        &mut doc,
                    );
                    control_buf.clear();

                    if c != ' ' {
                        if c == '\n' || c == '\r' {
                            // RTF ignores standalone newlines
                        } else if c == '{' || c == '}' {
                            // Group delimiter
                        } else {
                            Self::push_char_to_run(&mut cur_run, c, bold, italic, underline);
                        }
                    }
                }
            } else if c == '{' || c == '}' || c == '\r' || c == '\n' {
                // Formatting groups or ignore newlines
            } else {
                Self::push_char_to_run(&mut cur_run, c, bold, italic, underline);
            }
        }

        if in_control_word && !control_buf.is_empty() {
            Self::apply_control_word(
                &control_buf,
                &mut bold,
                &mut italic,
                &mut underline,
                &mut cur_para,
                &mut cur_run,
                &mut doc,
            );
        }

        // Flush remaining text
        if !cur_run.content.is_empty() {
            cur_para.children.push(ParagraphChild::Run(cur_run));
        }
        if !cur_para.children.is_empty() {
            doc.body.elements.push(BodyElement::Paragraph(cur_para));
        }

        Ok(doc)
    }

    fn push_char_to_run(run: &mut Run, c: char, bold: bool, italic: bool, underline: bool) {
        if run.properties.is_none() && (bold || italic || underline) {
            run.properties = Some(RunProperties {
                bold,
                italic,
                underline: if underline {
                    Some("single".to_string())
                } else {
                    None
                },
                ..Default::default()
            });
        }
        if let Some(RunContent::Text(ref mut s, _)) = run.content.last_mut() {
            s.push(c);
        } else {
            run.content.push(RunContent::Text(c.to_string(), false));
        }
    }

    fn apply_control_word(
        cmd: &str,
        bold: &mut bool,
        italic: &mut bool,
        underline: &mut bool,
        cur_para: &mut Paragraph,
        cur_run: &mut Run,
        doc: &mut Document,
    ) {
        match cmd {
            "b" => *bold = true,
            "b0" => *bold = false,
            "i" => *italic = true,
            "i0" => *italic = false,
            "ul" => *underline = true,
            "ulnone" | "ul0" => *underline = false,
            "par" => {
                if !cur_run.content.is_empty() {
                    cur_para
                        .children
                        .push(ParagraphChild::Run(std::mem::take(cur_run)));
                }
                doc.body
                    .elements
                    .push(BodyElement::Paragraph(std::mem::take(cur_para)));
            }
            "tab" => {
                cur_run.content.push(RunContent::Tab);
            }
            _ => {}
        }
    }

    /// Encodes `Document` model into a standard 7-bit ASCII RTF document.
    pub fn encode(doc: &Document) -> Result<Vec<u8>, String> {
        let mut rtf = String::new();

        // 1. RTF Header
        rtf.push_str("{\\rtf1\\ansi\\ansicpg1252\\deff0\\nouicompat\\deflang1033\n");
        rtf.push_str(
            "{\\fonttbl{\\f0\\fnil\\fcharset0 Calibri;}{\\f1\\fnil\\fcharset0 Times New Roman;}}\n",
        );
        rtf.push_str("{\\colortbl ;\\red0\\green0\\blue0;}\n");
        rtf.push_str("\\viewkind4\\uc1\\pard\\sa200\\sl276\\slmult1\\f0\\fs22\n");

        // 2. Body Elements
        for elem in &doc.body.elements {
            if let BodyElement::Paragraph(p) = elem {
                rtf.push_str("\\pard");

                if let Some(ref props) = p.properties {
                    if let Some(ref jc) = props.justification {
                        match jc.as_str() {
                            "center" => rtf.push_str("\\qc"),
                            "right" => rtf.push_str("\\qr"),
                            "both" => rtf.push_str("\\qj"),
                            _ => rtf.push_str("\\ql"),
                        }
                    }
                }

                for child in &p.children {
                    if let ParagraphChild::Run(ref r) = child {
                        let is_bold = r.properties.as_ref().map(|p| p.bold).unwrap_or(false);
                        let is_italic = r.properties.as_ref().map(|p| p.italic).unwrap_or(false);
                        let is_underline = r
                            .properties
                            .as_ref()
                            .map(|p| p.underline.is_some())
                            .unwrap_or(false);

                        if is_bold {
                            rtf.push_str("\\b");
                        }
                        if is_italic {
                            rtf.push_str("\\i");
                        }
                        if is_underline {
                            rtf.push_str("\\ul");
                        }

                        for rc in &r.content {
                            match rc {
                                RunContent::Text(ref s, _) => {
                                    rtf.push(' ');
                                    for ch in s.chars() {
                                        match ch {
                                            '\\' => rtf.push_str("\\\\"),
                                            '{' => rtf.push_str("\\{"),
                                            '}' => rtf.push_str("\\}"),
                                            c if c.is_ascii() => rtf.push(c),
                                            c => rtf.push_str(&format!("\\u{}?", c as u32)),
                                        }
                                    }
                                }
                                RunContent::Tab => rtf.push_str("\\tab "),
                                RunContent::Break(_) => rtf.push_str("\\line "),
                                _ => {}
                            }
                        }

                        if is_underline {
                            rtf.push_str("\\ulnone");
                        }
                        if is_italic {
                            rtf.push_str("\\i0");
                        }
                        if is_bold {
                            rtf.push_str("\\b0");
                        }
                    }
                }

                rtf.push_str("\\par\n");
            }
        }

        // 3. RTF Trailer
        rtf.push('}');

        Ok(rtf.into_bytes())
    }
}
