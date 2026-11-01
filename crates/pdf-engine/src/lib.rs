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
//!
//! Subphase 5.2.1 Deliverable:
//! - Non-destructive incremental update writer
//! - Multi-revision document history inspector
//! - Digital signature detection & cryptographic ByteRange validator
//!
//! Subphase 5.2.2 Deliverable:
//! - Interactive AcroForms engine (Text, Checkbox, Radio, Choice fields)
//! - Annotations model (Text, Highlight, FreeText, Shapes, Links)
//! - Freehand stylus Ink path layer compiler and appearance generator
//!
//! Subphase 49.1.1 Deliverable:
//! - ISO 32000-2 Vector Path, Image XObject & Font CMap Canvas Renderer
//!
//! Subphase 49.1.2 Deliverable:
//! - Page Split, Merge, Extract, Bates Numbering & Optical Character Recognition (OCR)

#![forbid(unsafe_code)]

pub mod acroform;
pub mod annotation;
pub mod catalog;
pub mod content;
pub mod diff;
pub mod error;
pub mod filters;
pub mod font;
pub mod ink;
pub mod lexer;
pub mod object_stream;
pub mod page;
pub mod page_ops;
pub mod parser;
pub mod renderer;
pub mod revision;
pub mod security_export;
pub mod signature;
pub mod text;
pub mod types;
pub mod writer;
pub mod xref;

pub use acroform::{AcroFormEngine, FormField, FormFieldType, FormFieldValue};
pub use annotation::{AnnotationEngine, AnnotationSubtype, PdfAnnotation};
pub use catalog::PdfDocument;
pub use content::{ContentOperator, ContentParser, TextChunk};
pub use diff::{
    DiffBoundingBox, DiffItemDto, DiffKind, PageDiffDto, PdfDiffEngine, PdfDiffOptions,
    PdfDiffResultDto, PdfDiffSummaryDto,
};
pub use error::PdfError;
pub use filters::predictor::PredictorParams;
pub use filters::FilterPipeline;
pub use font::{FontCMap, PdfFont};
pub use ink::{InkAnnotation, InkStroke};
pub use lexer::{PdfLexer, Token};
pub use object_stream::ObjectStream;
pub use page::PdfPage;
pub use page_ops::{
    apply_bates_numbering, extract_pages, inject_ocr_layer, merge_documents, split_document,
    BatesNumberingConfig, BatesPosition, OcrBoundingBox, OcrPageData, OcrWord, PdfPageOps,
};
pub use parser::PdfParser;
pub use renderer::{
    AffineMatrix, PdfPageRenderer, RenderedImageElementDto, RenderedPageCanvasDto,
    RenderedTextElementDto, RenderedVectorPathDto,
};
pub use revision::{PdfRevision, RevisionHistory};
pub use security_export::{
    PdfRedactionReportDto, PdfRedactionTarget, PdfSecurityExportEngine, PdfSignatureValidationDto,
    PdfSigningOptions, PdfaExportOptions,
};
pub use signature::{PdfSignature, SignatureValidator};
pub use text::{ExtractedPageText, TextGlyph, TextRun};
pub use types::{LinearizationDict, PdfObject, PdfObjectId, PdfRect, PdfStream};
pub use writer::{serialize_object, PdfIncrementalWriter};
pub use xref::{XrefEntry, XrefTable};
