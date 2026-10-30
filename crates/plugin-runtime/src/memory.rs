use crate::error::PluginError;

pub const WASM_PAGE_SIZE: usize = 65_536; // 64 KiB

/// Sandboxed linear memory for a WebAssembly plugin instance.
#[derive(Debug, Clone)]
pub struct LinearMemory {
    buffer: Vec<u8>,
    min_pages: u32,
    max_pages: u32,
}

impl LinearMemory {
    /// Create a new linear memory instance bounded by min and max pages.
    pub fn new(min_pages: u32, max_pages: Option<u32>) -> Result<Self, PluginError> {
        let max = max_pages.unwrap_or(256); // default 16MB max
        if min_pages > max {
            return Err(PluginError::WasmParseError(format!(
                "Minimum pages ({min_pages}) exceeds maximum pages ({max})"
            )));
        }

        let initial_capacity = (min_pages as usize)
            .checked_mul(WASM_PAGE_SIZE)
            .ok_or_else(|| {
                PluginError::WasmParseError("Linear memory initial size overflow".to_string())
            })?;

        Ok(Self {
            buffer: vec![0u8; initial_capacity],
            min_pages,
            max_pages: max,
        })
    }

    /// Minimum configured 64 KiB pages.
    pub fn min_pages(&self) -> u32 {
        self.min_pages
    }

    /// Maximum configured 64 KiB pages.
    pub fn max_pages(&self) -> u32 {
        self.max_pages
    }

    /// Current size of linear memory in bytes.
    pub fn byte_len(&self) -> usize {
        self.buffer.len()
    }

    /// Current size of linear memory in 64 KiB pages.
    pub fn page_count(&self) -> u32 {
        (self.buffer.len() / WASM_PAGE_SIZE) as u32
    }

    /// Grow linear memory by a number of 64 KiB pages.
    pub fn grow(&mut self, delta_pages: u32) -> Result<u32, PluginError> {
        let current_pages = self.page_count();
        let new_pages = current_pages.saturating_add(delta_pages);
        if new_pages > self.max_pages {
            return Err(PluginError::MemoryOutOfBounds {
                offset: self.buffer.len(),
                len: (delta_pages as usize).saturating_mul(WASM_PAGE_SIZE),
                limit: (self.max_pages as usize).saturating_mul(WASM_PAGE_SIZE),
            });
        }

        let new_len = (new_pages as usize)
            .checked_mul(WASM_PAGE_SIZE)
            .ok_or_else(|| PluginError::MemoryOutOfBounds {
                offset: self.buffer.len(),
                len: usize::MAX,
                limit: (self.max_pages as usize).saturating_mul(WASM_PAGE_SIZE),
            })?;

        self.buffer.resize(new_len, 0);
        Ok(current_pages)
    }

    /// Read an arbitrary slice of bytes safely.
    pub fn read_bytes(&self, offset: usize, len: usize) -> Result<Vec<u8>, PluginError> {
        let end = offset
            .checked_add(len)
            .ok_or(PluginError::MemoryOutOfBounds {
                offset,
                len,
                limit: self.buffer.len(),
            })?;

        if end > self.buffer.len() {
            return Err(PluginError::MemoryOutOfBounds {
                offset,
                len,
                limit: self.buffer.len(),
            });
        }

        Ok(self.buffer[offset..end].to_vec())
    }

    /// Write an arbitrary slice of bytes safely.
    pub fn write_bytes(&mut self, offset: usize, data: &[u8]) -> Result<(), PluginError> {
        let end = offset
            .checked_add(data.len())
            .ok_or(PluginError::MemoryOutOfBounds {
                offset,
                len: data.len(),
                limit: self.buffer.len(),
            })?;

        if end > self.buffer.len() {
            return Err(PluginError::MemoryOutOfBounds {
                offset,
                len: data.len(),
                limit: self.buffer.len(),
            });
        }

        self.buffer[offset..end].copy_from_slice(data);
        Ok(())
    }

    /// Read a UTF-8 string from linear memory.
    pub fn read_utf8(&self, offset: usize, len: usize) -> Result<String, PluginError> {
        let bytes = self.read_bytes(offset, len)?;
        String::from_utf8(bytes).map_err(|e| {
            PluginError::HostCallError(format!("Invalid UTF-8 string at memory [{offset}]: {e}"))
        })
    }

    /// Write a UTF-8 string into linear memory. Returns number of bytes written.
    pub fn write_utf8(&mut self, offset: usize, text: &str) -> Result<usize, PluginError> {
        let bytes = text.as_bytes();
        self.write_bytes(offset, bytes)?;
        Ok(bytes.len())
    }
}
