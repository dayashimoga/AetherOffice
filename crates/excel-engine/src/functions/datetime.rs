//! OpenOfficeRS Excel Engine - Date & Time Functions.
//! Strictly conforms to GEC-v1.0 (Zero Panics, Typed Results, Clean-Room Standards).
//! Implements exact 1900 vs 1904 date system math, including intentional Excel/Lotus 1-2-3
//! 1900 leap-year bug compatibility (`1900-02-29` = serial 60).

use crate::grid::{CellError, CellValue};
use ooxml_opc::error::{EngineError, Result};

/// Returns whether a year is a Gregorian leap year.
#[inline]
pub fn is_leap_year_gregorian(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in a given month for a given year.
pub fn days_in_month(year: i32, month: i32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year_gregorian(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Computes cumulative days from year 1 to the beginning of a given year (Gregorian).
fn days_before_year(year: i32) -> i64 {
    let y = (year - 1) as i64;
    y * 365 + y / 4 - y / 100 + y / 400
}

/// Computes days from beginning of year to beginning of month (1-based, Gregorian).
fn days_before_month(year: i32, month: i32) -> i32 {
    let is_leap = is_leap_year_gregorian(year);
    let regular = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];
    let idx = (month - 1).clamp(0, 11) as usize;
    if is_leap {
        leap[idx]
    } else {
        regular[idx]
    }
}

/// Converts a Gregorian (year, month, day) to an absolute day count from 0001-01-01 (where 0001-01-01 is day 1).
fn gregorian_to_abs_days(year: i32, month: i32, day: i32) -> i64 {
    days_before_year(year) + days_before_month(year, month) as i64 + (day as i64)
}

/// Converts an absolute day count from 0001-01-01 to Gregorian (year, month, day).
fn abs_days_to_gregorian(mut days: i64) -> (i32, i32, i32) {
    if days < 1 {
        return (1900, 1, 1);
    }
    // 400-year cycles = 146097 days
    let mut year = 1;
    let cycles400 = (days - 1) / 146097;
    year += (cycles400 * 400) as i32;
    days -= cycles400 * 146097;

    let cycles100 = ((days - 1) / 36524).min(3);
    year += (cycles100 * 100) as i32;
    days -= cycles100 * 36524;

    let cycles4 = ((days - 1) / 1461).min(24);
    year += (cycles4 * 4) as i32;
    days -= cycles4 * 1461;

    let cycles1 = ((days - 1) / 365).min(3);
    year += cycles1 as i32;
    days -= cycles1 * 365;

    let mut month = 1;
    while month <= 12 {
        let dim = days_in_month(year, month) as i64;
        if days <= dim {
            break;
        }
        days -= dim;
        month += 1;
    }

    (year, month, days as i32)
}

/// Converts date components to an Excel serial number, strictly accounting for
/// 1900 vs 1904 date system and the 1900-02-29 bug (serial 60).
pub fn serial_from_date(mut year: i32, mut month: i32, day: i32, date1904: bool) -> Result<f64> {
    if !(0..=9999).contains(&year) {
        return Ok(f64::NAN);
    }
    if year < 1900 {
        year += 1900;
    }

    // Month normalization: handle month < 1 or > 12
    if month > 12 {
        year += (month - 1) / 12;
        month = ((month - 1) % 12) + 1;
    } else if month < 1 {
        let years_to_sub = (-month / 12) + 1;
        year -= years_to_sub;
        month = 12 - ((-month) % 12);
    }

    // Day normalization: advance days into month
    let abs_days = gregorian_to_abs_days(year, month, day);

    if date1904 {
        // 1904-01-01 is Day 0
        let base1904 = gregorian_to_abs_days(1904, 1, 1);
        let serial = abs_days - base1904;
        if serial < 0 {
            return Err(EngineError::FormulaError(
                "Date prior to 1904 not supported in 1904 system".to_string(),
            ));
        }
        Ok(serial as f64)
    } else {
        // Excel 1900 system:
        // Day 1 = 1900-01-01
        // Day 59 = 1900-02-28
        // Day 60 = 1900-02-29 (Lotus 1-2-3 / Excel bug: 1900 treated as leap year)
        // Day 61 = 1900-03-01
        let jan1_1900 = gregorian_to_abs_days(1900, 1, 1);
        let days_from_jan1 = abs_days - jan1_1900 + 1;

        if year == 1900 && month == 2 && day == 29 {
            return Ok(60.0);
        }

        if days_from_jan1 < 1 {
            return Err(EngineError::FormulaError(
                "Date prior to 1900-01-01 not supported".to_string(),
            ));
        }

        if days_from_jan1 < 60 {
            Ok(days_from_jan1 as f64)
        } else {
            // Add 1 day for dates on or after 1900-03-01 to account for the fictional Feb 29
            Ok((days_from_jan1 + 1) as f64)
        }
    }
}

/// Converts an Excel serial number to (year, month, day), preserving 1900-02-29 (serial 60).
pub fn date_from_serial(serial: f64, date1904: bool) -> Result<(i32, i32, i32)> {
    let whole_serial = serial.floor() as i64;
    if whole_serial < 0 {
        return Err(EngineError::FormulaError(
            CellError::Num.as_str().to_string(),
        ));
    }

    if date1904 {
        let base1904 = gregorian_to_abs_days(1904, 1, 1);
        let abs_days = base1904 + whole_serial;
        Ok(abs_days_to_gregorian(abs_days))
    } else {
        if whole_serial == 0 {
            return Ok((1900, 1, 0));
        }
        if whole_serial == 60 {
            // Excel's intentional leap day bug
            return Ok((1900, 2, 29));
        }

        let jan1_1900 = gregorian_to_abs_days(1900, 1, 1);
        let adjusted_days = if whole_serial < 60 {
            whole_serial - 1
        } else {
            whole_serial - 2
        };

        let abs_days = jan1_1900 + adjusted_days;
        Ok(abs_days_to_gregorian(abs_days))
    }
}

/// Evaluates `DATE(year, month, day)`.
pub fn fn_date(args: &[CellValue], date1904: bool) -> CellValue {
    if args.len() < 3 {
        return CellValue::Error(CellError::Value);
    }
    let y = match args[0].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };
    let m = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };
    let d = match args[2].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    match serial_from_date(y, m, d, date1904) {
        Ok(serial) => CellValue::Number(serial),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Evaluates `YEAR(serial)`.
pub fn fn_year(args: &[CellValue], date1904: bool) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let s = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    match date_from_serial(s, date1904) {
        Ok((year, _, _)) => CellValue::Number(year as f64),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Evaluates `MONTH(serial)`.
pub fn fn_month(args: &[CellValue], date1904: bool) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let s = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    match date_from_serial(s, date1904) {
        Ok((_, month, _)) => CellValue::Number(month as f64),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Evaluates `DAY(serial)`.
pub fn fn_day(args: &[CellValue], date1904: bool) -> CellValue {
    if args.is_empty() {
        return CellValue::Error(CellError::Value);
    }
    let s = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    match date_from_serial(s, date1904) {
        Ok((_, _, day)) => CellValue::Number(day as f64),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Evaluates `EDATE(start_date, months)`.
pub fn fn_edate(args: &[CellValue], date1904: bool) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let s = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let months = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let (year, month, day) = match date_from_serial(s, date1904) {
        Ok(d) => d,
        Err(_) => return CellValue::Error(CellError::Num),
    };

    let total_months = (year * 12 + (month - 1)) + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12) + 1;
    let max_day = days_in_month(new_year, new_month);
    let clamped_day = day.min(max_day);

    match serial_from_date(new_year, new_month, clamped_day, date1904) {
        Ok(new_serial) => CellValue::Number(new_serial),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Evaluates `EOMONTH(start_date, months)`.
pub fn fn_eomonth(args: &[CellValue], date1904: bool) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let s = match args[0].as_f64() {
        Some(v) => v,
        None => return CellValue::Error(CellError::Value),
    };
    let months = match args[1].as_f64() {
        Some(v) => v as i32,
        None => return CellValue::Error(CellError::Value),
    };

    let (year, month, _) = match date_from_serial(s, date1904) {
        Ok(d) => d,
        Err(_) => return CellValue::Error(CellError::Num),
    };

    let total_months = (year * 12 + (month - 1)) + months;
    let new_year = total_months / 12;
    let new_month = (total_months % 12) + 1;
    let last_day = days_in_month(new_year, new_month);

    match serial_from_date(new_year, new_month, last_day, date1904) {
        Ok(new_serial) => CellValue::Number(new_serial),
        Err(_) => CellValue::Error(CellError::Num),
    }
}

/// Helper parsing weekend definition: returns a 7-element boolean array [Mon..Sun] where true = weekend.
fn parse_weekend_pattern(val: Option<&CellValue>) -> [bool; 7] {
    let mut weekend = [false; 7];
    match val {
        None => {
            // Default 1: Sat & Sun
            weekend[5] = true;
            weekend[6] = true;
        }
        Some(CellValue::Number(n)) => {
            let code = *n as i32;
            match code {
                1 => {
                    weekend[5] = true;
                    weekend[6] = true;
                } // Sat, Sun
                2 => {
                    weekend[6] = true;
                    weekend[0] = true;
                } // Sun, Mon
                3 => {
                    weekend[0] = true;
                    weekend[1] = true;
                } // Mon, Tue
                4 => {
                    weekend[1] = true;
                    weekend[2] = true;
                } // Tue, Wed
                5 => {
                    weekend[2] = true;
                    weekend[3] = true;
                } // Wed, Thu
                6 => {
                    weekend[3] = true;
                    weekend[4] = true;
                } // Thu, Fri
                7 => {
                    weekend[4] = true;
                    weekend[5] = true;
                } // Fri, Sat
                11 => {
                    weekend[6] = true;
                } // Sun only
                12 => {
                    weekend[0] = true;
                } // Mon only
                13 => {
                    weekend[1] = true;
                } // Tue only
                14 => {
                    weekend[2] = true;
                } // Wed only
                15 => {
                    weekend[3] = true;
                } // Thu only
                16 => {
                    weekend[4] = true;
                } // Fri only
                17 => {
                    weekend[5] = true;
                } // Sat only
                _ => {
                    weekend[5] = true;
                    weekend[6] = true;
                }
            }
        }
        Some(CellValue::InlineString(s)) if s.len() == 7 => {
            for (i, ch) in s.chars().enumerate() {
                if ch == '1' {
                    weekend[i] = true;
                }
            }
        }
        _ => {
            weekend[5] = true;
            weekend[6] = true;
        }
    }
    weekend
}

/// Returns day of week 0..6 (0 = Monday, 6 = Sunday) for an Excel serial date.
fn day_of_week_from_serial(serial: f64, date1904: bool) -> usize {
    // 1900-01-01 was a Monday (index 0)
    // Excel Day 1 = Monday
    if let Ok((y, m, d)) = date_from_serial(serial, date1904) {
        let abs_days = gregorian_to_abs_days(y, m, d);
        // 0001-01-01 was a Monday
        ((abs_days - 1) % 7) as usize
    } else {
        0
    }
}

/// Evaluates `WORKDAY.INTL(start_date, days, [weekend], [holidays])`.
pub fn fn_workday_intl(args: &[CellValue], date1904: bool) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let start_serial = match args[0].as_f64() {
        Some(v) => v.floor(),
        None => return CellValue::Error(CellError::Value),
    };
    let days = match args[1].as_f64() {
        Some(v) => v as i64,
        None => return CellValue::Error(CellError::Value),
    };

    let weekend = parse_weekend_pattern(args.get(2));

    // Optional holidays set
    let mut holidays = std::collections::HashSet::new();
    if let Some(hol_arg) = args.get(3) {
        match hol_arg {
            CellValue::Number(h) => {
                holidays.insert(h.floor() as i64);
            }
            CellValue::Array(rows) => {
                for r in rows {
                    for c in r {
                        if let Some(h) = c.as_f64() {
                            holidays.insert(h.floor() as i64);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut current_serial = start_serial as i64;
    let step = if days >= 0 { 1 } else { -1 };
    let mut remaining = days.abs();

    while remaining > 0 {
        current_serial += step;
        let dow = day_of_week_from_serial(current_serial as f64, date1904);
        if !weekend[dow] && !holidays.contains(&current_serial) {
            remaining -= 1;
        }
    }

    CellValue::Number(current_serial as f64)
}

/// Evaluates `NETWORKDAYS.INTL(start_date, end_date, [weekend], [holidays])`.
pub fn fn_networkdays_intl(args: &[CellValue], date1904: bool) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let start_serial = match args[0].as_f64() {
        Some(v) => v.floor() as i64,
        None => return CellValue::Error(CellError::Value),
    };
    let end_serial = match args[1].as_f64() {
        Some(v) => v.floor() as i64,
        None => return CellValue::Error(CellError::Value),
    };

    let weekend = parse_weekend_pattern(args.get(2));

    let mut holidays = std::collections::HashSet::new();
    if let Some(hol_arg) = args.get(3) {
        match hol_arg {
            CellValue::Number(h) => {
                holidays.insert(h.floor() as i64);
            }
            CellValue::Array(rows) => {
                for r in rows {
                    for c in r {
                        if let Some(h) = c.as_f64() {
                            holidays.insert(h.floor() as i64);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let forward = start_serial <= end_serial;
    let (d1, d2) = if forward {
        (start_serial, end_serial)
    } else {
        (end_serial, start_serial)
    };

    let mut count = 0i64;
    for day in d1..=d2 {
        let dow = day_of_week_from_serial(day as f64, date1904);
        if !weekend[dow] && !holidays.contains(&day) {
            count += 1;
        }
    }

    if forward {
        CellValue::Number(count as f64)
    } else {
        CellValue::Number(-count as f64)
    }
}
