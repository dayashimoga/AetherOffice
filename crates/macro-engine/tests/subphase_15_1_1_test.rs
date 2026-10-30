use macro_engine::{
    compress_vba_stream, copy_token_offset_bit_count, decompress_vba_stream, MacroError,
    OleCompoundReader, VbaModuleType, VbaProject, OLE_MAGIC,
};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Helper to build a minimal valid OLE Compound Document in memory.
/// Supports both MiniFAT small streams (< 4096 bytes) and FAT large streams (>= 4096 bytes).
fn build_synthetic_cfb(streams: &[(&str, &[u8], bool)], // (name, data, is_mini)
) -> Vec<u8> {
    const SECTOR_SIZE: usize = 512;
    const MINI_SECTOR_SIZE: usize = 64;
    const SECTOR_ENDOFCHAIN: u32 = 0xFFFFFFFE;
    const SECTOR_FREESECT: u32 = 0xFFFFFFFF;

    // Separate mini streams from regular streams
    let mut mini_stream_data = Vec::new();
    let mut minifat: Vec<u32> = Vec::new();

    // Map each stream to its starting sector and size
    struct StreamAlloc {
        name: String,
        start_sector: u32,
        size: u64,
    }

    let mut allocations = Vec::new();

    // 1. Pack mini streams into mini_stream_data and minifat
    for &(name, data, is_mini) in streams {
        if is_mini {
            let start_mini_sector = (mini_stream_data.len() / MINI_SECTOR_SIZE) as u32;
            let sectors_needed = if data.is_empty() {
                1
            } else {
                data.len().div_ceil(MINI_SECTOR_SIZE)
            };

            for s in 0..sectors_needed {
                if s + 1 == sectors_needed {
                    minifat.push(SECTOR_ENDOFCHAIN);
                } else {
                    minifat.push(start_mini_sector + s as u32 + 1);
                }
            }

            mini_stream_data.extend_from_slice(data);
            let pad = (sectors_needed * MINI_SECTOR_SIZE).saturating_sub(data.len());
            mini_stream_data.extend(std::iter::repeat_n(0u8, pad));

            allocations.push(StreamAlloc {
                name: name.to_string(),
                start_sector: start_mini_sector,
                size: data.len() as u64,
            });
        }
    }

    // Pad minifat to full sector (128 entries = 512 bytes)
    while minifat.len() < SECTOR_SIZE / 4 {
        minifat.push(SECTOR_FREESECT);
    }

    // 2. Determine regular FAT sectors layout:
    // Sector 0: Directory Sector
    // Sector 1: MiniFAT Sector (if mini streams exist)
    // Sector 2: FAT Sector
    // Sector 3..: Mini-stream container data (if mini streams exist)
    // Sector K..: Regular streams data
    let mut fat: Vec<u32> = vec![SECTOR_FREESECT; SECTOR_SIZE / 4];
    fat[0] = SECTOR_ENDOFCHAIN; // Sector 0 is Directory
    fat[1] = SECTOR_ENDOFCHAIN; // Sector 1 is MiniFAT
    fat[2] = 0xFFFFFFFD; // Sector 2 is FAT

    let mut current_sec = 3u32;
    let mut mini_stream_start_sec = SECTOR_ENDOFCHAIN;

    if !mini_stream_data.is_empty() {
        mini_stream_start_sec = current_sec;
        let mini_sectors_needed = mini_stream_data.len().div_ceil(SECTOR_SIZE);
        for s in 0..mini_sectors_needed {
            let sec_idx = current_sec as usize;
            if s + 1 == mini_sectors_needed {
                fat[sec_idx] = SECTOR_ENDOFCHAIN;
            } else {
                fat[sec_idx] = current_sec + 1;
            }
            current_sec += 1;
        }
    }

    let mut regular_stream_sectors = Vec::new();
    for &(name, data, is_mini) in streams {
        if !is_mini {
            let start = current_sec;
            let sectors_needed = if data.is_empty() {
                1
            } else {
                data.len().div_ceil(SECTOR_SIZE)
            };

            for s in 0..sectors_needed {
                let sec_idx = current_sec as usize;
                if s + 1 == sectors_needed {
                    fat[sec_idx] = SECTOR_ENDOFCHAIN;
                } else {
                    fat[sec_idx] = current_sec + 1;
                }
                current_sec += 1;
            }

            regular_stream_sectors.push((start, data.to_vec(), sectors_needed));
            allocations.push(StreamAlloc {
                name: name.to_string(),
                start_sector: start,
                size: data.len() as u64,
            });
        }
    }

    // 3. Build Directory Sector (Sector 0)
    let mut dir_sector = vec![0u8; SECTOR_SIZE];
    // Entry 0: Root Entry
    let root_entry_slice = &mut dir_sector[0..128];
    write_dir_entry(
        root_entry_slice,
        "Root Entry",
        5, // RootStorage
        SECTOR_FREESECT,
        SECTOR_FREESECT,
        if allocations.is_empty() {
            SECTOR_FREESECT
        } else {
            1
        },
        mini_stream_start_sec,
        mini_stream_data.len() as u64,
    );

    // Entries 1..N: Stream entries
    for (i, alloc) in allocations.iter().enumerate() {
        let entry_id = i + 1;
        if entry_id * 128 + 128 <= dir_sector.len() {
            let slice = &mut dir_sector[entry_id * 128..entry_id * 128 + 128];
            let right_sibling = if entry_id < allocations.len() {
                (entry_id + 1) as u32
            } else {
                SECTOR_FREESECT
            };
            write_dir_entry(
                slice,
                &alloc.name,
                2, // Stream
                SECTOR_FREESECT,
                right_sibling,
                SECTOR_FREESECT,
                alloc.start_sector,
                alloc.size,
            );
        }
    }

    // 4. Build MiniFAT Sector (Sector 1)
    let mut minifat_sector = vec![0u8; SECTOR_SIZE];
    for (i, &entry) in minifat.iter().enumerate() {
        minifat_sector[i * 4..i * 4 + 4].copy_from_slice(&entry.to_le_bytes());
    }

    // 5. Build FAT Sector (Sector 2)
    let mut fat_sector = vec![0u8; SECTOR_SIZE];
    for (i, &entry) in fat.iter().enumerate() {
        fat_sector[i * 4..i * 4 + 4].copy_from_slice(&entry.to_le_bytes());
    }

    // 6. Build Header
    let mut header = vec![0u8; SECTOR_SIZE];
    header[0..8].copy_from_slice(&OLE_MAGIC);
    header[24..26].copy_from_slice(&0x003Eu16.to_le_bytes()); // minor version
    header[26..28].copy_from_slice(&0x0003u16.to_le_bytes()); // major version
    header[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes()); // byte order (little-endian)
    header[30..32].copy_from_slice(&9u16.to_le_bytes()); // sector shift 9 -> 512
    header[32..34].copy_from_slice(&6u16.to_le_bytes()); // mini sector shift 6 -> 64
    header[44..48].copy_from_slice(&1u32.to_le_bytes()); // FAT sectors count
    header[48..52].copy_from_slice(&0u32.to_le_bytes()); // First dir sector = 0
    header[56..60].copy_from_slice(&4096u32.to_le_bytes()); // Mini cutoff size = 4096
    header[60..64].copy_from_slice(&1u32.to_le_bytes()); // First MiniFAT sector = 1
    header[64..68].copy_from_slice(&1u32.to_le_bytes()); // MiniFAT sectors count = 1
    header[68..72].copy_from_slice(&SECTOR_ENDOFCHAIN.to_le_bytes()); // First DIFAT sector
    header[72..76].copy_from_slice(&0u32.to_le_bytes()); // DIFAT sectors count

    // DIFAT[0] points to sector 2 (FAT sector)
    header[76..80].copy_from_slice(&2u32.to_le_bytes());
    for i in 1..109 {
        let off = 76 + i * 4;
        header[off..off + 4].copy_from_slice(&SECTOR_FREESECT.to_le_bytes());
    }

    // 7. Assemble file bytes
    let mut file_bytes = Vec::new();
    file_bytes.extend_from_slice(&header);
    file_bytes.extend_from_slice(&dir_sector); // Sector 0
    file_bytes.extend_from_slice(&minifat_sector); // Sector 1
    file_bytes.extend_from_slice(&fat_sector); // Sector 2

    // Sector 3.. (Mini-stream container)
    if !mini_stream_data.is_empty() {
        let total_pad = mini_stream_data.len().div_ceil(SECTOR_SIZE) * SECTOR_SIZE;
        file_bytes.extend_from_slice(&mini_stream_data);
        file_bytes.extend(std::iter::repeat_n(0u8, total_pad - mini_stream_data.len()));
    }

    // Regular stream data sectors
    for (_, data, sectors_needed) in regular_stream_sectors {
        let padded_size = sectors_needed * SECTOR_SIZE;
        file_bytes.extend_from_slice(&data);
        file_bytes.extend(std::iter::repeat_n(
            0u8,
            padded_size.saturating_sub(data.len()),
        ));
    }

    file_bytes
}

#[allow(clippy::too_many_arguments)]
fn write_dir_entry(
    dest: &mut [u8],
    name: &str,
    entry_type: u8,
    left: u32,
    right: u32,
    child: u32,
    start_sec: u32,
    size: u64,
) {
    let utf16: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let name_byte_len = usize::min(utf16.len() * 2, 64);
    for (i, &ch) in utf16.iter().enumerate() {
        if i * 2 + 1 < 64 {
            let bytes = ch.to_le_bytes();
            dest[i * 2] = bytes[0];
            dest[i * 2 + 1] = bytes[1];
        }
    }
    dest[64] = (name_byte_len & 0xFF) as u8;
    dest[65] = ((name_byte_len >> 8) & 0xFF) as u8;
    dest[66] = entry_type;
    dest[68..72].copy_from_slice(&left.to_le_bytes());
    dest[72..76].copy_from_slice(&right.to_le_bytes());
    dest[76..80].copy_from_slice(&child.to_le_bytes());
    dest[116..120].copy_from_slice(&start_sec.to_le_bytes());
    dest[120..128].copy_from_slice(&size.to_le_bytes());
}

#[test]
fn test_ole_invalid_header_rejection() {
    // Empty data
    let res = OleCompoundReader::new(&[]);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), MacroError::InvalidOleHeader(_)));

    // Short data < 512 bytes
    let res = OleCompoundReader::new(&[0u8; 256]);
    assert!(res.is_err());

    // Bad magic
    let mut bad_magic = vec![0u8; 512];
    bad_magic[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let res = OleCompoundReader::new(&bad_magic);
    assert!(res.is_err());

    // Bad sector shift (e.g. 7 instead of 9 or 12)
    let mut bad_shift = vec![0u8; 512];
    bad_shift[0..8].copy_from_slice(&OLE_MAGIC);
    bad_shift[28..30].copy_from_slice(&0xFFFEu16.to_le_bytes());
    bad_shift[30..32].copy_from_slice(&7u16.to_le_bytes()); // invalid shift
    let res = OleCompoundReader::new(&bad_shift);
    assert!(res.is_err());
}

#[test]
fn test_ovba_copy_token_offset_bit_count() {
    assert_eq!(copy_token_offset_bit_count(1), 4);
    assert_eq!(copy_token_offset_bit_count(16), 4);
    assert_eq!(copy_token_offset_bit_count(17), 5);
    assert_eq!(copy_token_offset_bit_count(32), 5);
    assert_eq!(copy_token_offset_bit_count(33), 6);
    assert_eq!(copy_token_offset_bit_count(64), 6);
    assert_eq!(copy_token_offset_bit_count(65), 7);
    assert_eq!(copy_token_offset_bit_count(128), 7);
    assert_eq!(copy_token_offset_bit_count(129), 8);
    assert_eq!(copy_token_offset_bit_count(256), 8);
    assert_eq!(copy_token_offset_bit_count(257), 9);
    assert_eq!(copy_token_offset_bit_count(512), 9);
    assert_eq!(copy_token_offset_bit_count(513), 10);
    assert_eq!(copy_token_offset_bit_count(1024), 10);
    assert_eq!(copy_token_offset_bit_count(1025), 11);
    assert_eq!(copy_token_offset_bit_count(2048), 11);
    assert_eq!(copy_token_offset_bit_count(2049), 12);
    assert_eq!(copy_token_offset_bit_count(4096), 12);
}

#[test]
fn test_ovba_uncompressed_chunk_roundtrip() {
    let source = "Attribute VB_Name = \"Sheet1\"\nSub Test()\n  MsgBox \"Hello VBA\"\nEnd Sub\n";
    let compressed = compress_vba_stream(source.as_bytes());
    assert_eq!(compressed[0], 0x01); // MS-OVBA signature

    let decompressed = decompress_vba_stream(&compressed).unwrap();
    let text = String::from_utf8(decompressed).unwrap();
    assert_eq!(text, source);
}

#[test]
fn test_ovba_compressed_lz77_roundtrip() {
    let mut source = String::new();
    for i in 0..50 {
        source.push_str(&format!(
            "Dim variable_{i} As Long: variable_{i} = {i} * 100\n"
        ));
    }

    let compressed = compress_vba_stream(source.as_bytes());
    assert_eq!(compressed[0], 0x01);
    // Highly repetitive text should compress
    assert!(compressed.len() < source.len());

    let decompressed = decompress_vba_stream(&compressed).unwrap();
    let text = String::from_utf8(decompressed).unwrap();
    assert_eq!(text, source);
}

#[test]
fn test_ovba_multichunk_compression() {
    let mut source = String::new();
    // > 8KB of code to span across at least 3 chunks (chunk max 4096)
    for i in 0..300 {
        source.push_str(&format!(
            "Public Function ComputeTotal_{i}(ByVal arg1 As Double, ByVal arg2 As Double) As Double\n    ComputeTotal_{i} = arg1 * arg2 + {i}.5\nEnd Function\n"
        ));
    }
    assert!(source.len() > 8192);

    let compressed = compress_vba_stream(source.as_bytes());
    assert_eq!(compressed[0], 0x01);

    let decompressed = decompress_vba_stream(&compressed).unwrap();
    let text = String::from_utf8(decompressed).unwrap();
    assert_eq!(text, source);
}

#[test]
fn test_ole_cfb_synthetic_compound_file() {
    let mini_data = b"Hello Mini Stream Data in CFB";
    let fat_data = vec![0xAA; 512]; // exactly 1 sector

    let cfb_bytes = build_synthetic_cfb(&[
        ("StreamMini", mini_data, true),
        ("StreamFat", &fat_data, false),
    ]);

    let reader = OleCompoundReader::new(&cfb_bytes).unwrap();
    let streams = reader.list_streams();
    assert!(streams.contains(&"StreamMini".to_string()));
    assert!(streams.contains(&"StreamFat".to_string()));

    assert!(reader.has_stream("streammini"));
    assert!(reader.has_stream("STREAMFAT"));
    assert!(!reader.has_stream("NonExistent"));
}

#[test]
fn test_ole_cfb_small_stream_minifat() {
    let small_content = b"Sub SmallVbaSnippet()\n  Dim x As Integer\n  x = 10\nEnd Sub\n";
    let cfb_bytes = build_synthetic_cfb(&[("SmallSnippet", small_content, true)]);

    let reader = OleCompoundReader::new(&cfb_bytes).unwrap();
    let read_back = reader.read_stream("SmallSnippet").unwrap();
    assert_eq!(read_back, small_content);
}

#[test]
fn test_ole_cfb_large_stream_fat() {
    // 5000 bytes spans multiple 512-byte sectors (10 sectors)
    let mut large_content = Vec::with_capacity(5000);
    for i in 0..5000 {
        large_content.push((i % 251) as u8);
    }

    let cfb_bytes = build_synthetic_cfb(&[("LargeFatStream", &large_content, false)]);

    let reader = OleCompoundReader::new(&cfb_bytes).unwrap();
    let read_back = reader.read_stream("LargeFatStream").unwrap();
    assert_eq!(read_back.len(), 5000);
    assert_eq!(read_back, large_content);
}

#[test]
fn test_vba_project_parsing_with_modules() {
    let mod1_code = "Attribute VB_Name = \"Module1\"\nSub Run()\n  MsgBox 42\nEnd Sub\n";
    let mod1_compressed = compress_vba_stream(mod1_code.as_bytes());

    let thiswb_code =
        "Attribute VB_Name = \"ThisWorkbook\"\nPrivate Sub Workbook_Open()\nEnd Sub\n";
    let thiswb_compressed = compress_vba_stream(thiswb_code.as_bytes());

    // Build synthetic dir stream records ([MS-OVBA] 2.3.4.2)
    let mut dir_raw = Vec::new();
    // PROJECTNAME record (0x0004)
    let proj_name = b"FinanceMacros";
    dir_raw.extend_from_slice(&0x0004u16.to_le_bytes());
    dir_raw.extend_from_slice(&(proj_name.len() as u32).to_le_bytes());
    dir_raw.extend_from_slice(proj_name);

    // PROJECTCODEPAGE record (0x0003)
    dir_raw.extend_from_slice(&0x0003u16.to_le_bytes());
    dir_raw.extend_from_slice(&2u32.to_le_bytes());
    dir_raw.extend_from_slice(&1252u16.to_le_bytes());

    // MODULE 1: Module1 (Procedural)
    let mod1_name = b"Module1";
    dir_raw.extend_from_slice(&0x0019u16.to_le_bytes());
    dir_raw.extend_from_slice(&(mod1_name.len() as u32).to_le_bytes());
    dir_raw.extend_from_slice(mod1_name);

    dir_raw.extend_from_slice(&0x0022u16.to_le_bytes()); // Procedural
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    dir_raw.extend_from_slice(&0x0031u16.to_le_bytes()); // MODULEOFFSET = 0
    dir_raw.extend_from_slice(&4u32.to_le_bytes());
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    // MODULE 2: ThisWorkbook (Document)
    let thiswb_name = b"ThisWorkbook";
    dir_raw.extend_from_slice(&0x0019u16.to_le_bytes());
    dir_raw.extend_from_slice(&(thiswb_name.len() as u32).to_le_bytes());
    dir_raw.extend_from_slice(thiswb_name);

    dir_raw.extend_from_slice(&0x0021u16.to_le_bytes()); // Document
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    dir_raw.extend_from_slice(&0x0031u16.to_le_bytes()); // MODULEOFFSET = 0
    dir_raw.extend_from_slice(&4u32.to_le_bytes());
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    // Terminating record
    dir_raw.extend_from_slice(&0x0010u16.to_le_bytes());
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    let dir_compressed = compress_vba_stream(&dir_raw);

    let cfb_bytes = build_synthetic_cfb(&[
        ("dir", &dir_compressed, true),
        ("Module1", &mod1_compressed, true),
        ("ThisWorkbook", &thiswb_compressed, true),
    ]);

    let project = VbaProject::from_ole_bytes(&cfb_bytes).unwrap();
    assert_eq!(project.name, "FinanceMacros");
    assert_eq!(project.code_page, 1252);
    assert_eq!(project.modules.len(), 2);

    let m1 = project
        .modules
        .iter()
        .find(|m| m.name == "Module1")
        .unwrap();
    assert_eq!(m1.module_type, VbaModuleType::Procedural);
    assert!(m1.source_code.contains("Sub Run()"));

    let m2 = project
        .modules
        .iter()
        .find(|m| m.name == "ThisWorkbook")
        .unwrap();
    assert_eq!(m2.module_type, VbaModuleType::Document);
    assert!(m2.source_code.contains("Workbook_Open()"));
}

#[test]
fn test_ooxml_vba_project_bin_integration() {
    let source_code = "Sub AutoCalculate()\n  ActiveSheet.Calculate\nEnd Sub\n";
    let compressed_mod = compress_vba_stream(source_code.as_bytes());

    let mut dir_raw = Vec::new();
    let mod_name = b"AutoMacro";
    dir_raw.extend_from_slice(&0x0019u16.to_le_bytes());
    dir_raw.extend_from_slice(&(mod_name.len() as u32).to_le_bytes());
    dir_raw.extend_from_slice(mod_name);
    dir_raw.extend_from_slice(&0x0031u16.to_le_bytes());
    dir_raw.extend_from_slice(&4u32.to_le_bytes());
    dir_raw.extend_from_slice(&0u32.to_le_bytes());
    dir_raw.extend_from_slice(&0x0010u16.to_le_bytes());
    dir_raw.extend_from_slice(&0u32.to_le_bytes());

    let compressed_dir = compress_vba_stream(&dir_raw);

    let cfb_bytes = build_synthetic_cfb(&[
        ("dir", &compressed_dir, true),
        ("AutoMacro", &compressed_mod, true),
    ]);

    // Create an OOXML (.xlsm) package in memory
    let mut zip_buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut zip_buf));
        let options = SimpleFileOptions::default();

        zip.start_file("xl/vbaProject.bin", options).unwrap();
        zip.write_all(&cfb_bytes).unwrap();

        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.finish().unwrap();
    }

    // Extract xl/vbaProject.bin from the zip buffer
    let mut reader = zip::ZipArchive::new(Cursor::new(&zip_buf)).unwrap();
    let mut vba_bin_file = reader.by_name("xl/vbaProject.bin").unwrap();
    let mut vba_bin_bytes = Vec::new();
    std::io::Read::read_to_end(&mut vba_bin_file, &mut vba_bin_bytes).unwrap();

    // Parse extracted vbaProject.bin with VbaProject
    let project = VbaProject::from_ole_bytes(&vba_bin_bytes).unwrap();
    assert_eq!(project.modules.len(), 1);
    assert_eq!(project.modules[0].name, "AutoMacro");
    assert!(project.modules[0].source_code.contains("AutoCalculate()"));
}
