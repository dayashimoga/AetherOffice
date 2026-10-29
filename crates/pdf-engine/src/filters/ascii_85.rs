//! ASCII85Decode filter (ISO 32000-1 Clause 7.4.3).

use crate::error::PdfError;

pub fn decode_ascii_85(data: &[u8]) -> Result<Vec<u8>, PdfError> {
    // Strip leading '<~' and locate trailing '~>'
    let mut start = 0;
    if data.len() >= 2 && data[0] == b'<' && data[1] == b'~' {
        start = 2;
    }

    let end = data[start..]
        .windows(2)
        .position(|w| w == b"~>")
        .map(|pos| start + pos)
        .unwrap_or(data.len());

    let slice = &data[start..end];
    let mut tuple = [0u8; 5];
    let mut tuple_len = 0;
    let mut result = Vec::with_capacity(slice.len());

    for (idx, &b) in slice.iter().enumerate() {
        if b.is_ascii_whitespace() {
            continue;
        }

        if b == b'z' {
            if tuple_len != 0 {
                return Err(PdfError::DecompressionError(format!(
                    "Unexpected 'z' inside ASCII85 5-tuple at offset {}",
                    idx
                )));
            }
            result.extend_from_slice(&[0, 0, 0, 0]);
            continue;
        }

        if !(b'!'..=b'u').contains(&b) {
            return Err(PdfError::DecompressionError(format!(
                "Invalid byte '{}' at offset {} in ASCII85 stream",
                b as char, idx
            )));
        }

        tuple[tuple_len] = b - b'!';
        tuple_len += 1;

        if tuple_len == 5 {
            let val = (tuple[0] as u32) * 85u32.pow(4)
                + (tuple[1] as u32) * 85u32.pow(3)
                + (tuple[2] as u32) * 85u32.pow(2)
                + (tuple[3] as u32) * 85
                + (tuple[4] as u32);

            result.push((val >> 24) as u8);
            result.push((val >> 16) as u8);
            result.push((val >> 8) as u8);
            result.push(val as u8);
            tuple_len = 0;
        }
    }

    // Handle partial final tuple (2 to 4 characters)
    if tuple_len > 1 {
        for slot in tuple.iter_mut().skip(tuple_len) {
            *slot = 84; // 84 corresponds to 'u'
        }
        let val = (tuple[0] as u32) * 85u32.pow(4)
            + (tuple[1] as u32) * 85u32.pow(3)
            + (tuple[2] as u32) * 85u32.pow(2)
            + (tuple[3] as u32) * 85
            + (tuple[4] as u32);

        let bytes = [
            (val >> 24) as u8,
            (val >> 16) as u8,
            (val >> 8) as u8,
            val as u8,
        ];
        result.extend_from_slice(&bytes[..tuple_len - 1]);
    } else if tuple_len == 1 {
        return Err(PdfError::DecompressionError(
            "ASCII85 stream ends with invalid single character".into(),
        ));
    }

    Ok(result)
}
