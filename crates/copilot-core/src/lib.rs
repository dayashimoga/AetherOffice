#![forbid(unsafe_code)]

pub mod chunker;
pub mod embedding;
pub mod error;
pub mod tokenizer;
pub mod vector_index;

pub use chunker::{
    ChunkerConfig, DocumentChunk, DocumentKind, DocumentLocation, SemanticChunker, SlideInput,
    SlideShapeInput, WordParagraphInput,
};
pub use embedding::{
    cosine_similarity, cosine_similarity_quantized, LocalEmbeddingGenerator, QuantizedVector,
    EMBEDDING_DIM,
};
pub use error::CopilotError;
pub use tokenizer::{
    LocalTokenizer, CLS_TOKEN_ID, MASK_TOKEN_ID, PAD_TOKEN_ID, SEP_TOKEN_ID, UNK_TOKEN_ID,
};
pub use vector_index::{IndexEntry, SearchResult, VectorIndex};
