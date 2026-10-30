//! Key Derivation Functions: HMAC and PBKDF2 (RFC 2104 & RFC 8018).
//! Pure-Rust, memory-safe, compliant with `#![forbid(unsafe_code)]`.

use crate::digest::{compute_digest, HashAlgorithm};
use crate::error::CryptoSecurityError;

/// Compute HMAC according to RFC 2104.
pub fn hmac(alg: HashAlgorithm, key: &[u8], data: &[u8]) -> Vec<u8> {
    let block_size = match alg {
        HashAlgorithm::Sha256 => 64,
        HashAlgorithm::Sha384 | HashAlgorithm::Sha512 => 128,
    };

    let mut key_block = vec![0u8; block_size];
    if key.len() > block_size {
        let hashed = compute_digest(alg, key);
        key_block[..hashed.len()].copy_from_slice(&hashed);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut inner_input = Vec::with_capacity(block_size + data.len());
    for &b in &key_block {
        inner_input.push(b ^ 0x36);
    }
    inner_input.extend_from_slice(data);
    let inner_hash = compute_digest(alg, &inner_input);

    let mut outer_input = Vec::with_capacity(block_size + inner_hash.len());
    for &b in &key_block {
        outer_input.push(b ^ 0x5C);
    }
    outer_input.extend_from_slice(&inner_hash);
    compute_digest(alg, &outer_input)
}

/// Derive key bytes using PBKDF2 with specified HMAC hash algorithm (RFC 8018).
pub fn pbkdf2(
    alg: HashAlgorithm,
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    output: &mut [u8],
) -> Result<(), CryptoSecurityError> {
    if iterations == 0 {
        return Err(CryptoSecurityError::BufferError(
            "PBKDF2 iteration count must be greater than 0".to_string(),
        ));
    }
    if output.is_empty() {
        return Ok(());
    }

    let hlen = alg.digest_len();
    let num_blocks = output.len().div_ceil(hlen);

    let mut out_idx = 0;
    for block_num in 1..=num_blocks {
        let mut salt_and_block = Vec::with_capacity(salt.len() + 4);
        salt_and_block.extend_from_slice(salt);
        salt_and_block.extend_from_slice(&(block_num as u32).to_be_bytes());

        let mut u = hmac(alg, password, &salt_and_block);
        let mut t = u.clone();

        for _ in 1..iterations {
            u = hmac(alg, password, &u);
            for (t_byte, &u_byte) in t.iter_mut().zip(u.iter()) {
                *t_byte ^= u_byte;
            }
        }

        let to_copy = (output.len() - out_idx).min(hlen);
        output[out_idx..out_idx + to_copy].copy_from_slice(&t[..to_copy]);
        out_idx += to_copy;
    }

    Ok(())
}

