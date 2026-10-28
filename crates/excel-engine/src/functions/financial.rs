//! OpenOfficeRS Excel Engine - Financial Functions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements exact Excel financial mathematics: PMT, IPMT, PPMT, NPV, XNPV,
//! and iterative Newton-Raphson / secant root-finders for IRR and XIRR.

use crate::grid::{CellError, CellValue};

/// Evaluates `PMT(rate, nper, pv, [fv = 0], [type = 0])`.
pub fn fn_pmt(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let rate = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let nper = match args[1].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let pv = match args[2].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let fv = args.get(3).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let type_val = args.get(4).and_then(|v| v.as_f64()).unwrap_or(0.0);

    if nper == 0.0 {
        return CellValue::Error(CellError::Div0);
    }

    if rate == 0.0 {
        return CellValue::Number(-(pv + fv) / nper);
    }

    let pvif = (1.0 + rate).powf(nper);
    let num = rate * (pv * pvif + fv);
    let den = (1.0 + rate * type_val) * (pvif - 1.0);

    if den == 0.0 {
        CellValue::Error(CellError::Div0)
    } else {
        CellValue::Number(-num / den)
    }
}

/// Evaluates `IPMT(rate, per, nper, pv, [fv = 0], [type = 0])`.
pub fn fn_ipmt(args: &[CellValue]) -> CellValue {
    if args.len() < 4 {
        return CellValue::Error(CellError::Value);
    }
    let rate = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let per = match args[1].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let nper = match args[2].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let pv = match args[3].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let fv = args.get(4).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let type_val = args.get(5).and_then(|v| v.as_f64()).unwrap_or(0.0);

    if per < 1.0 || per > nper {
        return CellValue::Error(CellError::Num);
    }

    if rate == 0.0 {
        return CellValue::Number(0.0);
    }

    if type_val != 0.0 && per == 1.0 {
        return CellValue::Number(0.0);
    }

    let pmt_val = match fn_pmt(&[
        CellValue::Number(rate),
        CellValue::Number(nper),
        CellValue::Number(pv),
        CellValue::Number(fv),
        CellValue::Number(type_val),
    ]) {
        CellValue::Number(p) => p,
        err => return err,
    };

    // Calculate balance at period per - 1
    let k = per - 1.0;
    let pvif_k = (1.0 + rate).powf(k);
    let bal = pv * pvif_k + pmt_val * (1.0 + rate * type_val) * ((pvif_k - 1.0) / rate);

    CellValue::Number(-bal * rate)
}

/// Evaluates `PPMT(rate, per, nper, pv, [fv = 0], [type = 0])`.
pub fn fn_ppmt(args: &[CellValue]) -> CellValue {
    if args.len() < 4 {
        return CellValue::Error(CellError::Value);
    }
    let pmt_args = [
        args[0].clone(),
        args[2].clone(),
        args[3].clone(),
        args.get(4).cloned().unwrap_or(CellValue::Number(0.0)),
        args.get(5).cloned().unwrap_or(CellValue::Number(0.0)),
    ];
    let pmt_val = match fn_pmt(&pmt_args) {
        CellValue::Number(v) => v,
        err => return err,
    };
    let ipmt_val = match fn_ipmt(args) {
        CellValue::Number(v) => v,
        err => return err,
    };
    CellValue::Number(pmt_val - ipmt_val)
}

/// Evaluates `NPV(rate, value1, [value2, ...])`.
pub fn fn_npv(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let rate = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };

    if rate <= -1.0 {
        return CellValue::Error(CellError::Num);
    }

    let mut sum = 0.0;
    let mut period = 1.0;

    for arg in &args[1..] {
        match arg {
            CellValue::Array(rows) => {
                for r in rows {
                    for c in r {
                        if let Some(v) = c.as_f64() {
                            sum += v / (1.0 + rate).powf(period);
                            period += 1.0;
                        }
                    }
                }
            }
            CellValue::Number(v) => {
                sum += v / (1.0 + rate).powf(period);
                period += 1.0;
            }
            _ => {}
        }
    }

    CellValue::Number(sum)
}

/// Helper extracting a flat slice of f64 from a CellValue (scalar or array).
fn collect_f64_values(val: &CellValue, out: &mut Vec<f64>) {
    match val {
        CellValue::Number(n) => out.push(*n),
        CellValue::Array(rows) => {
            for r in rows {
                for c in r {
                    if let Some(n) = c.as_f64() {
                        out.push(n);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Evaluates `XNPV(rate, values, dates)`.
pub fn fn_xnpv(args: &[CellValue]) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let rate = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };

    if rate <= -1.0 {
        return CellValue::Error(CellError::Num);
    }

    let mut values = Vec::new();
    collect_f64_values(&args[1], &mut values);

    let mut dates = Vec::new();
    collect_f64_values(&args[2], &mut dates);

    if values.is_empty() || values.len() != dates.len() {
        return CellValue::Error(CellError::Value);
    }

    let d0 = dates[0];
    let mut xnpv = 0.0;

    for i in 0..values.len() {
        let frac = (dates[i] - d0) / 365.0;
        xnpv += values[i] / (1.0 + rate).powf(frac);
    }

    CellValue::Number(xnpv)
}

/// Evaluates `IRR(values, [guess = 0.1])` using Newton-Raphson with secant fallback bounds.
pub fn fn_irr(args: &[CellValue]) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }

    let mut values = Vec::new();
    collect_f64_values(&args[0], &mut values);

    if values.len() < 2 {
        return CellValue::Error(CellError::Num);
    }

    // Must have at least one positive and one negative cash flow
    let has_pos = values.iter().any(|&v| v > 0.0);
    let has_neg = values.iter().any(|&v| v < 0.0);
    if !has_pos || !has_neg {
        return CellValue::Error(CellError::Num);
    }

    let guess = args.get(1).and_then(|v| v.as_f64()).unwrap_or(0.1);

    let calc_npv_and_deriv = |r: f64| -> (f64, f64) {
        let mut npv = 0.0;
        let mut deriv = 0.0;
        for (i, &v) in values.iter().enumerate() {
            let t = i as f64;
            let denom = (1.0 + r).powf(t);
            npv += v / denom;
            if i > 0 {
                deriv += -t * v / ((1.0 + r).powf(t + 1.0));
            }
        }
        (npv, deriv)
    };

    let mut r = guess;
    let mut prev_r = guess * 1.05 + 0.01;
    let max_iter = 100;
    let tol = 1e-7;

    for _ in 0..max_iter {
        let (f_val, f_prime) = calc_npv_and_deriv(r);
        if f_val.abs() < tol {
            return CellValue::Number(r);
        }

        let step = if f_prime.abs() > 1e-12 {
            f_val / f_prime
        } else {
            // Secant fallback
            let (f_prev, _) = calc_npv_and_deriv(prev_r);
            let denom = f_val - f_prev;
            if denom.abs() > 1e-12 {
                f_val * (r - prev_r) / denom
            } else {
                0.01
            }
        };

        prev_r = r;
        r -= step;

        // Keep rate within plausible financial domain
        if r <= -1.0 {
            r = -0.999;
        }
    }

    // Check final convergence
    let (final_npv, _) = calc_npv_and_deriv(r);
    if final_npv.abs() < 1e-4 {
        CellValue::Number(r)
    } else {
        CellValue::Error(CellError::Num)
    }
}

/// Evaluates `XIRR(values, dates, [guess = 0.1])` using Newton-Raphson with secant fallback bounds.
pub fn fn_xirr(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }

    let mut values = Vec::new();
    collect_f64_values(&args[0], &mut values);

    let mut dates = Vec::new();
    collect_f64_values(&args[1], &mut dates);

    if values.len() < 2 || values.len() != dates.len() {
        return CellValue::Error(CellError::Num);
    }

    let has_pos = values.iter().any(|&v| v > 0.0);
    let has_neg = values.iter().any(|&v| v < 0.0);
    if !has_pos || !has_neg {
        return CellValue::Error(CellError::Num);
    }

    let guess = args.get(2).and_then(|v| v.as_f64()).unwrap_or(0.1);
    let d0 = dates[0];

    let calc_xnpv_and_deriv = |r: f64| -> (f64, f64) {
        let mut f_val = 0.0;
        let mut f_prime = 0.0;
        for i in 0..values.len() {
            let frac = (dates[i] - d0) / 365.0;
            let denom = (1.0 + r).powf(frac);
            f_val += values[i] / denom;
            if frac != 0.0 {
                f_prime += -frac * values[i] / ((1.0 + r).powf(frac + 1.0));
            }
        }
        (f_val, f_prime)
    };

    let mut r = guess;
    let mut prev_r = guess * 1.05 + 0.01;
    let max_iter = 100;
    let tol = 1e-7;

    for _ in 0..max_iter {
        let (f_val, f_prime) = calc_xnpv_and_deriv(r);
        if f_val.abs() < tol {
            return CellValue::Number(r);
        }

        let step = if f_prime.abs() > 1e-12 {
            f_val / f_prime
        } else {
            let (f_prev, _) = calc_xnpv_and_deriv(prev_r);
            let denom = f_val - f_prev;
            if denom.abs() > 1e-12 {
                f_val * (r - prev_r) / denom
            } else {
                0.01
            }
        };

        prev_r = r;
        r -= step;
        if r <= -1.0 {
            r = -0.999;
        }
    }

    let (final_val, _) = calc_xnpv_and_deriv(r);
    if final_val.abs() < 1e-4 {
        CellValue::Number(r)
    } else {
        CellValue::Error(CellError::Num)
    }
}
