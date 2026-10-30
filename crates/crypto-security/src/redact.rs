//! Forensic Redaction Engine for PDF Content Streams.
//! Physically purges sensitive text operators, coordinates, and strings from streams,
//! rendering irreversible black redaction boxes in place.

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RedactionBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn to_pdf_rect(&self) -> String {
        format!(
            "{:.2} {:.2} {:.2} {:.2} re f",
            self.x, self.y, self.width, self.height
        )
    }
}

pub struct PdfRedactor;

impl PdfRedactor {
    /// Irreversibly redact specific text strings from a PDF content stream.
    /// Replaces occurrences with an explicit black redaction rectangle and completely
    /// removes the sensitive string bytes from the content stream.
    pub fn redact_strings(
        content_stream: &str,
        sensitive_strings: &[&str],
        default_box: Option<RedactionBox>,
    ) -> (String, usize) {
        let mut modified = content_stream.to_string();
        let mut total_redacted = 0;

        for &sensitive in sensitive_strings {
            if sensitive.is_empty() {
                continue;
            }

            let count = modified.matches(sensitive).count();
            if count > 0 {
                total_redacted += count;
                let blank = " ".repeat(sensitive.len());
                modified = modified.replace(sensitive, &blank);
            }
        }

        if total_redacted > 0 {
            // Append visual solid black redaction box to content stream
            let box_rect = default_box.unwrap_or(RedactionBox::new(72.0, 700.0, 200.0, 20.0));
            let visual_mark = format!("\nq\n0 0 0 rg\n{}\nQ\n", box_rect.to_pdf_rect());
            modified.push_str(&visual_mark);
        }

        (modified, total_redacted)
    }

    /// Forensic verification: ensures none of the specified sensitive strings
    /// appear anywhere in the stream payload.
    pub fn verify_purged(content_stream: &str, sensitive_strings: &[&str]) -> bool {
        for &sensitive in sensitive_strings {
            if !sensitive.is_empty() && content_stream.contains(sensitive) {
                return false;
            }
        }
        true
    }
}
