//! Document Privacy Inspector and Metadata Sanitizer for OOXML packages.
//! Irreversibly strips document properties, revision history, comments, and hidden elements.

use crate::pii::{PiiMatch, PiiScanner};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizeOptions {
    pub strip_metadata: bool,
    pub strip_comments: bool,
    pub strip_revisions: bool,
    pub remove_hidden_content: bool,
    pub redact_pii: bool,
}

impl Default for SanitizeOptions {
    fn default() -> Self {
        Self {
            strip_metadata: true,
            strip_comments: true,
            strip_revisions: true,
            remove_hidden_content: true,
            redact_pii: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrivacyInspectionReport {
    pub author_metadata: Vec<(String, String)>,
    pub comments_count: usize,
    pub tracked_changes_count: usize,
    pub hidden_elements_count: usize,
    pub pii_matches: Vec<PiiMatch>,
}

pub struct DocumentSanitizer;

impl DocumentSanitizer {
    /// Inspect package parts for privacy leaks, metadata, comments, and PII.
    pub fn inspect_ooxml(parts: &[(&str, &[u8])]) -> PrivacyInspectionReport {
        let scanner = PiiScanner::new();
        let mut report = PrivacyInspectionReport::default();

        for (path, bytes) in parts {
            let path_lower = path.to_ascii_lowercase();

            // Metadata parts
            if path_lower.starts_with("docprops/") {
                if let Ok(xml) = std::str::from_utf8(bytes) {
                    extract_metadata_entries(xml, &mut report.author_metadata);
                }
            }

            // Comments
            if path_lower.contains("comments") {
                if let Ok(xml) = std::str::from_utf8(bytes) {
                    report.comments_count += xml.matches("<w:comment ").count()
                        + xml.matches("<comment ").count()
                        + xml.matches("<p:cm ").count();
                }
            }

            // Track changes in document bodies
            if path_lower.ends_with(".xml") {
                if let Ok(xml) = std::str::from_utf8(bytes) {
                    report.tracked_changes_count += xml.matches("<w:ins ").count()
                        + xml.matches("<w:del ").count()
                        + xml.matches("<w:rPrChange").count();

                    // Hidden sheets in Calc
                    if path_lower.contains("workbook.xml") {
                        report.hidden_elements_count += xml.matches(r#"state="hidden""#).count()
                            + xml.matches(r#"state="veryHidden""#).count();
                    }

                    // Hidden slides in Slides
                    if path_lower.contains("presentation.xml") {
                        report.hidden_elements_count += xml.matches(r#"show="0""#).count();
                    }

                    // Scan for PII in user-facing content
                    if path_lower.contains("document.xml")
                        || path_lower.contains("sharedstrings.xml")
                        || path_lower.contains("slides/slide")
                    {
                        let matches = scanner.scan(xml);
                        report.pii_matches.extend(matches);
                    }
                }
            }
        }

        report
    }

    /// Clean package parts according to selected sanitize options.
    pub fn clean_ooxml(
        parts: &[(&str, &[u8])],
        options: &SanitizeOptions,
    ) -> Vec<(String, Vec<u8>)> {
        let scanner = PiiScanner::new();
        let mut cleaned_parts = Vec::new();

        for (path, bytes) in parts {
            let path_lower = path.to_ascii_lowercase();

            // Strip metadata files
            if options.strip_metadata && path_lower.starts_with("docprops/") {
                // Replace with anonymized empty metadata
                let anon_xml = if path_lower.contains("core.xml") {
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Document</dc:title>
  <dc:creator>Anonymous</dc:creator>
  <cp:lastModifiedBy>Anonymous</cp:lastModifiedBy>
</cp:coreProperties>"#
                } else {
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>AetherOffice</Application>
</Properties>"#
                };
                cleaned_parts.push((path.to_string(), anon_xml.as_bytes().to_vec()));
                continue;
            }

            // Strip comment files
            if options.strip_comments && path_lower.contains("comments") {
                // Drop comment part
                continue;
            }

            // Process XML parts for tracked changes, hidden items, and PII
            if path_lower.ends_with(".xml") {
                if let Ok(xml_str) = std::str::from_utf8(bytes) {
                    let mut modified = xml_str.to_string();

                    if options.strip_revisions {
                        modified = strip_tracked_revisions(&modified);
                    }

                    if options.remove_hidden_content {
                        if path_lower.contains("workbook.xml") {
                            modified = modified
                                .replace(r#"state="hidden""#, "")
                                .replace(r#"state="veryHidden""#, "");
                        }
                        if path_lower.contains("presentation.xml") {
                            modified = modified.replace(r#"show="0""#, "");
                        }
                    }

                    if options.redact_pii
                        && (path_lower.contains("document.xml")
                            || path_lower.contains("sharedstrings.xml")
                            || path_lower.contains("slides/slide"))
                    {
                        let (redacted, _) = scanner.redact(&modified);
                        modified = redacted;
                    }

                    cleaned_parts.push((path.to_string(), modified.into_bytes()));
                    continue;
                }
            }

            // Keep unmodified part
            cleaned_parts.push((path.to_string(), bytes.to_vec()));
        }

        cleaned_parts
    }
}

fn extract_metadata_entries(xml: &str, out: &mut Vec<(String, String)>) {
    let keys = [
        "creator",
        "lastModifiedBy",
        "title",
        "subject",
        "Company",
        "Manager",
    ];
    for key in keys {
        for prefix in ["dc:", "cp:", ""] {
            let open_tag = format!("<{prefix}{key}>");
            let close_tag = format!("</{prefix}{key}>");
            if let Some(start) = xml.find(&open_tag) {
                if let Some(end) = xml[start..].find(&close_tag) {
                    let val = &xml[start + open_tag.len()..start + end];
                    out.push((key.to_string(), val.to_string()));
                    break;
                }
            }
        }
    }
}

fn strip_tracked_revisions(xml: &str) -> String {
    // Strip deletions completely, accept insertions as plain text
    let without_dels = Regex::new(r"(?s)<w:del\b[^>]*>.*?</w:del>")
        .map(|re| re.replace_all(xml, "").into_owned())
        .unwrap_or_else(|_| xml.to_string());

    let unwrapped_ins = Regex::new(r"</?w:ins\b[^>]*>")
        .map(|re| re.replace_all(&without_dels, "").into_owned())
        .unwrap_or_else(|_| without_dels);

    unwrapped_ins
}
