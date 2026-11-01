# Sub-Phase 46.2.2: Mail Merge, OMML/LaTeX Equations & .odt/.rtf Import/Export

## Objective
Implement industrial-grade Mail Merge (MERGEFIELD with CSV/Record mapping), Office Math Markup Language (OMML) & bidirectional LaTeX equation conversion, and clean-room OpenDocument Text (.odt) and Rich Text Format (.rtf) codecs in `word-engine` and the unified presentation shell.

## Strict Engineering Invariants (GEC-v7.0)
1. `#![forbid(unsafe_code)]` enforced across all Rust domain crates.
2. Zero panics, unwrap, or expect in non-test library code.
3. Clean-room ECMA-376 Part 1 (§17.16 Fields, §22 Math OMML), OASIS ODF 1.3 Text, and MS-RTF open specification.
4. Instance-scoped, zero global singletons.
5. All test suites (Rust workspace and Vitest frontend) pass 100% with zero regressions.

## Architecture & Implementation Requirements

### 1. Mail Merge Engine (`mail_merge.rs`)
- `MailMergeEngine`:
  - `find_merge_fields(doc: &Document) -> Vec<String>`: scans for `MERGEFIELD <name>` and chevrons `«<name>»`.
  - `merge_record(template: &Document, record: &HashMap<String, String>) -> Document`: clones template and replaces merge fields with record values.
  - `merge_batch(template: &Document, records: &[HashMap<String, String>]) -> Document`: generates single multi-section document with page/section breaks between records.
  - `parse_csv_datasource(csv_data: &str) -> (Vec<String>, Vec<HashMap<String, String>>)`: parses CSV into headers and records.

### 2. OMML & Bidirectional LaTeX Math Engine (`math.rs`)
- Models OMML DOM:
  - `MathElement`: `Run(String)`, `Fraction { num: Box<MathElement>, den: Box<MathElement> }`, `Radical { base: Box<MathElement>, degree: Option<Box<MathElement>> }`, `Superscript { base: Box<MathElement>, exp: Box<MathElement> }`, `Subscript { base: Box<MathElement>, sub: Box<MathElement> }`, `Delimiter { open: String, close: String, content: Vec<MathElement> }`, `Group(Vec<MathElement>)`.
- `MathConverter`:
  - `latex_to_omml(latex: &str) -> MathElement`: parses fractions `\frac{a}{b}`, radicals `\sqrt{x}`, `\sqrt[n]{x}`, superscripts `^`, subscripts `_`, delimiters `\left( ... \right)`, and Greek/math symbols.
  - `omml_to_latex(elem: &MathElement) -> String`: converts OMML DOM back into standard LaTeX representation.
  - `omml_to_xml(elem: &MathElement) -> String`: serializes into `<m:oMath>` OpenXML markup.

### 3. OpenDocument Text (.odt) & Rich Text Format (.rtf) Codecs (`odt_codec.rs`, `rtf_codec.rs`)
- `OdtCodec`:
  - `decode(odt_bytes: &[u8]) -> Result<Document, CodecError>`: extracts and parses `content.xml` for headings, paragraphs, styled runs, and tables.
  - `encode(doc: &Document) -> Result<Vec<u8>, CodecError>`: packages document into compliant ODF `.odt` archive with `mimetype`, `META-INF/manifest.xml`, and `content.xml`.
- `RtfCodec`:
  - `decode(rtf_bytes: &[u8]) -> Result<Document, CodecError>`: parses RTF control words (`\b`, `\i`, `\ul`, `\par`, `\tab`, `\fs`, `\qc`, `\qr`, font tables, color tables).
  - `encode(doc: &Document) -> Result<Vec<u8>, CodecError>`: generates clean 7-bit standard RTF.

### 4. Frontend Coordinator (`WordMailMergeCoordinator.ts`)
- Manages active data source (headers, records), active record index preview, available merge fields, and LaTeX equation insertion.

