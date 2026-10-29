#![forbid(unsafe_code)]
//! DrawingML Guide Formula Evaluator (ECMA-376 Part 1, Section 20.1.9.11)
//!
//! Evaluates guide formulas (`a:gdLst`, `a:avLst`) used for parametric shape geometry.
//! Supports all 17 DrawingML arithmetic and geometric operations:
//! `val`, `+-`, `*/`, `+/`, `?:`, `abs`, `at2`, `cat2`, `sat2`, `cos`, `sin`, `tan`, `sqrt`, `min`, `max`, `pin`, `mod`.

use ooxml_opc::error::EngineError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Single geometry guide or adjust value: `<a:gd name="..." fmla="..."/>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeomGuide {
    pub name: String,
    pub formula: String,
}

impl GeomGuide {
    pub fn new(name: impl Into<String>, formula: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            formula: formula.into(),
        }
    }
}

/// Evaluation context containing built-in constants, adjust values, and computed guides.
#[derive(Debug, Clone)]
pub struct GuideContext {
    variables: BTreeMap<String, i64>,
}

impl GuideContext {
    /// Create a new guide context initialized with standard ECMA-376 shape bounds and constants.
    pub fn new(w: i64, h: i64) -> Self {
        let mut ctx = Self {
            variables: BTreeMap::new(),
        };

        let l = 0i64;
        let t = 0i64;
        let r = w;
        let b = h;
        let hc = w / 2;
        let vc = h / 2;
        let ss = w.min(h);
        let ls = w.max(h);
        let ssd2 = ss / 2;

        // Angle constants in 60,000ths of a degree (ECMA-376 standard)
        let cd2 = 10_800_000i64; // 180 degrees
        let cd4 = 5_400_000i64; // 90 degrees
        let cd8 = 2_700_000i64; // 45 degrees
        let _3cd4 = 16_200_000i64; // 270 degrees
        let _3cd8 = 8_100_000i64; // 135 degrees
        let _5cd8 = 13_500_000i64; // 225 degrees
        let _7cd8 = 18_900_000i64; // 315 degrees

        ctx.set("w", w);
        ctx.set("h", h);
        ctx.set("l", l);
        ctx.set("t", t);
        ctx.set("r", r);
        ctx.set("b", b);
        ctx.set("hc", hc);
        ctx.set("vc", vc);
        ctx.set("ss", ss);
        ctx.set("ls", ls);
        ctx.set("ssd2", ssd2);

        ctx.set("cd2", cd2);
        ctx.set("cd4", cd4);
        ctx.set("cd8", cd8);
        ctx.set("3cd4", _3cd4);
        ctx.set("3cd8", _3cd8);
        ctx.set("5cd8", _5cd8);
        ctx.set("7cd8", _7cd8);

        ctx
    }

    /// Retrieve a variable by name.
    pub fn get(&self, name: &str) -> Option<i64> {
        self.variables.get(name).copied()
    }

    /// Set or update a variable.
    pub fn set(&mut self, name: impl Into<String>, value: i64) {
        self.variables.insert(name.into(), value);
    }

    /// Resolve an operand token: either an integer literal or a variable lookup.
    pub fn resolve_operand(&self, token: &str) -> Result<i64, EngineError> {
        let trimmed = token.trim();
        if let Ok(val) = trimmed.parse::<i64>() {
            return Ok(val);
        }
        self.get(trimmed).ok_or_else(|| {
            EngineError::FormulaError(format!("Undefined DrawingML guide variable: '{trimmed}'"))
        })
    }

    /// Evaluate a single DrawingML formula string and return the computed value.
    pub fn eval_formula(&self, formula: &str) -> Result<i64, EngineError> {
        let tokens: Vec<&str> = formula.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(0);
        }

        let op = tokens[0];
        let args = &tokens[1..];

        match op {
            "val" => {
                if args.is_empty() {
                    return Err(EngineError::FormulaError("val requires 1 argument".into()));
                }
                self.resolve_operand(args[0])
            }
            "+-" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("+- requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                Ok(x + y - z)
            }
            "*/" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("*/ requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                if z == 0 {
                    Ok(0)
                } else {
                    let prod = (x as i128) * (y as i128);
                    Ok((prod / (z as i128)) as i64)
                }
            }
            "+/" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("+/ requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                if z == 0 {
                    Ok(0)
                } else {
                    let sum = (x as i128) + (y as i128);
                    Ok((sum / (z as i128)) as i64)
                }
            }
            "?:" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("?: requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                Ok(if x > 0 { y } else { z })
            }
            "abs" => {
                if args.is_empty() {
                    return Err(EngineError::FormulaError("abs requires 1 argument".into()));
                }
                let x = self.resolve_operand(args[0])?;
                Ok(x.abs())
            }
            "at2" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("at2 requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                // atan2(y, x) converted to 60,000ths of a degree
                let rad = (y as f64).atan2(x as f64);
                let deg = rad.to_degrees();
                let drawingml_angle = (deg * 60_000.0).round() as i64;
                Ok(drawingml_angle)
            }
            "cat2" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError(
                        "cat2 requires 3 arguments".into(),
                    ));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                let angle = (z as f64).atan2(y as f64);
                let res = (x as f64) * angle.cos();
                Ok(res.round() as i64)
            }
            "sat2" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError(
                        "sat2 requires 3 arguments".into(),
                    ));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                let angle = (z as f64).atan2(y as f64);
                let res = (x as f64) * angle.sin();
                Ok(res.round() as i64)
            }
            "cos" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("cos requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let deg = (y as f64) / 60_000.0;
                let rad = deg.to_radians();
                let res = (x as f64) * rad.cos();
                Ok(res.round() as i64)
            }
            "sin" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("sin requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let deg = (y as f64) / 60_000.0;
                let rad = deg.to_radians();
                let res = (x as f64) * rad.sin();
                Ok(res.round() as i64)
            }
            "tan" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("tan requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let deg = (y as f64) / 60_000.0;
                let rad = deg.to_radians();
                let res = (x as f64) * rad.tan();
                Ok(res.round() as i64)
            }
            "sqrt" => {
                if args.is_empty() {
                    return Err(EngineError::FormulaError("sqrt requires 1 argument".into()));
                }
                let x = self.resolve_operand(args[0])?;
                if x <= 0 {
                    Ok(0)
                } else {
                    Ok((x as f64).sqrt().round() as i64)
                }
            }
            "min" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("min requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                Ok(x.min(y))
            }
            "max" => {
                if args.len() < 2 {
                    return Err(EngineError::FormulaError("max requires 2 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                Ok(x.max(y))
            }
            "pin" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("pin requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])?;
                let y = self.resolve_operand(args[1])?;
                let z = self.resolve_operand(args[2])?;
                if y < x {
                    Ok(x)
                } else if y > z {
                    Ok(z)
                } else {
                    Ok(y)
                }
            }
            "mod" => {
                if args.len() < 3 {
                    return Err(EngineError::FormulaError("mod requires 3 arguments".into()));
                }
                let x = self.resolve_operand(args[0])? as f64;
                let y = self.resolve_operand(args[1])? as f64;
                let z = self.resolve_operand(args[2])? as f64;
                let mag = (x * x + y * y + z * z).sqrt();
                Ok(mag.round() as i64)
            }
            _ => Err(EngineError::FormulaError(format!(
                "Unsupported DrawingML guide operation: '{op}'"
            ))),
        }
    }

    /// Evaluate a sequence of guide definitions sequentially, updating context variables.
    pub fn evaluate_guides(&mut self, guides: &[GeomGuide]) -> Result<(), EngineError> {
        for gd in guides {
            let val = self.eval_formula(&gd.formula)?;
            self.set(&gd.name, val);
        }
        Ok(())
    }
}
