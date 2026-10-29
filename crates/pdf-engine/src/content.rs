//! Content Stream Tokenizer and Operator Interpreter (ISO 32000-1 Clause 9 & Annex A).

use crate::error::PdfError;
use crate::lexer::{PdfLexer, Token};
use crate::parser::PdfParser;
use crate::types::PdfObject;

#[derive(Debug, Clone, PartialEq)]
pub enum TextChunk {
    Text(Vec<u8>),
    Spacing(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentOperator {
    // Graphics State
    SaveGraphicsState,
    RestoreGraphicsState,
    ConcatMatrix([f64; 6]),
    SetLineWidth(f64),
    SetLineCap(i32),
    SetLineJoin(i32),
    SetMiterLimit(f64),
    SetDash {
        pattern: Vec<f64>,
        phase: f64,
    },
    SetExtGState(String),

    // Color Operators
    SetGrayStroke(f32),
    SetGrayFill(f32),
    SetRgbStroke {
        r: f32,
        g: f32,
        b: f32,
    },
    SetRgbFill {
        r: f32,
        g: f32,
        b: f32,
    },
    SetCmykStroke {
        c: f32,
        m: f32,
        y: f32,
        k: f32,
    },
    SetCmykFill {
        c: f32,
        m: f32,
        y: f32,
        k: f32,
    },

    // Path Construction
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
    },
    Rectangle {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    ClosePath,

    // Path Painting
    Stroke,
    CloseAndStroke,
    Fill,
    FillEvenOdd,
    FillAndStroke,
    FillAndStrokeEvenOdd,
    EndPathNoOp,

    // Text Objects
    BeginText,
    EndText,
    SetFont {
        name: String,
        size: f64,
    },
    SetCharSpacing(f64),
    SetWordSpacing(f64),
    SetHorizontalScaling(f64),
    SetLeading(f64),
    SetTextRenderingMode(i32),
    SetTextRise(f64),

    // Text Positioning
    MoveTextPosition {
        tx: f64,
        ty: f64,
    },
    MoveTextPositionSetLeading {
        tx: f64,
        ty: f64,
    },
    SetTextMatrix([f64; 6]),
    MoveToNextLine,

    // Text Showing
    ShowText(Vec<u8>),
    MoveToNextLineShowText(Vec<u8>),
    SetSpacingShowText {
        aw: f64,
        ac: f64,
        text: Vec<u8>,
    },
    ShowTextAdjusted(Vec<TextChunk>),

    // XObjects
    Do(String),

    // Unmodeled or custom operator
    Other {
        op: String,
        operands: Vec<PdfObject>,
    },
}

pub struct ContentParser<'a> {
    data: &'a [u8],
}

impl<'a> ContentParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Parses the entire content stream into a sequence of operators.
    pub fn parse(&self) -> Result<Vec<ContentOperator>, PdfError> {
        let mut lexer = PdfLexer::new(self.data);
        let mut operands = Vec::new();
        let mut operators = Vec::new();

        while !lexer.is_eof() {
            lexer.skip_whitespace_and_comments();
            if lexer.is_eof() {
                break;
            }

            let token = lexer.next_token()?;
            match token {
                Token::Keyword(kw) => {
                    // Check if this keyword is a content stream operator or a literal value
                    if kw == "true" {
                        operands.push(PdfObject::Boolean(true));
                    } else if kw == "false" {
                        operands.push(PdfObject::Boolean(false));
                    } else if kw == "null" {
                        operands.push(PdfObject::Null);
                    } else {
                        // It is an operator
                        let op = Self::dispatch_operator(&kw, std::mem::take(&mut operands))?;
                        operators.push(op);
                    }
                }
                Token::Integer(i) => operands.push(PdfObject::Integer(i)),
                Token::Real(f) => operands.push(PdfObject::Real(f)),
                Token::Name(n) => operands.push(PdfObject::Name(n)),
                Token::LiteralString(s) => operands.push(PdfObject::LiteralString(s)),
                Token::HexString(s) => operands.push(PdfObject::HexString(s)),
                Token::ArrayOpen => {
                    let mut p = PdfParser::with_pos(self.data, lexer.pos());
                    let arr = p.parse_array()?;
                    lexer.set_pos(p.pos());
                    operands.push(arr);
                }
                Token::DictOpen => {
                    let mut p = PdfParser::with_pos(self.data, lexer.pos());
                    let dict = p.parse_dictionary()?;
                    lexer.set_pos(p.pos());
                    operands.push(dict);
                }
                Token::DictClose | Token::ArrayClose | Token::Eof => {}
            }
        }

        Ok(operators)
    }

    fn dispatch_operator(op: &str, mut args: Vec<PdfObject>) -> Result<ContentOperator, PdfError> {
        let count = args.len();
        let parsed_op = match op {
            "q" => ContentOperator::SaveGraphicsState,
            "Q" => ContentOperator::RestoreGraphicsState,
            "cm" if count >= 6 => {
                let f = Self::pop_floats(&mut args, 6);
                ContentOperator::ConcatMatrix([f[0], f[1], f[2], f[3], f[4], f[5]])
            }
            "w" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(1.0);
                ContentOperator::SetLineWidth(val)
            }
            "J" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_i64()).unwrap_or(0) as i32;
                ContentOperator::SetLineCap(val)
            }
            "j" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_i64()).unwrap_or(0) as i32;
                ContentOperator::SetLineJoin(val)
            }
            "M" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(10.0);
                ContentOperator::SetMiterLimit(val)
            }
            "d" if count >= 2 => {
                let phase = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                let pat_obj = args.pop().unwrap_or(PdfObject::Null);
                let mut pattern = Vec::new();
                if let Some(arr) = pat_obj.as_array() {
                    for item in arr {
                        if let Some(f) = item.as_f64() {
                            pattern.push(f);
                        }
                    }
                }
                ContentOperator::SetDash { pattern, phase }
            }
            "gs" if count >= 1 => {
                let name = args
                    .pop()
                    .and_then(|o| o.as_name().map(String::from))
                    .unwrap_or_default();
                ContentOperator::SetExtGState(name)
            }
            "G" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) as f32;
                ContentOperator::SetGrayStroke(val)
            }
            "g" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0) as f32;
                ContentOperator::SetGrayFill(val)
            }
            "RG" if count >= 3 => {
                let f = Self::pop_floats(&mut args, 3);
                ContentOperator::SetRgbStroke {
                    r: f[0] as f32,
                    g: f[1] as f32,
                    b: f[2] as f32,
                }
            }
            "rg" if count >= 3 => {
                let f = Self::pop_floats(&mut args, 3);
                ContentOperator::SetRgbFill {
                    r: f[0] as f32,
                    g: f[1] as f32,
                    b: f[2] as f32,
                }
            }
            "K" if count >= 4 => {
                let f = Self::pop_floats(&mut args, 4);
                ContentOperator::SetCmykStroke {
                    c: f[0] as f32,
                    m: f[1] as f32,
                    y: f[2] as f32,
                    k: f[3] as f32,
                }
            }
            "k" if count >= 4 => {
                let f = Self::pop_floats(&mut args, 4);
                ContentOperator::SetCmykFill {
                    c: f[0] as f32,
                    m: f[1] as f32,
                    y: f[2] as f32,
                    k: f[3] as f32,
                }
            }
            "m" if count >= 2 => {
                let f = Self::pop_floats(&mut args, 2);
                ContentOperator::MoveTo { x: f[0], y: f[1] }
            }
            "l" if count >= 2 => {
                let f = Self::pop_floats(&mut args, 2);
                ContentOperator::LineTo { x: f[0], y: f[1] }
            }
            "c" if count >= 6 => {
                let f = Self::pop_floats(&mut args, 6);
                ContentOperator::CurveTo {
                    x1: f[0],
                    y1: f[1],
                    x2: f[2],
                    y2: f[3],
                    x3: f[4],
                    y3: f[5],
                }
            }
            "re" if count >= 4 => {
                let f = Self::pop_floats(&mut args, 4);
                ContentOperator::Rectangle {
                    x: f[0],
                    y: f[1],
                    w: f[2],
                    h: f[3],
                }
            }
            "h" => ContentOperator::ClosePath,
            "S" => ContentOperator::Stroke,
            "s" => ContentOperator::CloseAndStroke,
            "f" | "F" => ContentOperator::Fill,
            "f*" => ContentOperator::FillEvenOdd,
            "B" => ContentOperator::FillAndStroke,
            "B*" => ContentOperator::FillAndStrokeEvenOdd,
            "n" => ContentOperator::EndPathNoOp,

            // Text Objects
            "BT" => ContentOperator::BeginText,
            "ET" => ContentOperator::EndText,
            "Tf" if count >= 2 => {
                let size = args.pop().and_then(|o| o.as_f64()).unwrap_or(12.0);
                let name = args
                    .pop()
                    .and_then(|o| o.as_name().map(String::from))
                    .unwrap_or_default();
                ContentOperator::SetFont { name, size }
            }
            "Tc" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                ContentOperator::SetCharSpacing(val)
            }
            "Tw" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                ContentOperator::SetWordSpacing(val)
            }
            "Tz" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(100.0);
                ContentOperator::SetHorizontalScaling(val)
            }
            "TL" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                ContentOperator::SetLeading(val)
            }
            "Tr" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_i64()).unwrap_or(0) as i32;
                ContentOperator::SetTextRenderingMode(val)
            }
            "Ts" if count >= 1 => {
                let val = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                ContentOperator::SetTextRise(val)
            }
            "Td" if count >= 2 => {
                let f = Self::pop_floats(&mut args, 2);
                ContentOperator::MoveTextPosition { tx: f[0], ty: f[1] }
            }
            "TD" if count >= 2 => {
                let f = Self::pop_floats(&mut args, 2);
                ContentOperator::MoveTextPositionSetLeading { tx: f[0], ty: f[1] }
            }
            "Tm" if count >= 6 => {
                let f = Self::pop_floats(&mut args, 6);
                ContentOperator::SetTextMatrix([f[0], f[1], f[2], f[3], f[4], f[5]])
            }
            "T*" => ContentOperator::MoveToNextLine,
            "Tj" if count >= 1 => {
                let bytes = args
                    .pop()
                    .and_then(|o| o.as_bytes().map(|b| b.to_vec()))
                    .unwrap_or_default();
                ContentOperator::ShowText(bytes)
            }
            "'" if count >= 1 => {
                let bytes = args
                    .pop()
                    .and_then(|o| o.as_bytes().map(|b| b.to_vec()))
                    .unwrap_or_default();
                ContentOperator::MoveToNextLineShowText(bytes)
            }
            "\"" if count >= 3 => {
                let bytes = args
                    .pop()
                    .and_then(|o| o.as_bytes().map(|b| b.to_vec()))
                    .unwrap_or_default();
                let ac = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                let aw = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
                ContentOperator::SetSpacingShowText {
                    aw,
                    ac,
                    text: bytes,
                }
            }
            "TJ" if count >= 1 => {
                let arr_obj = args.pop().unwrap_or(PdfObject::Null);
                let mut chunks = Vec::new();
                if let Some(arr) = arr_obj.as_array() {
                    for item in arr {
                        if let Some(b) = item.as_bytes() {
                            chunks.push(TextChunk::Text(b.to_vec()));
                        } else if let Some(adj) = item.as_f64() {
                            chunks.push(TextChunk::Spacing(adj));
                        }
                    }
                }
                ContentOperator::ShowTextAdjusted(chunks)
            }
            "Do" if count >= 1 => {
                let name = args
                    .pop()
                    .and_then(|o| o.as_name().map(String::from))
                    .unwrap_or_default();
                ContentOperator::Do(name)
            }
            other => ContentOperator::Other {
                op: other.to_string(),
                operands: args,
            },
        };

        Ok(parsed_op)
    }

    fn pop_floats(args: &mut Vec<PdfObject>, count: usize) -> Vec<f64> {
        let mut vals = Vec::with_capacity(count);
        for _ in 0..count {
            let f = args.pop().and_then(|o| o.as_f64()).unwrap_or(0.0);
            vals.push(f);
        }
        vals.reverse();
        vals
    }
}
