use crate::error::MacroError;

/// Magic signature byte starting MS-OVBA CompressedSourceCode stream.
pub const OVBA_MAGIC: u8 = 0x01;

/// Maximum uncompressed size of a single MS-OVBA chunk.
pub const CHUNK_MAX_SIZE: usize = 4096;

/// Calculates the number of bits allocated to the CopyToken offset based on decompressed chunk position.
/// Specified in [MS-OVBA] 2.4.1.3.19.2 (CopyToken Help).
#[inline]
pub fn copy_token_offset_bit_count(decompressed_pos: usize) -> usize {
    let mut bit_count = 4;
    while (1 << bit_count) < decompressed_pos && bit_count < 12 {
        bit_count += 1;
    }
    bit_count
}

/// Decompresses an MS-OVBA `CompressedSourceCode` byte stream ([MS-OVBA] 2.4.1.3).
pub fn decompress_vba_stream(data: &[u8]) -> Result<Vec<u8>, MacroError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    if data[0] != OVBA_MAGIC {
        return Err(MacroError::InvalidCompressedStream(format!(
            "Invalid MS-OVBA header byte: 0x{:02X} (expected 0x01)",
            data[0]
        )));
    }

    let mut decompressed = Vec::new();
    let mut offset = 1;

    while offset < data.len() {
        if offset + 2 > data.len() {
            break;
        }

        let header = u16::from_le_bytes([data[offset], data[offset + 1]]);
        let chunk_size = ((header & 0x0FFF) as usize) + 3;
        let chunk_sig = (header >> 12) & 0x07;
        let is_compressed = ((header >> 15) & 0x01) == 1;

        if chunk_sig != 3 {
            return Err(MacroError::InvalidCompressedStream(format!(
                "Invalid chunk signature: {chunk_sig} (expected 3) at offset {offset}"
            )));
        }

        let chunk_end = usize::min(offset + chunk_size, data.len());
        offset += 2;

        if !is_compressed {
            // Raw uncompressed chunk
            let raw_len = usize::min(chunk_size - 2, chunk_end.saturating_sub(offset));
            decompressed.extend_from_slice(&data[offset..offset + raw_len]);
            offset += raw_len;
        } else {
            // Compressed chunk with Token Sequences
            let mut chunk_decompressed = Vec::with_capacity(CHUNK_MAX_SIZE);

            while offset < chunk_end && chunk_decompressed.len() < CHUNK_MAX_SIZE {
                let flag_byte = data[offset];
                offset += 1;

                for bit in 0..8 {
                    if offset >= chunk_end || chunk_decompressed.len() >= CHUNK_MAX_SIZE {
                        break;
                    }

                    if (flag_byte & (1 << bit)) == 0 {
                        // LiteralToken: 1 raw byte
                        chunk_decompressed.push(data[offset]);
                        offset += 1;
                    } else {
                        // CopyToken: 2 bytes
                        if offset + 2 > chunk_end {
                            offset = chunk_end;
                            break;
                        }

                        let token = u16::from_le_bytes([data[offset], data[offset + 1]]);
                        offset += 2;

                        let pos = chunk_decompressed.len();
                        let bit_count = copy_token_offset_bit_count(pos);
                        let length_bits = 16 - bit_count;
                        let length_mask = (1 << length_bits) - 1;

                        let length = ((token & length_mask) as usize) + 3;
                        let copy_offset = ((token >> length_bits) as usize) + 1;

                        if copy_offset > pos {
                            return Err(MacroError::DecompressionError(format!(
                                "CopyToken offset {copy_offset} exceeds decompressed buffer size {pos}"
                            )));
                        }

                        let copy_start = pos - copy_offset;
                        for i in 0..length {
                            let b = chunk_decompressed[copy_start + (i % copy_offset)];
                            chunk_decompressed.push(b);
                        }
                    }
                }
            }

            decompressed.extend_from_slice(&chunk_decompressed);
        }
    }

    Ok(decompressed)
}

/// Compresses raw VBA source code bytes into an MS-OVBA `CompressedSourceCode` byte stream.
/// Emits compliant MS-OVBA chunks compatible with Microsoft Office and third-party tools.
pub fn compress_vba_stream(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return vec![OVBA_MAGIC];
    }

    let mut output = Vec::with_capacity(data.len() + 32);
    output.push(OVBA_MAGIC);

    let mut in_pos = 0;
    while in_pos < data.len() {
        let chunk_in_end = usize::min(in_pos + CHUNK_MAX_SIZE, data.len());
        let chunk_slice = &data[in_pos..chunk_in_end];
        in_pos = chunk_in_end;

        // Compress the chunk using greedy LZ77 matching
        let mut chunk_body = Vec::new();
        let mut slice_pos = 0;

        while slice_pos < chunk_slice.len() {
            let flag_byte_idx = chunk_body.len();
            chunk_body.push(0u8); // Placeholder for flag_byte
            let mut flag_byte = 0u8;

            for bit in 0..8 {
                if slice_pos >= chunk_slice.len() {
                    break;
                }

                let current_decomp_pos = slice_pos;
                let bit_count = copy_token_offset_bit_count(current_decomp_pos);
                let length_bits = 16 - bit_count;
                let max_offset = 1 << bit_count;
                let max_length = ((1 << length_bits) - 1) + 3;

                // Search for best match in sliding window
                let mut best_match_len = 0;
                let mut best_match_offset = 0;

                let window_start = current_decomp_pos.saturating_sub(max_offset);
                if current_decomp_pos > window_start {
                    for cand_start in window_start..current_decomp_pos {
                        let cand_offset = current_decomp_pos - cand_start;
                        let mut match_len = 0;
                        let cand_max =
                            usize::min(max_length, chunk_slice.len() - current_decomp_pos);

                        while match_len < cand_max
                            && chunk_slice[cand_start + (match_len % cand_offset)]
                                == chunk_slice[current_decomp_pos + match_len]
                        {
                            match_len += 1;
                        }

                        if match_len >= 3 && match_len > best_match_len {
                            best_match_len = match_len;
                            best_match_offset = cand_offset;
                            if best_match_len == max_length {
                                break;
                            }
                        }
                    }
                }

                if best_match_len >= 3 {
                    // Emit CopyToken
                    flag_byte |= 1 << bit;
                    let length_field = (best_match_len - 3) as u16;
                    let offset_field = ((best_match_offset - 1) as u16) << length_bits;
                    let token = offset_field | length_field;
                    chunk_body.extend_from_slice(&token.to_le_bytes());
                    slice_pos += best_match_len;
                } else {
                    // Emit LiteralToken
                    chunk_body.push(chunk_slice[slice_pos]);
                    slice_pos += 1;
                }
            }

            // Write back computed flag byte
            chunk_body[flag_byte_idx] = flag_byte;
        }

        // Chunk header: compressed bit (1) | sig (3) | (chunk_size - 3)
        let chunk_size = chunk_body.len() + 2;
        let header = (1u16 << 15) | (3u16 << 12) | ((chunk_size - 3) as u16);
        output.extend_from_slice(&header.to_le_bytes());
        output.extend_from_slice(&chunk_body);
    }

    output
}
