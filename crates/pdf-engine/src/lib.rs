//! OpenOfficeRS PDF Engine: ISO 32000-1 / ISO 32000-2 Certified PDF Parser & Model
//!
//! Subphase 5.1.1 Deliverable:
//! - Streaming PDF object tokenizer & parser
//! - Linearized XRef & compressed object stream reader
//! - Document Catalog & hierarchical page tree resolver with attribute inheritance

#![forbid(unsafe_code)]

pub mod catalog;
pub mod error;
pub mod lexer;
pub mod object_stream;
pub mod page;
pub mod parser;
pub mod types;
pub mod xref;

pub use catalog::PdfDocument;
pub use error::PdfError;
pub use lexer::{PdfLexer, Token};
pub use object_stream::ObjectStream;
pub use page::PdfPage;
pub use parser::PdfParser;
pub use types::{LinearizationDict, PdfObject, PdfObjectId, PdfRect, PdfStream};
pub use xref::{XrefEntry, XrefTable};
