# Sub-Phase 46.2.1: Dynamic Table of Contents, Footnotes/Endnotes & Bibliography Engine

## Objective
Implement industrial-grade Dynamic Table of Contents (TOC), bottom-of-page Footnotes/Endnotes separator layout and numbering, and BibTeX/CSL Citation & Bibliography Engine for WordprocessingML in `word-engine` and the unified presentation shell.

## Strict Engineering Invariants (GEC-v7.0)
1. `#![forbid(unsafe_code)]` enforced across all Rust crates.
2. Zero panics, unwrap, or expect in non-test library code.
3. Clean-room ECMA-376 Part 1 (§17.4 Fields and Hyperlinks, §17.11 Footnotes & Endnotes, §17.16 Bibliography) and ISO/IEC 29500-1 compliance.
4. Instance-scoped, zero global singletons.
5. All test suites (Rust workspace and Vitest frontend) pass 100% with zero regressions.

## Architecture & Implementation Requirements

### 1. Dynamic Table of Contents (`toc.rs`)
- Model `TocOptions`:
  - `start_level: u8`, `end_level: u8` (default 1..=3)
  - `use_hyperlinks: bool` (default true)
  - `show_page_numbers: bool` (default true)
  - `right_align_page_numbers: bool` (default true)
  - `tab_leader: TabLeader` (`Dot`, `Hyphen`, `Underscore`, `None`)
- Model `TocEntry`:
  - `level: u8`
  - `title: String`
  - `page_number: usize`
  - `bookmark_name: String`
  - `paragraph_index: usize`
- `TocEngine`:
  - `extract_headings(doc: &Document, cascade: &StyleCascadeResolver, pages: &[PageLayout]) -> Vec<TocEntry>`
  - `generate_toc_paragraphs(entries: &[TocEntry], options: &TocOptions) -> Vec<Paragraph>`
  - `update_toc_fields(doc: &mut Document, cascade: &StyleCascadeResolver, pages: &[PageLayout]) -> usize`

### 2. Footnotes & Endnotes Separator & Layout (`sub_documents.rs`, `layout.rs`)
- Numbering formats: Arabic (`1, 2, 3`), RomanLower (`i, ii, iii`), RomanUpper (`I, II, III`), Symbol (`*`, `†`, `‡`).
- Bottom-of-page footnote placement in `PaginationSolver`:
  - Calculates total footnote block height (separator line + spacing + footnote paragraphs).
  - Deducts footnote block from page body available height budget.
  - Generates `FootnoteArea` in `PageLayout`.

### 3. Bibliography & Citation Engine (`citations.rs`)
- Model `CitationSource`:
  - `id: String`, `source_type: SourceType` (Book, Article, Conference, Web, Report)
  - `authors: Vec<String>`, `title: String`, `year: Option<u32>`, `journal: Option<String>`, `doi: Option<String>`, `url: Option<String>`
- Citation Styles:
  - `CitationStyle::Apa` (`(Smith, 2024)`)
  - `CitationStyle::Ieee` (`[1]`)
  - `CitationStyle::Mla` (`(Smith 42)`)
  - `CitationStyle::Chicago` (`Smith (2024)`)
- `CitationEngine`:
  - Parses BibTeX format into `Vec<CitationSource>`
  - Formats inline in-text citations according to active `CitationStyle`
  - Generates sorted Bibliography / References section paragraphs according to style conventions

### 4. Frontend Coordinator (`WordReferenceCoordinator.ts`)
- Manages active TOC options, footnote numbering options, citations catalog, and bibliography style.
- Provides functions for inserting/refreshing TOC, inserting footnote/endnote references, and compiling bibliography.

