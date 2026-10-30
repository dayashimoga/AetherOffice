//! ASN.1 DER (Distinguished Encoding Rules) parser and encoder.

use crate::error::CryptoSecurityError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerValue {
    Boolean(bool),
    Integer(Vec<u8>),
    BitString {
        unused_bits: u8,
        data: Vec<u8>,
    },
    OctetString(Vec<u8>),
    Null,
    Oid(String),
    Utf8String(String),
    PrintableString(String),
    Ia5String(String),
    UtcTime(u64),
    GeneralizedTime(u64),
    Sequence(Vec<DerValue>),
    Set(Vec<DerValue>),
    ContextSpecific {
        tag: u8,
        constructed: bool,
        data: Vec<u8>,
    },
    Raw {
        tag: u8,
        data: Vec<u8>,
    },
}

pub const TAG_BOOLEAN: u8 = 0x01;
pub const TAG_INTEGER: u8 = 0x02;
pub const TAG_BIT_STRING: u8 = 0x03;
pub const TAG_OCTET_STRING: u8 = 0x04;
pub const TAG_NULL: u8 = 0x05;
pub const TAG_OID: u8 = 0x06;
pub const TAG_UTF8_STRING: u8 = 0x0C;
pub const TAG_PRINTABLE_STRING: u8 = 0x13;
pub const TAG_IA5_STRING: u8 = 0x16;
pub const TAG_UTCTIME: u8 = 0x17;
pub const TAG_GENERALIZED_TIME: u8 = 0x18;
pub const TAG_SEQUENCE: u8 = 0x30;
pub const TAG_SET: u8 = 0x31;

impl DerValue {
    /// Parse a single DER value from bytes, returning the value and total consumed byte count.
    pub fn parse(input: &[u8]) -> Result<(Self, usize), CryptoSecurityError> {
        if input.is_empty() {
            return Err(CryptoSecurityError::DerDecodeError(
                "Unexpected end of input reading DER tag".to_string(),
            ));
        }

        let tag = input[0];
        let mut offset = 1;

        if offset >= input.len() {
            return Err(CryptoSecurityError::DerDecodeError(
                "Unexpected end of input reading length".to_string(),
            ));
        }

        let len_byte = input[offset];
        offset += 1;

        let (value_len, len_octets) = if len_byte & 0x80 == 0 {
            (len_byte as usize, 0)
        } else {
            let num_octets = (len_byte & 0x7F) as usize;
            if num_octets == 0 || num_octets > 4 {
                return Err(CryptoSecurityError::DerDecodeError(format!(
                    "Unsupported DER multi-byte length: {num_octets} octets"
                )));
            }
            if offset + num_octets > input.len() {
                return Err(CryptoSecurityError::DerDecodeError(
                    "Unexpected EOF reading DER length octets".to_string(),
                ));
            }
            let mut acc = 0usize;
            for &b in &input[offset..offset + num_octets] {
                acc = (acc << 8) | (b as usize);
            }
            offset += num_octets;
            (acc, num_octets)
        };

        if offset + value_len > input.len() {
            return Err(CryptoSecurityError::DerDecodeError(format!(
                "DER value length {value_len} exceeds available input {}",
                input.len() - offset
            )));
        }

        let payload = &input[offset..offset + value_len];
        let total_consumed = 1 + 1 + len_octets + value_len;

        let parsed = match tag {
            TAG_BOOLEAN => {
                if payload.len() != 1 {
                    return Err(CryptoSecurityError::DerDecodeError(
                        "Boolean value must have length 1".to_string(),
                    ));
                }
                DerValue::Boolean(payload[0] != 0)
            }
            TAG_INTEGER => DerValue::Integer(payload.to_vec()),
            TAG_BIT_STRING => {
                if payload.is_empty() {
                    return Err(CryptoSecurityError::DerDecodeError(
                        "BitString cannot be empty".to_string(),
                    ));
                }
                DerValue::BitString {
                    unused_bits: payload[0],
                    data: payload[1..].to_vec(),
                }
            }
            TAG_OCTET_STRING => DerValue::OctetString(payload.to_vec()),
            TAG_NULL => {
                if !payload.is_empty() {
                    return Err(CryptoSecurityError::DerDecodeError(
                        "Null value must have length 0".to_string(),
                    ));
                }
                DerValue::Null
            }
            TAG_OID => {
                let oid_str = decode_oid(payload)?;
                DerValue::Oid(oid_str)
            }
            TAG_UTF8_STRING => {
                let s = std::str::from_utf8(payload)
                    .map_err(|e| CryptoSecurityError::DerDecodeError(e.to_string()))?;
                DerValue::Utf8String(s.to_string())
            }
            TAG_PRINTABLE_STRING => {
                let s = std::str::from_utf8(payload)
                    .map_err(|e| CryptoSecurityError::DerDecodeError(e.to_string()))?;
                DerValue::PrintableString(s.to_string())
            }
            TAG_IA5_STRING => {
                let s = std::str::from_utf8(payload)
                    .map_err(|e| CryptoSecurityError::DerDecodeError(e.to_string()))?;
                DerValue::Ia5String(s.to_string())
            }
            TAG_UTCTIME => {
                let s = std::str::from_utf8(payload)
                    .map_err(|e| CryptoSecurityError::DerDecodeError(e.to_string()))?;
                let epoch = parse_utc_time(s)?;
                DerValue::UtcTime(epoch)
            }
            TAG_GENERALIZED_TIME => {
                let s = std::str::from_utf8(payload)
                    .map_err(|e| CryptoSecurityError::DerDecodeError(e.to_string()))?;
                let epoch = parse_generalized_time(s)?;
                DerValue::GeneralizedTime(epoch)
            }
            TAG_SEQUENCE => {
                let items = parse_children(payload)?;
                DerValue::Sequence(items)
            }
            TAG_SET => {
                let items = parse_children(payload)?;
                DerValue::Set(items)
            }
            0x80..=0xBF => {
                let is_constructed = (tag & 0x20) != 0;
                let context_tag = tag & 0x1F;
                DerValue::ContextSpecific {
                    tag: context_tag,
                    constructed: is_constructed,
                    data: payload.to_vec(),
                }
            }
            _ => DerValue::Raw {
                tag,
                data: payload.to_vec(),
            },
        };

        Ok((parsed, total_consumed))
    }

    /// Encode this DER value into bytes.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            DerValue::Boolean(val) => {
                let mut out = vec![TAG_BOOLEAN, 0x01];
                out.push(if *val { 0xFF } else { 0x00 });
                out
            }
            DerValue::Integer(bytes) => wrap_tlv(TAG_INTEGER, bytes),
            DerValue::BitString { unused_bits, data } => {
                let mut payload = Vec::with_capacity(1 + data.len());
                payload.push(*unused_bits);
                payload.extend_from_slice(data);
                wrap_tlv(TAG_BIT_STRING, &payload)
            }
            DerValue::OctetString(bytes) => wrap_tlv(TAG_OCTET_STRING, bytes),
            DerValue::Null => vec![TAG_NULL, 0x00],
            DerValue::Oid(oid) => {
                let encoded = encode_oid(oid);
                wrap_tlv(TAG_OID, &encoded)
            }
            DerValue::Utf8String(s) => wrap_tlv(TAG_UTF8_STRING, s.as_bytes()),
            DerValue::PrintableString(s) => wrap_tlv(TAG_PRINTABLE_STRING, s.as_bytes()),
            DerValue::Ia5String(s) => wrap_tlv(TAG_IA5_STRING, s.as_bytes()),
            DerValue::UtcTime(epoch) => {
                let s = format_utc_time(*epoch);
                wrap_tlv(TAG_UTCTIME, s.as_bytes())
            }
            DerValue::GeneralizedTime(epoch) => {
                let s = format_generalized_time(*epoch);
                wrap_tlv(TAG_GENERALIZED_TIME, s.as_bytes())
            }
            DerValue::Sequence(items) => {
                let mut payload = Vec::new();
                for item in items {
                    payload.extend(item.encode());
                }
                wrap_tlv(TAG_SEQUENCE, &payload)
            }
            DerValue::Set(items) => {
                let mut payload = Vec::new();
                for item in items {
                    payload.extend(item.encode());
                }
                wrap_tlv(TAG_SET, &payload)
            }
            DerValue::ContextSpecific {
                tag,
                constructed,
                data,
            } => {
                let tag_byte = 0x80 | (if *constructed { 0x20 } else { 0 }) | (*tag & 0x1F);
                wrap_tlv(tag_byte, data)
            }
            DerValue::Raw { tag, data } => wrap_tlv(*tag, data),
        }
    }
}

fn wrap_tlv(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + 4 + payload.len());
    out.push(tag);
    encode_length(payload.len(), &mut out);
    out.extend_from_slice(payload);
    out
}

fn encode_length(len: usize, out: &mut Vec<u8>) {
    if len < 128 {
        out.push(len as u8);
    } else if len <= 0xFF {
        out.push(0x81);
        out.push(len as u8);
    } else if len <= 0xFFFF {
        out.push(0x82);
        out.push(((len >> 8) & 0xFF) as u8);
        out.push((len & 0xFF) as u8);
    } else if len <= 0xFF_FFFF {
        out.push(0x83);
        out.push(((len >> 16) & 0xFF) as u8);
        out.push(((len >> 8) & 0xFF) as u8);
        out.push((len & 0xFF) as u8);
    } else {
        out.push(0x84);
        out.push(((len >> 24) & 0xFF) as u8);
        out.push(((len >> 16) & 0xFF) as u8);
        out.push(((len >> 8) & 0xFF) as u8);
        out.push((len & 0xFF) as u8);
    }
}

fn parse_children(payload: &[u8]) -> Result<Vec<DerValue>, CryptoSecurityError> {
    let mut items = Vec::new();
    let mut offset = 0;
    while offset < payload.len() {
        let (child, consumed) = DerValue::parse(&payload[offset..])?;
        if consumed == 0 {
            break;
        }
        items.push(child);
        offset += consumed;
    }
    Ok(items)
}

fn decode_oid(bytes: &[u8]) -> Result<String, CryptoSecurityError> {
    if bytes.is_empty() {
        return Err(CryptoSecurityError::DerDecodeError("Empty OID".to_string()));
    }
    let first = bytes[0];
    let arc1 = first / 40;
    let arc2 = first % 40;
    let mut parts = vec![arc1.to_string(), arc2.to_string()];

    let mut current_arc: u64 = 0;
    for &b in &bytes[1..] {
        current_arc = (current_arc << 7) | ((b & 0x7F) as u64);
        if (b & 0x80) == 0 {
            parts.push(current_arc.to_string());
            current_arc = 0;
        }
    }
    Ok(parts.join("."))
}

fn encode_oid(oid: &str) -> Vec<u8> {
    let parts: Vec<u64> = oid.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() < 2 {
        return vec![0x00];
    }
    let first_byte = ((parts[0] * 40) + parts[1]) as u8;
    let mut out = vec![first_byte];

    for &arc in &parts[2..] {
        let mut tmp = Vec::new();
        let mut val = arc;
        tmp.push((val & 0x7F) as u8);
        val >>= 7;
        while val > 0 {
            tmp.push(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
        tmp.reverse();
        out.extend(tmp);
    }
    out
}

fn parse_utc_time(s: &str) -> Result<u64, CryptoSecurityError> {
    // Format: YYMMDDhhmmssZ
    if s.len() < 13 {
        return Err(CryptoSecurityError::DerDecodeError(format!(
            "Invalid UTCTime string: {s}"
        )));
    }
    let yy: u64 = s[0..2]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime YY error: {e}")))?;
    let year = if yy >= 50 { 1900 + yy } else { 2000 + yy };
    let month: u64 = s[2..4]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime MM error: {e}")))?;
    let day: u64 = s[4..6]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime DD error: {e}")))?;
    let hour: u64 = s[6..8]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime hh error: {e}")))?;
    let min: u64 = s[8..10]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime mm error: {e}")))?;
    let sec: u64 = s[10..12]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("UTCTime ss error: {e}")))?;

    compute_unix_epoch(year, month, day, hour, min, sec)
}

fn parse_generalized_time(s: &str) -> Result<u64, CryptoSecurityError> {
    // Format: YYYYMMDDhhmmssZ
    if s.len() < 15 {
        return Err(CryptoSecurityError::DerDecodeError(format!(
            "Invalid GeneralizedTime string: {s}"
        )));
    }
    let year: u64 = s[0..4]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime YYYY error: {e}")))?;
    let month: u64 = s[4..6]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime MM error: {e}")))?;
    let day: u64 = s[6..8]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime DD error: {e}")))?;
    let hour: u64 = s[8..10]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime hh error: {e}")))?;
    let min: u64 = s[10..12]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime mm error: {e}")))?;
    let sec: u64 = s[12..14]
        .parse()
        .map_err(|e| CryptoSecurityError::DerDecodeError(format!("GenTime ss error: {e}")))?;

    compute_unix_epoch(year, month, day, hour, min, sec)
}

fn compute_unix_epoch(
    year: u64,
    month: u64,
    day: u64,
    hour: u64,
    min: u64,
    sec: u64,
) -> Result<u64, CryptoSecurityError> {
    if year < 1970 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(CryptoSecurityError::DerDecodeError(format!(
            "Date out of range: {year}-{month}-{day}"
        )));
    }
    let mut days: u64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for m in 1..month {
        days += month_days[(m - 1) as usize];
    }
    days += day - 1;
    let total_secs = days * 86400 + hour * 3600 + min * 60 + sec;
    Ok(total_secs)
}

fn is_leap_year(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn format_utc_time(epoch: u64) -> String {
    // Simplified UTCTime formatting: YYMMDDhhmmssZ
    let (year, month, day, hour, min, sec) = epoch_to_datetime(epoch);
    let yy = year % 100;
    format!("{yy:02}{month:02}{day:02}{hour:02}{min:02}{sec:02}Z")
}

fn format_generalized_time(epoch: u64) -> String {
    let (year, month, day, hour, min, sec) = epoch_to_datetime(epoch);
    format!("{year:04}{month:02}{day:02}{hour:02}{min:02}{sec:02}Z")
}

fn epoch_to_datetime(epoch: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = epoch % 60;
    let total_min = epoch / 60;
    let min = total_min % 60;
    let total_hours = total_min / 60;
    let hour = total_hours % 24;
    let mut total_days = total_hours / 24;

    let mut year = 1970;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if total_days >= days_in_year {
            total_days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let month_days = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for &d in &month_days {
        if total_days >= d {
            total_days -= d;
            month += 1;
        } else {
            break;
        }
    }
    let day = total_days + 1;

    (year, month, day, hour, min, sec)
}
