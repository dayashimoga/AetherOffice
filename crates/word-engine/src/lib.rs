#![forbid(unsafe_code)]
//! word-engine - ECMA-376 / Clean-Room Engine Module

pub mod layout;
pub mod model;
pub mod numbering;
pub mod package;
pub mod parser;
pub mod piece_tree;
pub mod section;
pub mod serializer;
pub mod styles;
pub mod sub_documents;

pub use layout::*;
pub use model::*;
pub use numbering::*;
pub use package::WordPackage;
pub use parser::*;
pub use piece_tree::PieceTree;
pub use section::*;
pub use serializer::*;
pub use styles::*;
pub use sub_documents::*;

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
