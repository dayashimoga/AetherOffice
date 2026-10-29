#![forbid(unsafe_code)]
//! word-engine - ECMA-376 / Clean-Room Engine Module

pub mod model;
pub mod package;
pub mod parser;
pub mod piece_tree;
pub mod serializer;
pub mod sub_documents;

pub use model::*;
pub use package::WordPackage;
pub use parser::*;
pub use piece_tree::PieceTree;
pub use serializer::*;
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
