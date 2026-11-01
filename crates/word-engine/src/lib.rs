#![forbid(unsafe_code)]
//! word-engine - ECMA-376 / Clean-Room Engine Module

pub mod citations;
pub mod comments;
pub mod layout;
pub mod mail_merge;
pub mod math;
pub mod model;
pub mod numbering;
pub mod odt_codec;
pub mod package;
pub mod parser;
pub mod piece_tree;
pub mod revision;
pub mod rtf_codec;
pub mod section;
pub mod serializer;
pub mod styles;
pub mod sub_documents;
pub mod table_layout;
pub mod toc;

pub use citations::*;
pub use comments::*;
pub use layout::*;
pub use mail_merge::*;
pub use math::*;
pub use model::*;
pub use numbering::*;
pub use odt_codec::*;
pub use package::WordPackage;
pub use parser::*;
pub use piece_tree::PieceTree;
pub use revision::*;
pub use rtf_codec::*;
pub use section::*;
pub use serializer::*;
pub use styles::*;
pub use sub_documents::*;
pub use table_layout::*;
pub use toc::*;

pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
