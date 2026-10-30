//! Zero-Knowledge memory-safe key wrapper that securely wipes bytes on drop.

use std::hint::black_box;

#[derive(Clone, PartialEq, Eq)]
pub struct ProtectedKey {
    bytes: Vec<u8>,
}

impl ProtectedKey {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            bytes: slice.to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl Drop for ProtectedKey {
    fn drop(&mut self) {
        for b in self.bytes.iter_mut() {
            *b = black_box(0u8);
        }
    }
}

impl std::fmt::Debug for ProtectedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProtectedKey([REDACTED; {} bytes])", self.bytes.len())
    }
}
