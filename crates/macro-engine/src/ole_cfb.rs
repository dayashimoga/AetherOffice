use crate::error::MacroError;
use std::collections::HashMap;

/// OLE Compound Document magic header bytes (`\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1`).
pub const OLE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Special sector marker constants in [MS-CFB].
pub const SECTOR_ENDOFCHAIN: u32 = 0xFFFFFFFE;
pub const SECTOR_FREESECT: u32 = 0xFFFFFFFF;
pub const NO_STREAM: u32 = 0xFFFFFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Empty = 0,
    Storage = 1,
    Stream = 2,
    RootStorage = 5,
}

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub id: u32,
    pub name: String,
    pub entry_type: EntryType,
    pub starting_sector: u32,
    pub stream_size: u64,
    pub left_sibling: u32,
    pub right_sibling: u32,
    pub child: u32,
}

/// OLE Compound File Binary Format (CFBF / [MS-CFB]) Parser.
#[derive(Debug, Clone)]
pub struct OleCompoundReader {
    sector_size: usize,
    mini_sector_size: usize,
    mini_cutoff_size: u64,
    fat: Vec<u32>,
    minifat: Vec<u32>,
    directory_entries: Vec<DirectoryEntry>,
    mini_stream_data: Vec<u8>,
    stream_map: HashMap<String, u32>,
    raw_data: Vec<u8>,
}

impl OleCompoundReader {
    pub fn new(data: &[u8]) -> Result<Self, MacroError> {
        if data.len() < 512 {
            return Err(MacroError::InvalidOleHeader(
                "Data too short to contain OLE header (minimum 512 bytes)".to_string(),
            ));
        }

        if data[0..8] != OLE_MAGIC {
            return Err(MacroError::InvalidOleHeader(
                "Invalid OLE signature bytes".to_string(),
            ));
        }

        // Byte order check (must be little-endian 0xFFFE)
        let byte_order = u16::from_le_bytes([data[28], data[29]]);
        if byte_order != 0xFFFE {
            return Err(MacroError::InvalidOleHeader(format!(
                "Unsupported endianness marker: 0x{byte_order:04X}"
            )));
        }

        let sector_shift = u16::from_le_bytes([data[30], data[31]]) as usize;
        let mini_sector_shift = u16::from_le_bytes([data[32], data[33]]) as usize;

        if sector_shift != 9 && sector_shift != 12 {
            return Err(MacroError::InvalidOleHeader(format!(
                "Unsupported sector shift: {sector_shift} (expected 9 for 512B or 12 for 4096B)"
            )));
        }

        let sector_size = 1 << sector_shift;
        let mini_sector_size = 1 << mini_sector_shift;

        let fat_sectors_count =
            u32::from_le_bytes([data[44], data[45], data[46], data[47]]) as usize;
        let first_dir_sector = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let mini_cutoff_size = u32::from_le_bytes([data[56], data[57], data[58], data[59]]) as u64;
        let first_minifat_sector = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        let minifat_sectors_count =
            u32::from_le_bytes([data[64], data[65], data[66], data[67]]) as usize;
        let first_difat_sector = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        let difat_sectors_count =
            u32::from_le_bytes([data[72], data[73], data[74], data[75]]) as usize;

        // 1. Collect DIFAT array (first 109 entries are in the header at bytes 76..512)
        let mut difat = Vec::with_capacity(109 + difat_sectors_count * (sector_size / 4 - 1));
        for i in 0..109 {
            let offset = 76 + i * 4;
            if offset + 4 <= 512 {
                let s = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                if s != SECTOR_FREESECT && s != SECTOR_ENDOFCHAIN {
                    difat.push(s);
                }
            }
        }

        // Additional DIFAT sectors if needed
        let mut curr_difat_sector = first_difat_sector;
        while curr_difat_sector != SECTOR_ENDOFCHAIN && curr_difat_sector != SECTOR_FREESECT {
            let s_offset = (curr_difat_sector as usize + 1) * sector_size;
            if s_offset + sector_size > data.len() {
                break;
            }
            let entries_per_sector = sector_size / 4 - 1;
            for i in 0..entries_per_sector {
                let off = s_offset + i * 4;
                let s =
                    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
                if s != SECTOR_FREESECT && s != SECTOR_ENDOFCHAIN {
                    difat.push(s);
                }
            }
            let next_off = s_offset + entries_per_sector * 4;
            curr_difat_sector = u32::from_le_bytes([
                data[next_off],
                data[next_off + 1],
                data[next_off + 2],
                data[next_off + 3],
            ]);
        }

        // 2. Read FAT table
        let mut fat = Vec::new();
        for &fat_sector in &difat {
            if fat.len() >= fat_sectors_count * (sector_size / 4) && fat_sectors_count > 0 {
                break;
            }
            let s_offset = (fat_sector as usize + 1) * sector_size;
            if s_offset + sector_size <= data.len() {
                for i in 0..(sector_size / 4) {
                    let off = s_offset + i * 4;
                    let next = u32::from_le_bytes([
                        data[off],
                        data[off + 1],
                        data[off + 2],
                        data[off + 3],
                    ]);
                    fat.push(next);
                }
            }
        }

        // Helper to read a sector chain from FAT
        let read_fat_chain = |start: u32, fat_table: &[u32]| -> Vec<u8> {
            let mut result = Vec::new();
            let mut curr = start;
            let mut visited = std::collections::HashSet::new();

            while curr != SECTOR_ENDOFCHAIN && curr != SECTOR_FREESECT {
                if !visited.insert(curr) {
                    // Prevent circular loop in corrupted files
                    break;
                }
                let s_offset = (curr as usize + 1) * sector_size;
                if s_offset + sector_size <= data.len() {
                    result.extend_from_slice(&data[s_offset..s_offset + sector_size]);
                } else if s_offset < data.len() {
                    result.extend_from_slice(&data[s_offset..]);
                } else {
                    break;
                }

                if (curr as usize) < fat_table.len() {
                    curr = fat_table[curr as usize];
                } else {
                    break;
                }
            }
            result
        };

        // 3. Read MiniFAT table
        let mut minifat = Vec::new();
        if minifat_sectors_count > 0 && first_minifat_sector != SECTOR_ENDOFCHAIN {
            let minifat_bytes = read_fat_chain(first_minifat_sector, &fat);
            for i in 0..(minifat_bytes.len() / 4) {
                let off = i * 4;
                let next = u32::from_le_bytes([
                    minifat_bytes[off],
                    minifat_bytes[off + 1],
                    minifat_bytes[off + 2],
                    minifat_bytes[off + 3],
                ]);
                minifat.push(next);
            }
        }

        // 4. Read Directory Entries
        let dir_bytes = read_fat_chain(first_dir_sector, &fat);
        let num_entries = dir_bytes.len() / 128;
        let mut directory_entries = Vec::with_capacity(num_entries);

        for i in 0..num_entries {
            let off = i * 128;
            let name_len = u16::from_le_bytes([dir_bytes[off + 64], dir_bytes[off + 65]]) as usize;
            let raw_type = dir_bytes[off + 66];
            let entry_type = match raw_type {
                1 => EntryType::Storage,
                2 => EntryType::Stream,
                5 => EntryType::RootStorage,
                _ => EntryType::Empty,
            };

            let name = if name_len > 2 && name_len <= 64 {
                let u16_len = (name_len - 2) / 2;
                let mut chars = Vec::with_capacity(u16_len);
                for c_idx in 0..u16_len {
                    let c_off = off + c_idx * 2;
                    let ch = u16::from_le_bytes([dir_bytes[c_off], dir_bytes[c_off + 1]]);
                    chars.push(ch);
                }
                String::from_utf16_lossy(&chars)
            } else {
                String::new()
            };

            let left_sibling = u32::from_le_bytes([
                dir_bytes[off + 68],
                dir_bytes[off + 69],
                dir_bytes[off + 70],
                dir_bytes[off + 71],
            ]);
            let right_sibling = u32::from_le_bytes([
                dir_bytes[off + 72],
                dir_bytes[off + 73],
                dir_bytes[off + 74],
                dir_bytes[off + 75],
            ]);
            let child = u32::from_le_bytes([
                dir_bytes[off + 76],
                dir_bytes[off + 77],
                dir_bytes[off + 78],
                dir_bytes[off + 79],
            ]);
            let starting_sector = u32::from_le_bytes([
                dir_bytes[off + 116],
                dir_bytes[off + 117],
                dir_bytes[off + 118],
                dir_bytes[off + 119],
            ]);
            let stream_size = u64::from_le_bytes([
                dir_bytes[off + 120],
                dir_bytes[off + 121],
                dir_bytes[off + 122],
                dir_bytes[off + 123],
                dir_bytes[off + 124],
                dir_bytes[off + 125],
                dir_bytes[off + 126],
                dir_bytes[off + 127],
            ]);

            directory_entries.push(DirectoryEntry {
                id: i as u32,
                name,
                entry_type,
                starting_sector,
                stream_size,
                left_sibling,
                right_sibling,
                child,
            });
        }

        // 5. Mini stream container is defined by root storage entry (entry 0)
        let mini_stream_data = if let Some(root) = directory_entries.first() {
            if root.entry_type == EntryType::RootStorage
                && root.starting_sector != SECTOR_ENDOFCHAIN
            {
                read_fat_chain(root.starting_sector, &fat)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // 6. Build directory hierarchy stream map
        let mut stream_map = HashMap::new();
        if !directory_entries.is_empty() {
            let mut stack = Vec::new();
            if directory_entries[0].child != NO_STREAM {
                stack.push((directory_entries[0].child, String::new()));
            }

            while let Some((curr_id, path_prefix)) = stack.pop() {
                if let Some(entry) = directory_entries.get(curr_id as usize) {
                    if entry.entry_type == EntryType::Empty {
                        continue;
                    }
                    let full_path = if path_prefix.is_empty() {
                        entry.name.clone()
                    } else {
                        format!("{}/{}", path_prefix, entry.name)
                    };

                    if entry.entry_type == EntryType::Stream {
                        stream_map.insert(full_path.clone(), curr_id);
                        // Also index case-insensitively and by basename for ease of lookup
                        stream_map.insert(full_path.to_ascii_lowercase(), curr_id);
                        stream_map.insert(entry.name.to_ascii_lowercase(), curr_id);
                    }

                    if entry.child != NO_STREAM {
                        stack.push((entry.child, full_path));
                    }
                    if entry.left_sibling != NO_STREAM {
                        stack.push((entry.left_sibling, path_prefix.clone()));
                    }
                    if entry.right_sibling != NO_STREAM {
                        stack.push((entry.right_sibling, path_prefix));
                    }
                }
            }
        }

        Ok(Self {
            sector_size,
            mini_sector_size,
            mini_cutoff_size,
            fat,
            minifat,
            directory_entries,
            mini_stream_data,
            stream_map,
            raw_data: data.to_vec(),
        })
    }

    /// Lists all discovered stream paths in the OLE compound file.
    pub fn list_streams(&self) -> Vec<String> {
        self.directory_entries
            .iter()
            .filter(|e| e.entry_type == EntryType::Stream && !e.name.is_empty())
            .map(|e| e.name.clone())
            .collect()
    }

    /// Checks if a stream exists by name or full path.
    pub fn has_stream(&self, path: &str) -> bool {
        self.stream_map.contains_key(&path.to_ascii_lowercase())
    }

    /// Reads raw decompressed bytes of an OLE stream by name or path.
    pub fn read_stream(&self, path: &str) -> Result<Vec<u8>, MacroError> {
        let key = path.to_ascii_lowercase();
        let entry_id = self
            .stream_map
            .get(&key)
            .copied()
            .ok_or_else(|| MacroError::StreamNotFound(path.to_string()))?;

        let entry = self
            .directory_entries
            .get(entry_id as usize)
            .ok_or_else(|| MacroError::StreamNotFound(path.to_string()))?;

        if entry.stream_size == 0 {
            return Ok(Vec::new());
        }

        // Small streams (< 4096 bytes) are stored in the mini stream
        if entry.stream_size < self.mini_cutoff_size && !self.minifat.is_empty() {
            let mut result = Vec::with_capacity(entry.stream_size as usize);
            let mut curr = entry.starting_sector;
            let mut visited = std::collections::HashSet::new();

            while curr != SECTOR_ENDOFCHAIN && curr != SECTOR_FREESECT {
                if !visited.insert(curr) {
                    break;
                }
                let mini_off = curr as usize * self.mini_sector_size;
                if mini_off + self.mini_sector_size <= self.mini_stream_data.len() {
                    result.extend_from_slice(
                        &self.mini_stream_data[mini_off..mini_off + self.mini_sector_size],
                    );
                } else if mini_off < self.mini_stream_data.len() {
                    result.extend_from_slice(&self.mini_stream_data[mini_off..]);
                } else {
                    break;
                }

                if result.len() >= entry.stream_size as usize {
                    break;
                }

                if (curr as usize) < self.minifat.len() {
                    curr = self.minifat[curr as usize];
                } else {
                    break;
                }
            }

            result.truncate(entry.stream_size as usize);
            Ok(result)
        } else {
            // Regular stream in FAT
            let mut result = Vec::with_capacity(entry.stream_size as usize);
            let mut curr = entry.starting_sector;
            let mut visited = std::collections::HashSet::new();

            while curr != SECTOR_ENDOFCHAIN && curr != SECTOR_FREESECT {
                if !visited.insert(curr) {
                    break;
                }
                let s_offset = (curr as usize + 1) * self.sector_size;
                if s_offset + self.sector_size <= self.raw_data.len() {
                    result.extend_from_slice(&self.raw_data[s_offset..s_offset + self.sector_size]);
                } else if s_offset < self.raw_data.len() {
                    result.extend_from_slice(&self.raw_data[s_offset..]);
                } else {
                    break;
                }

                if result.len() >= entry.stream_size as usize {
                    break;
                }

                if (curr as usize) < self.fat.len() {
                    curr = self.fat[curr as usize];
                } else {
                    break;
                }
            }

            result.truncate(entry.stream_size as usize);
            Ok(result)
        }
    }
}
