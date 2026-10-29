//! OpenOfficeRS PDF Engine: ISO 32000-1 / ISO 32000-2 Certified PDF Parser & Model
//!
//! Subphase 5.1.1 Deliverable:
//! - Streaming PDF object tokenizer & parser
//! - Linearized XRef & compressed object stream reader
//! - Document Catalog & hierarchical page tree resolver with attribute inheritance
//!
//! Subphase 5.1.2 Deliverable:
//! - Stream filter pipeline (ASCIIHex, ASCII85, Flate with PNG/TIFF predictors)
//! - Content stream operator interpreter
//! - Font CMap (/ToUnicode & standard encodings) & structured text extraction engine

#![forbid(unsafe_code)]

pub mod catalog;
pub mod content;
pub mod error;
pub mod filters;
pub mod font;
pub mod lexer;
pub mod object_stream;
pub mod page;
pub mod parser;
pub mod text;
pub mod types;
pub mod xref;

pub use catalog::PdfDocument;
pub use content::{ContentOperator, ContentParser, TextChunk};
pub use error::PdfError;
pub use filters::predictor::PredictorParams;
pub use filters::FilterPipeline;
pub use font::{FontCMap, PdfFont};
pub use lexer::{PdfLexer, Token};
pub use object_stream::ObjectStream;
pub use page::PdfPage;
pub use parser::PdfParser;
pub use text::{ExtractedPageText, TextGlyph, TextRun};
pub use types::{LinearizationDict, PdfObject, PdfObjectId, PdfRect, PdfStream};
pub use xref::{XrefEntry, XrefTable};
