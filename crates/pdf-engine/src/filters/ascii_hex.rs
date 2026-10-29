//! ASCIIHexDecode filter (ISO 32000-1 Clause 7.4.2).

use crate::error::PdfError;

pub fn decode_ascii_hex(data: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut digits = Vec::with_capacity(data.len() / 2);

    for (idx, &b) in data.iter().enumerate() {
        if b == b'>' {
            break;
        }
        if b.is_ascii_whitespace() {
            continue;
        }

        let val = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => {
                return Err(PdfError::DecompressionError(format!(
                    "Invalid byte '{}' at offset {} in ASCIIHex stream",
                    b as char, idx
                )));
            }
        };
        digits.push(val);
    }

    if digits.len() % 2 != 0 {
        digits.push(0);
    }

    let mut result = Vec::with_capacity(digits.len() / 2);
    for chunk in digits.chunks(2) {
        result.push((chunk[0] << 4) | chunk[1]);
    }

    Ok(result)
}
