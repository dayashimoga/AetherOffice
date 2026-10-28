#![forbid(unsafe_code)]
//! ooxml-opc - ECMA-376 / Clean-Room Engine Module

pub mod error;
pub use error::{EngineError, Result};

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

    #[test]
    fn test_error_display_and_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let engine_err = EngineError::from(io_err);
        assert!(format!("{engine_err}").contains("file missing"));

        let custom_err = EngineError::PartNotFound("word/document.xml".to_string());
        assert_eq!(
            format!("{custom_err}"),
            "OPC Part not found: word/document.xml"
        );
    }
}
