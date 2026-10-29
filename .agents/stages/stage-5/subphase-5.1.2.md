# Subphase 5.1.2: Content Stream Filter Decompression & Font CMap Unicode Resolver
- **Stage:** 5 | **Phase:** 5.1
- **Target Crate:** `pdf-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (ISO 32000-1 / ISO 32000-2 Clean-Room Standard)
- **Deliverable:** Stream filter pipeline (ASCIIHex, ASCII85, Flate with PNG/TIFF predictors), content stream operator interpreter, and Font CMap (/ToUnicode & standard encodings) text extraction engine.
- **Validation Command:** `cargo test --package pdf-engine --test subphase_5_1_2_test`

---

## Architecture & Requirements (ISO 32000-1 Clause 7.4, 9.6-9.10)
1. **Stream Filter Pipeline (`crates/pdf-engine/src/filters/`):**
   - `/ASCIIHexDecode`: Decode hex character sequences ignoring whitespace.
   - `/ASCII85Decode`: Base85 decoding handling 5-character tuples and `z` zero abbreviation.
   - `/FlateDecode` with Predictor functions:
     - PNG Predictors (Sub, Up, Average, Paeth) and TIFF Predictor 2.
     - Decode parameters: `/Predictor`, `/Colors`, `/BitsPerComponent`, `/Columns`.
   - Multi-filter chaining (decoding sequential array of filters).
2. **Content Stream Lexer & Operator Parser (`crates/pdf-engine/src/content.rs`):**
   - Graphics state stack (`q`, `Q`, `cm`, `gs`).
   - Color operators (`g`/`G`, `rg`/`RG`, `k`/`K`).
   - Text state operators (`BT`, `ET`, `Tf`, `Tc`, `Tw`, `Tz`, `TL`, `Tr`).
   - Text positioning & matrices (`Td`, `TD`, `Tm`, `T*`).
   - Text showing operators (`Tj`, `'`, `"`, `TJ` with kerning/tracking adjustments).
3. **Font CMap & Unicode Resolver (`crates/pdf-engine/src/font.rs`):**
   - PostScript CMap parser for `/ToUnicode` streams (`beginbfchar`/`endbfchar`, `beginbfrange`/`endbfrange`, array ranges, surrogate pairs).
   - Standard PDF encodings fallback (`/WinAnsiEncoding`, `/StandardEncoding`, `/MacRomanEncoding`).
   - Font dictionary resolution from `/Resources` (`/BaseFont`, `/Subtype`, `/Widths`, `/FontDescriptor`).
4. **Text Extraction & Layout Model (`crates/pdf-engine/src/text.rs`):**
   - `TextGlyph` and `TextRun` models with bounding boxes, font sizing, and RGBA styling.
   - Plain text extraction reconstructing paragraph breaks and word spaces based on spatial coordinates.
5. **Clean-Room Invariants:**
   - `#[forbid(unsafe_code)]` enforced. Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in non-test paths.
