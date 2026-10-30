//! AES-256 Block Cipher, CBC Mode with PKCS#7, and GCM Mode with GHASH (FIPS 197 & NIST SP 800-38D).
//! Pure-Rust, memory-safe, compliant with `#![forbid(unsafe_code)]`.

use crate::digest::constant_time_eq;
use crate::error::CryptoSecurityError;

const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

const INV_SBOX: [u8; 256] = [
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
    0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d,
];

const RCON: [u8; 11] = [
    0x00, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36,
];

pub struct Aes256 {
    round_keys: [u32; 60],
}

impl Aes256 {
    pub fn new(key: &[u8; 32]) -> Self {
        let mut w = [0u32; 60];
        for i in 0..8 {
            w[i] = u32::from_be_bytes([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
        }

        for i in 8..60 {
            let mut temp = w[i - 1];
            if i % 8 == 0 {
                temp = sub_word(rot_word(temp)) ^ ((RCON[i / 8] as u32) << 24);
            } else if i % 8 == 4 {
                temp = sub_word(temp);
            }
            w[i] = w[i - 8] ^ temp;
        }

        Self { round_keys: w }
    }

    pub fn encrypt_block(&self, block: &mut [u8; 16]) {
        let mut s = [0u8; 16];
        s.copy_from_slice(block);

        add_round_key(&mut s, &self.round_keys[0..4]);

        for round in 1..14 {
            sub_bytes(&mut s);
            shift_rows(&mut s);
            mix_columns(&mut s);
            add_round_key(&mut s, &self.round_keys[4 * round..4 * round + 4]);
        }

        sub_bytes(&mut s);
        shift_rows(&mut s);
        add_round_key(&mut s, &self.round_keys[56..60]);

        block.copy_from_slice(&s);
    }

    pub fn decrypt_block(&self, block: &mut [u8; 16]) {
        let mut s = [0u8; 16];
        s.copy_from_slice(block);

        add_round_key(&mut s, &self.round_keys[56..60]);

        for round in (1..14).rev() {
            inv_shift_rows(&mut s);
            inv_sub_bytes(&mut s);
            add_round_key(&mut s, &self.round_keys[4 * round..4 * round + 4]);
            inv_mix_columns(&mut s);
        }

        inv_shift_rows(&mut s);
        inv_sub_bytes(&mut s);
        add_round_key(&mut s, &self.round_keys[0..4]);

        block.copy_from_slice(&s);
    }
}

// Helpers for AES core
fn rot_word(w: u32) -> u32 {
    w.rotate_left(8)
}

fn sub_word(w: u32) -> u32 {
    let bytes = w.to_be_bytes();
    u32::from_be_bytes([
        SBOX[bytes[0] as usize],
        SBOX[bytes[1] as usize],
        SBOX[bytes[2] as usize],
        SBOX[bytes[3] as usize],
    ])
}

fn add_round_key(s: &mut [u8; 16], rk: &[u32]) {
    for col in 0..4 {
        let k_bytes = rk[col].to_be_bytes();
        for row in 0..4 {
            s[row + 4 * col] ^= k_bytes[row];
        }
    }
}

fn sub_bytes(s: &mut [u8; 16]) {
    for b in s.iter_mut() {
        *b = SBOX[*b as usize];
    }
}

fn inv_sub_bytes(s: &mut [u8; 16]) {
    for b in s.iter_mut() {
        *b = INV_SBOX[*b as usize];
    }
}

fn shift_rows(s: &mut [u8; 16]) {
    let mut temp = *s;
    // Row 0 unchanged
    // Row 1 shifts left by 1
    temp[1] = s[5];
    temp[5] = s[9];
    temp[9] = s[13];
    temp[13] = s[1];
    // Row 2 shifts left by 2
    temp[2] = s[10];
    temp[6] = s[14];
    temp[10] = s[2];
    temp[14] = s[6];
    // Row 3 shifts left by 3
    temp[3] = s[15];
    temp[7] = s[3];
    temp[11] = s[7];
    temp[15] = s[11];
    *s = temp;
}

fn inv_shift_rows(s: &mut [u8; 16]) {
    let mut temp = *s;
    // Row 1 shifts right by 1
    temp[5] = s[1];
    temp[9] = s[5];
    temp[13] = s[9];
    temp[1] = s[13];
    // Row 2 shifts right by 2
    temp[10] = s[2];
    temp[14] = s[6];
    temp[2] = s[10];
    temp[6] = s[14];
    // Row 3 shifts right by 3
    temp[15] = s[3];
    temp[3] = s[7];
    temp[7] = s[11];
    temp[11] = s[15];
    *s = temp;
}

fn xtime(b: u8) -> u8 {
    if b & 0x80 != 0 {
        (b << 1) ^ 0x1b
    } else {
        b << 1
    }
}

fn mix_single_column(a: &mut [u8]) {
    let t = a[0] ^ a[1] ^ a[2] ^ a[3];
    let u = a[0];
    a[0] ^= t ^ xtime(a[0] ^ a[1]);
    a[1] ^= t ^ xtime(a[1] ^ a[2]);
    a[2] ^= t ^ xtime(a[2] ^ a[3]);
    a[3] ^= t ^ xtime(a[3] ^ u);
}

fn mix_columns(s: &mut [u8; 16]) {
    for col in 0..4 {
        mix_single_column(&mut s[col * 4..col * 4 + 4]);
    }
}

fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0u8;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    p
}

fn inv_mix_single_column(a: &mut [u8]) {
    let s0 = a[0];
    let s1 = a[1];
    let s2 = a[2];
    let s3 = a[3];
    a[0] = gmul(s0, 0x0e) ^ gmul(s1, 0x0b) ^ gmul(s2, 0x0d) ^ gmul(s3, 0x09);
    a[1] = gmul(s0, 0x09) ^ gmul(s1, 0x0e) ^ gmul(s2, 0x0b) ^ gmul(s3, 0x0d);
    a[2] = gmul(s0, 0x0d) ^ gmul(s1, 0x09) ^ gmul(s2, 0x0e) ^ gmul(s3, 0x0b);
    a[3] = gmul(s0, 0x0b) ^ gmul(s1, 0x0d) ^ gmul(s2, 0x09) ^ gmul(s3, 0x0e);
}

fn inv_mix_columns(s: &mut [u8; 16]) {
    for col in 0..4 {
        inv_mix_single_column(&mut s[col * 4..col * 4 + 4]);
    }
}

// ==========================================
// AES-256 CBC Mode with PKCS#7 Padding
// ==========================================

pub fn aes256_cbc_encrypt(key: &[u8; 32], iv: &[u8; 16], plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256::new(key);
    let pad_len = 16 - (plaintext.len() % 16);
    let total_len = plaintext.len() + pad_len;

    let mut out = Vec::with_capacity(total_len);
    let mut prev_block = *iv;

    let mut offset = 0;
    while offset < plaintext.len() {
        let take = (plaintext.len() - offset).min(16);
        let mut block = [pad_len as u8; 16];
        block[..take].copy_from_slice(&plaintext[offset..offset + take]);

        for i in 0..16 {
            block[i] ^= prev_block[i];
        }

        cipher.encrypt_block(&mut block);
        out.extend_from_slice(&block);
        prev_block = block;
        offset += take;
    }

    if plaintext.len().is_multiple_of(16) {
        let mut block = [16u8; 16];
        for i in 0..16 {
            block[i] ^= prev_block[i];
        }
        cipher.encrypt_block(&mut block);
        out.extend_from_slice(&block);
    }

    out
}

pub fn aes256_cbc_decrypt(
    key: &[u8; 32],
    iv: &[u8; 16],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoSecurityError> {
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(16) {
        return Err(CryptoSecurityError::InvalidPadding(
            "Ciphertext must be a non-empty multiple of 16 bytes".to_string(),
        ));
    }

    let cipher = Aes256::new(key);
    let mut out = Vec::with_capacity(ciphertext.len());
    let mut prev_block = *iv;

    for chunk in ciphertext.chunks(16) {
        let mut block = [0u8; 16];
        block.copy_from_slice(chunk);
        let curr_cipher = block;

        cipher.decrypt_block(&mut block);
        for i in 0..16 {
            block[i] ^= prev_block[i];
        }
        out.extend_from_slice(&block);
        prev_block = curr_cipher;
    }

    // Validate PKCS#7 padding
    let pad_byte = *out.last().unwrap_or(&0);
    let pad_len = pad_byte as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > out.len() {
        return Err(CryptoSecurityError::InvalidPadding(format!(
            "Invalid PKCS#7 padding length: {pad_len}"
        )));
    }

    for &b in &out[out.len() - pad_len..] {
        if b != pad_byte {
            return Err(CryptoSecurityError::InvalidPadding(
                "Inconsistent PKCS#7 padding bytes".to_string(),
            ));
        }
    }

    out.truncate(out.len() - pad_len);
    Ok(out)
}

// ==========================================
// AES-256 GCM Mode with GHASH
// ==========================================

fn ghash_multiply(x: &mut [u8; 16], y: &[u8; 16]) {
    let mut z = [0u8; 16];
    let mut v = *y;

    for i in 0..128 {
        let bit = (x[i / 8] >> (7 - (i % 8))) & 1;
        if bit != 0 {
            for j in 0..16 {
                z[j] ^= v[j];
            }
        }
        let lsb = v[15] & 1;
        // Shift v right by 1
        for j in (1..16).rev() {
            v[j] = (v[j] >> 1) | ((v[j - 1] & 1) << 7);
        }
        v[0] >>= 1;

        if lsb != 0 {
            v[0] ^= 0xe1;
        }
    }
    *x = z;
}

fn compute_ghash(h: &[u8; 16], aad: &[u8], ciphertext: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];

    for chunk in aad.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        for i in 0..16 {
            y[i] ^= block[i];
        }
        ghash_multiply(&mut y, h);
    }

    for chunk in ciphertext.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        for i in 0..16 {
            y[i] ^= block[i];
        }
        ghash_multiply(&mut y, h);
    }

    let mut len_block = [0u8; 16];
    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    len_block[..8].copy_from_slice(&aad_bits.to_be_bytes());
    len_block[8..].copy_from_slice(&ct_bits.to_be_bytes());

    for i in 0..16 {
        y[i] ^= len_block[i];
    }
    ghash_multiply(&mut y, h);

    y
}

pub fn aes256_gcm_encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> (Vec<u8>, [u8; 16]) {
    let cipher = Aes256::new(key);

    let mut h = [0u8; 16];
    cipher.encrypt_block(&mut h);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    let mut encrypted_j0 = j0;
    cipher.encrypt_block(&mut encrypted_j0);

    let mut ciphertext = Vec::with_capacity(plaintext.len());
    let mut counter = 2u32;

    for chunk in plaintext.chunks(16) {
        let mut cb = [0u8; 16];
        cb[..12].copy_from_slice(nonce);
        cb[12..].copy_from_slice(&counter.to_be_bytes());
        cipher.encrypt_block(&mut cb);

        for (p, c) in chunk.iter().zip(cb.iter()) {
            ciphertext.push(p ^ c);
        }
        counter = counter.wrapping_add(1);
    }

    let mut s = compute_ghash(&h, aad, &ciphertext);
    for i in 0..16 {
        s[i] ^= encrypted_j0[i];
    }

    (ciphertext, s)
}

pub fn aes256_gcm_decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    tag: &[u8; 16],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoSecurityError> {
    let cipher = Aes256::new(key);

    let mut h = [0u8; 16];
    cipher.encrypt_block(&mut h);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;

    let mut encrypted_j0 = j0;
    cipher.encrypt_block(&mut encrypted_j0);

    let mut s = compute_ghash(&h, aad, ciphertext);
    for i in 0..16 {
        s[i] ^= encrypted_j0[i];
    }

    if !constant_time_eq(&s, tag) {
        return Err(CryptoSecurityError::AuthenticationFailed(
            "AES-GCM authentication tag mismatch".to_string(),
        ));
    }

    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut counter = 2u32;

    for chunk in ciphertext.chunks(16) {
        let mut cb = [0u8; 16];
        cb[..12].copy_from_slice(nonce);
        cb[12..].copy_from_slice(&counter.to_be_bytes());
        cipher.encrypt_block(&mut cb);

        for (c, k) in chunk.iter().zip(cb.iter()) {
            plaintext.push(c ^ k);
        }
        counter = counter.wrapping_add(1);
    }

    Ok(plaintext)
}
