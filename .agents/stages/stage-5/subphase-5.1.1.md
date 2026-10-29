# Subphase 5.1.1: Streaming PDF Object Parser, Linearized XRef & Catalog Tree
- **Stage:** 5 | **Phase:** 5.1
- **Target Crate:** `pdf-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (ISO 32000-1 / ISO 32000-2 Clean-Room Standard)
- **Deliverable:** High-performance streaming lexer, recursive descent object parser, linearized XRef table/stream reader, trailer resolver, and document/page tree hierarchy with inherited attribute resolution.
- **Validation Command:** `cargo test --package pdf-engine --test subphase_5_1_1_test`

---

## Architecture & Requirements (ISO 32000-1 / ISO 32000-2)
1. **Streaming Lexer & Object Model (`crates/pdf-engine`):**
   - Implements PDF 1.0 - 2.0 primitives: `Null`, `Boolean`, `Integer`, `Real`, `Name`, `LiteralString`, `HexString`, `Array`, `Dictionary`, `Stream`, and `IndirectRef`.
   - Streaming tokenization skipping whitespace and comments (`%...`).
   - Literal string escape sequences: `\n`, `\r`, `\t`, `\b`, `\f`, `\(`, `\)`, `\\`, and octal `\ddd`.
   - Hex string parsing `<48656c6c6f>` with automatic whitespace skipping and odd-digit zero padding.
   - Name object decoding with `#xx` hex escape support (`/Name#20With#20Spaces`).
2. **Cross-Reference Table & Stream Reader:**
   - Classic XRef table (`xref`, subsections, byte offsets, generation numbers, in-use `n` / free `f` flags).
   - PDF 1.5+ Compressed Cross-Reference Streams (`/Type /XRef`) decoding Type 0 (free), Type 1 (uncompressed offset), and Type 2 (compressed inside `/ObjStm` object streams).
   - Trailer dictionary resolver (`/Size`, `/Root`, `/Info`, `/Prev`, `/Encrypt`) and backward pointer traversal for incremental updates.
   - Linearization dictionary parser (`/Linearized 1.0`, `/L`, `/H`, `/O`, `/E`, `/N`, `/T`) for sub-millisecond fast first-page catalog lookup.
3. **Document Catalog & Hierarchical Page Tree:**
   - Catalog (`/Root`) traversal to page tree root (`/Pages`).
   - Recursive `/Pages` tree traversal (`/Kids`, `/Count`) resolving inherited page attributes (`/MediaBox`, `/CropBox`, `/Rotate`, `/Resources`).
   - Page object model (`PdfPage`) with geometry bounding box (`PdfRect`), rotation angle, content stream references, and resource dictionaries.
4. **Clean-Room Contract Verification:**
   - Enforce `#[forbid(unsafe_code)]` in `crates/pdf-engine/src/lib.rs`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in non-test library code; return typed `Result<T, PdfError>` with `thiserror`.
   - Full test coverage validating against ISO 32000 fixtures and benchmarks.
