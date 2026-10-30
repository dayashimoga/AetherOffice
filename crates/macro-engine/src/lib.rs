#![forbid(unsafe_code)]

pub mod error;
pub mod ole_cfb;
pub mod ovba_decompress;
pub mod vba_project;

pub use error::MacroError;
pub use ole_cfb::{DirectoryEntry, EntryType, OleCompoundReader, OLE_MAGIC};
pub use ovba_decompress::{
    compress_vba_stream, copy_token_offset_bit_count, decompress_vba_stream,
};
pub use vba_project::{VbaModule, VbaModuleType, VbaProject};
