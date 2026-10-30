//! Pure-Rust, allocation-friendly Base64 encoder and decoder.

use crate::error::CryptoSecurityError;

const B64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode_base64(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        out.push(B64_CHARS[((n >> 18) & 63) as usize] as char);
        out.push(B64_CHARS[((n >> 12) & 63) as usize] as char);

        if i + 1 < data.len() {
            out.push(B64_CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(B64_CHARS[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }

        i += 3;
    }
    out
}

pub fn decode_base64(input: &str) -> Result<Vec<u8>, CryptoSecurityError> {
    let mut clean = Vec::with_capacity(input.len());
    for b in input.bytes() {
        if !b.is_ascii_whitespace() {
            clean.push(b);
        }
    }

    if clean.is_empty() {
        return Ok(Vec::new());
    }

    if clean.len() % 4 != 0 {
        return Err(CryptoSecurityError::Base64Error(format!(
            "Invalid base64 length: {}",
            clean.len()
        )));
    }

    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for chunk in clean.chunks(4) {
        let mut vals = [0u32; 4];
        let mut pad_count = 0;
        for (idx, &ch) in chunk.iter().enumerate() {
            if ch == b'=' {
                pad_count += 1;
                vals[idx] = 0;
            } else {
                vals[idx] = decode_char(ch)?;
            }
        }

        let triple = (vals[0] << 18) | (vals[1] << 12) | (vals[2] << 6) | vals[3];

        out.push(((triple >> 16) & 0xFF) as u8);
        if pad_count < 2 {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if pad_count < 1 {
            out.push((triple & 0xFF) as u8);
        }
    }

    Ok(out)
}

fn decode_char(ch: u8) -> Result<u32, CryptoSecurityError> {
    match ch {
        b'A'..=b'Z' => Ok((ch - b'A') as u32),
        b'a'..=b'z' => Ok((ch - b'a' + 26) as u32),
        b'0'..=b'9' => Ok((ch - b'0' + 52) as u32),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => Err(CryptoSecurityError::Base64Error(format!(
            "Invalid base64 character: '{}'",
            ch as char
        ))),
    }
}
