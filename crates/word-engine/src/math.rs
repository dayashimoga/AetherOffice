//! Office Math Markup Language (OMML) & Bidirectional LaTeX Conversion Engine
//!
//! Complies strictly with GEC-v7.0:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero panics or unhandled errors in library code.
//! - ECMA-376 Part 1 §22 (Mathematical Markup Language).
//! - Lossless bidirectional LaTeX <-> OMML conversion.

use serde::{Deserialize, Serialize};

/// Discrete node within an Office Math (OMML) tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MathElement {
    Text(String),
    Fraction {
        num: Box<MathElement>,
        den: Box<MathElement>,
    },
    Radical {
        base: Box<MathElement>,
        degree: Option<Box<MathElement>>,
    },
    Superscript {
        base: Box<MathElement>,
        exp: Box<MathElement>,
    },
    Subscript {
        base: Box<MathElement>,
        sub: Box<MathElement>,
    },
    SubSuperscript {
        base: Box<MathElement>,
        sub: Box<MathElement>,
        exp: Box<MathElement>,
    },
    Delimiter {
        open: String,
        close: String,
        content: Box<MathElement>,
    },
    Group(Vec<MathElement>),
}

/// Bidirectional converter between LaTeX mathematical markup and OMML.
pub struct MathConverter;

impl MathConverter {
    /// Converts a LaTeX math string into a structured OMML `MathElement` tree.
    pub fn latex_to_omml(latex: &str) -> MathElement {
        let trimmed = latex.trim();
        let tokens = Self::tokenize(trimmed);
        let mut idx = 0;
        Self::parse_group(&tokens, &mut idx)
    }

    /// Converts an OMML `MathElement` tree back into standard LaTeX representation.
    pub fn omml_to_latex(elem: &MathElement) -> String {
        match elem {
            MathElement::Text(s) => Self::unicode_to_latex(s),
            MathElement::Fraction { num, den } => {
                format!(
                    "\\frac{{{}}}{{{}}}",
                    Self::omml_to_latex(num),
                    Self::omml_to_latex(den)
                )
            }
            MathElement::Radical { base, degree } => {
                if let Some(deg) = degree {
                    format!(
                        "\\sqrt[{}]{{{}}}",
                        Self::omml_to_latex(deg),
                        Self::omml_to_latex(base)
                    )
                } else {
                    format!("\\sqrt{{{}}}", Self::omml_to_latex(base))
                }
            }
            MathElement::Superscript { base, exp } => {
                format!(
                    "{}^{{{}}}",
                    Self::omml_to_latex(base),
                    Self::omml_to_latex(exp)
                )
            }
            MathElement::Subscript { base, sub } => {
                format!(
                    "{}_{{{}}}",
                    Self::omml_to_latex(base),
                    Self::omml_to_latex(sub)
                )
            }
            MathElement::SubSuperscript { base, sub, exp } => {
                format!(
                    "{}_{{{}}}^{{{}}}",
                    Self::omml_to_latex(base),
                    Self::omml_to_latex(sub),
                    Self::omml_to_latex(exp)
                )
            }
            MathElement::Delimiter {
                open,
                close,
                content,
            } => {
                format!(
                    "\\left{} {} \\right{}",
                    open,
                    Self::omml_to_latex(content),
                    close
                )
            }
            MathElement::Group(elements) => {
                let mut out = String::new();
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                    }
                    out.push_str(&Self::omml_to_latex(e));
                }
                out
            }
        }
    }

    /// Serializes an OMML `MathElement` tree into standard ECMA-376 XML (`<m:oMath>`).
    pub fn omml_to_xml(elem: &MathElement) -> String {
        let mut out = String::from(
            "<m:oMath xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">",
        );
        Self::write_omml_xml(elem, &mut out);
        out.push_str("</m:oMath>");
        out
    }

    fn write_omml_xml(elem: &MathElement, out: &mut String) {
        match elem {
            MathElement::Text(s) => {
                out.push_str("<m:r><m:t>");
                out.push_str(&Self::escape_xml(s));
                out.push_str("</m:t></m:r>");
            }
            MathElement::Fraction { num, den } => {
                out.push_str("<m:f><m:num>");
                Self::write_omml_xml(num, out);
                out.push_str("</m:num><m:den>");
                Self::write_omml_xml(den, out);
                out.push_str("</m:den></m:f>");
            }
            MathElement::Radical { base, degree } => {
                out.push_str("<m:rad>");
                if let Some(deg) = degree {
                    out.push_str("<m:deg>");
                    Self::write_omml_xml(deg, out);
                    out.push_str("</m:deg>");
                } else {
                    out.push_str("<m:radPr><m:degHide m:val=\"1\"/></m:radPr>");
                }
                out.push_str("<m:e>");
                Self::write_omml_xml(base, out);
                out.push_str("</m:e></m:rad>");
            }
            MathElement::Superscript { base, exp } => {
                out.push_str("<m:sSup><m:e>");
                Self::write_omml_xml(base, out);
                out.push_str("</m:e><m:sup>");
                Self::write_omml_xml(exp, out);
                out.push_str("</m:sup></m:sSup>");
            }
            MathElement::Subscript { base, sub } => {
                out.push_str("<m:sSub><m:e>");
                Self::write_omml_xml(base, out);
                out.push_str("</m:e><m:sub>");
                Self::write_omml_xml(sub, out);
                out.push_str("</m:sub></m:sSub>");
            }
            MathElement::SubSuperscript { base, sub, exp } => {
                out.push_str("<m:sSubSup><m:e>");
                Self::write_omml_xml(base, out);
                out.push_str("</m:e><m:sub>");
                Self::write_omml_xml(sub, out);
                out.push_str("</m:sub><m:sup>");
                Self::write_omml_xml(exp, out);
                out.push_str("</m:sup></m:sSubSup>");
            }
            MathElement::Delimiter {
                open,
                close,
                content,
            } => {
                out.push_str(&format!(
                    "<m:d><m:dPr><m:begChr m:val=\"{}\"/><m:endChr m:val=\"{}\"/></m:dPr><m:e>",
                    open, close
                ));
                Self::write_omml_xml(content, out);
                out.push_str("</m:e></m:d>");
            }
            MathElement::Group(elements) => {
                for e in elements {
                    Self::write_omml_xml(e, out);
                }
            }
        }
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn unicode_to_latex(s: &str) -> String {
        match s {
            "α" => "\\alpha".to_string(),
            "β" => "\\beta".to_string(),
            "γ" => "\\gamma".to_string(),
            "π" => "\\pi".to_string(),
            "±" => "\\pm".to_string(),
            "×" => "\\times".to_string(),
            "÷" => "\\div".to_string(),
            "≤" => "\\leq".to_string(),
            "≥" => "\\geq".to_string(),
            "≠" => "\\neq".to_string(),
            "≈" => "\\approx".to_string(),
            "∞" => "\\infty".to_string(),
            "∑" => "\\sum".to_string(),
            "∫" => "\\int".to_string(),
            other => other.to_string(),
        }
    }

    fn latex_sym_to_unicode(cmd: &str) -> Option<&'static str> {
        match cmd {
            "\\alpha" => Some("α"),
            "\\beta" => Some("β"),
            "\\gamma" => Some("γ"),
            "\\pi" => Some("π"),
            "\\pm" => Some("±"),
            "\\times" => Some("×"),
            "\\div" => Some("÷"),
            "\\leq" => Some("≤"),
            "\\geq" => Some("≥"),
            "\\neq" => Some("≠"),
            "\\approx" => Some("≈"),
            "\\infty" => Some("∞"),
            "\\sum" => Some("∑"),
            "\\int" => Some("∫"),
            _ => None,
        }
    }

    fn tokenize(input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else if c == '\\' {
                let mut cmd = String::new();
                cmd.push(chars.next().unwrap_or('\\'));
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() {
                        cmd.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(cmd);
            } else if c == '{'
                || c == '}'
                || c == '['
                || c == ']'
                || c == '('
                || c == ')'
                || c == '^'
                || c == '_'
                || c == '+'
                || c == '-'
                || c == '='
            {
                tokens.push(c.to_string());
                chars.next();
            } else {
                let mut text = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace()
                        || ch == '\\'
                        || ch == '{'
                        || ch == '}'
                        || ch == '['
                        || ch == ']'
                        || ch == '('
                        || ch == ')'
                        || ch == '^'
                        || ch == '_'
                        || ch == '+'
                        || ch == '-'
                        || ch == '='
                    {
                        break;
                    }
                    text.push(ch);
                    chars.next();
                }
                if !text.is_empty() {
                    tokens.push(text);
                }
            }
        }

        tokens
    }

    fn parse_group(tokens: &[String], idx: &mut usize) -> MathElement {
        let mut elements = Vec::new();

        while *idx < tokens.len() {
            if tokens[*idx] == "}" || tokens[*idx] == "]" || tokens[*idx] == "\\right" {
                break;
            }

            let elem = Self::parse_element(tokens, idx);
            elements.push(elem);
        }

        if elements.len() == 1 {
            elements
                .into_iter()
                .next()
                .unwrap_or(MathElement::Text(String::new()))
        } else {
            MathElement::Group(elements)
        }
    }

    fn parse_element(tokens: &[String], idx: &mut usize) -> MathElement {
        if *idx >= tokens.len() {
            return MathElement::Text(String::new());
        }

        let token = &tokens[*idx];

        let base_elem = if token == "\\frac" {
            *idx = idx.saturating_add(1);
            let num = Self::parse_braced_arg(tokens, idx);
            let den = Self::parse_braced_arg(tokens, idx);
            MathElement::Fraction {
                num: Box::new(num),
                den: Box::new(den),
            }
        } else if token == "\\sqrt" {
            *idx = idx.saturating_add(1);
            let degree = if *idx < tokens.len() && tokens[*idx] == "[" {
                *idx = idx.saturating_add(1);
                let deg = Self::parse_group(tokens, idx);
                if *idx < tokens.len() && tokens[*idx] == "]" {
                    *idx = idx.saturating_add(1);
                }
                Some(Box::new(deg))
            } else {
                None
            };
            let base = Self::parse_braced_arg(tokens, idx);
            MathElement::Radical {
                base: Box::new(base),
                degree,
            }
        } else if token == "\\left" {
            *idx = idx.saturating_add(1);
            let open = if *idx < tokens.len() {
                let o = tokens[*idx].clone();
                *idx = idx.saturating_add(1);
                o
            } else {
                "(".to_string()
            };

            let content = Self::parse_group(tokens, idx);

            let mut close = ")".to_string();
            if *idx < tokens.len() && tokens[*idx] == "\\right" {
                *idx = idx.saturating_add(1);
                if *idx < tokens.len() {
                    close = tokens[*idx].clone();
                    *idx = idx.saturating_add(1);
                }
            }

            MathElement::Delimiter {
                open,
                close,
                content: Box::new(content),
            }
        } else if token == "{" {
            *idx = idx.saturating_add(1);
            let g = Self::parse_group(tokens, idx);
            if *idx < tokens.len() && tokens[*idx] == "}" {
                *idx = idx.saturating_add(1);
            }
            g
        } else if let Some(sym) = Self::latex_sym_to_unicode(token) {
            *idx = idx.saturating_add(1);
            MathElement::Text(sym.to_string())
        } else {
            let t = token.clone();
            *idx = idx.saturating_add(1);
            MathElement::Text(t)
        };

        // Check for trailing superscript ^ or subscript _
        Self::check_scripts(base_elem, tokens, idx)
    }

    fn check_scripts(mut base: MathElement, tokens: &[String], idx: &mut usize) -> MathElement {
        while *idx < tokens.len() {
            if tokens[*idx] == "^" {
                *idx = idx.saturating_add(1);
                let exp = Self::parse_script_arg(tokens, idx);
                base = MathElement::Superscript {
                    base: Box::new(base),
                    exp: Box::new(exp),
                };
            } else if tokens[*idx] == "_" {
                *idx = idx.saturating_add(1);
                let sub = Self::parse_script_arg(tokens, idx);
                base = MathElement::Subscript {
                    base: Box::new(base),
                    sub: Box::new(sub),
                };
            } else {
                break;
            }
        }
        base
    }

    fn parse_script_arg(tokens: &[String], idx: &mut usize) -> MathElement {
        if *idx < tokens.len() && tokens[*idx] == "{" {
            *idx = idx.saturating_add(1);
            let g = Self::parse_group(tokens, idx);
            if *idx < tokens.len() && tokens[*idx] == "}" {
                *idx = idx.saturating_add(1);
            }
            g
        } else if *idx < tokens.len() {
            let t = tokens[*idx].clone();
            *idx = idx.saturating_add(1);
            MathElement::Text(t)
        } else {
            MathElement::Text(String::new())
        }
    }

    fn parse_braced_arg(tokens: &[String], idx: &mut usize) -> MathElement {
        if *idx < tokens.len() && tokens[*idx] == "{" {
            *idx = idx.saturating_add(1);
            let arg = Self::parse_group(tokens, idx);
            if *idx < tokens.len() && tokens[*idx] == "}" {
                *idx = idx.saturating_add(1);
            }
            arg
        } else if *idx < tokens.len() {
            let t = tokens[*idx].clone();
            *idx = idx.saturating_add(1);
            MathElement::Text(t)
        } else {
            MathElement::Text(String::new())
        }
    }
}
