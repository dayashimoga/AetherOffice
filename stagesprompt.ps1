# ==============================================================================
# stagesprompt.ps1 - Automated Multi-Stage, Multi-Phase, Multi-Subphase Generator
# Creates all 22 Subphase Prompt Artifacts, Antigravity Workflows (/stage-1..5),
# Progress State Tracker (specs/stage-tracker.json), and Interactive Dispatcher.
# ==============================================================================
$ErrorActionPreference = "Stop"

Write-Host "=== [OpenOfficeRS] Generating Automated Multi-Stage Prompt Pipeline ===" -ForegroundColor Cyan

# 1. Ensure Directories Exist
$dirs = @(
    ".agents/stages/stage-1",
    ".agents/stages/stage-2",
    ".agents/stages/stage-3",
    ".agents/stages/stage-4",
    ".agents/stages/stage-5",
    ".agents/workflows",
    "specs",
    "tools"
)
foreach ($d in $dirs) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}

# 2. Define Complete 5-Stage, 11-Phase, 22-Subphase Matrix
$subphases = @(
    # ---------------- STAGE 1: FOUNDATION & ECMA-376 OPC CORE ----------------
    @{
        id = "1.1.1"; stage = 1; phase = "1.1"; title = "Workspace, GitHub Repo & CI/CD Initialization"
        crate = "ooxml-opc"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Verified Cargo workspace, public/private GitHub repository, and 5-platform GitHub Actions workflow."
        validation = "cargo check --workspace && powershell -File tools/verify-contract.ps1"
        prompt = @'
/execute-subphase 1.1.1
Invoke @ecma376-architect under strict Global Engineering Contract (GEC-v1.0) compliance.
1. Verify the Cargo workspace structure across `crates/ooxml-opc`, `crates/word-engine`, `crates/excel-engine`, `crates/ppt-engine`, and `crates/render-core`.
2. Define the shared `EngineError` enum using `thiserror` in `crates/ooxml-opc/src/error.rs` with zero `.unwrap()` or `.expect()` calls.
3. Verify `.github/workflows/build-all-platforms.yml` syntax and initialize the GitHub remote repository using `gh repo create` if not yet linked.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and commit with `feat(subphase-1.1.1): initialize workspace error model and CI/CD [GEC-Verified]`.
'@
    },
    @{
        id = "1.1.2"; stage = 1; phase = "1.1"; title = "COM Ground-Truth Differential Harness"
        crate = "ooxml-opc"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Automated Python COM harness generating baseline JSON test vectors from local Office installation."
        validation = "cargo test --package ooxml-opc"
        prompt = @'
/execute-subphase 1.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. Enhance `tools/com-ground-truth/generate_truth.py` to support headless generation of baseline `.docx`, `.xlsx`, and `.pptx` fixtures alongside calculated ground-truth JSON files in `specs/evidence/{word,excel,ppt}/`.
2. Create a fallback deterministic synthetic fixture generator in Rust (`crates/ooxml-opc/src/fixtures.rs`) so CI/CD runners without Microsoft Office installed can still generate and validate all ECMA-376 test packages.
3. Write 25 baseline vector tests in `crates/ooxml-opc/tests/subphase_1_1_2_test.rs`.
4. Run `tools/verify-contract.ps1` and commit with `feat(subphase-1.1.2): complete COM ground-truth and synthetic fixture harness [GEC-Verified]`.
'@
    },
    @{
        id = "1.2.1"; stage = 1; phase = "1.2"; title = "Streaming ZIP & [Content_Types].xml / .rels Parser"
        crate = "ooxml-opc"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Zero-copy ECMA-376 Part 2 Open Packaging Conventions (OPC) reader/writer in crates/ooxml-opc."
        validation = "cargo test --package ooxml-opc --test subphase_1_2_1_test"
        prompt = @'
/execute-subphase 1.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ooxml-opc`, implement the ECMA-376 Part 2 (ISO/IEC 29500-2) Open Packaging Conventions engine:
   - Streaming parser and serializer for `[Content_Types].xml` (`Types`, `Default`, `Override`).
   - Hierarchical relationship graph resolver for `_rels/.rels` and part-level `_rels/*.rels` (`Id`, `Type`, `Target`, `TargetMode`).
   - Lazy ZIP entry stream reader so a 100MB `.xlsx` or `.pptx` package parses relationships in `< 15 ms` without decompressing all streams upfront.
2. Document the OPC state machine in `specs/evidence/word/1.2.1.md` and write >= 25 edge-case test vectors in `specs/evidence/word/1.2.1_vectors.json`.
3. Validate with `crates/ooxml-opc/tests/subphase_1_2_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "1.2.2"; stage = 1; phase = "1.2"; title = "UnparsedPartStore & Zero-Loss Round-Trip Engine"
        crate = "ooxml-opc"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "100% lossless round-trip preservation of unknown XML nodes, macros (vbaProject.bin), and custom parts."
        validation = "cargo test --package ooxml-opc --test subphase_1_2_2_test"
        prompt = @'
/execute-subphase 1.2.2
Invoke @ecma376-architect under GEC-v1.0 Article II (Zero-Loss Round-Trip Fidelity).
1. In `crates/ooxml-opc/src/unparsed.rs`, implement `UnparsedPartStore` and `RawXmlNodeBuffer` to capture:
   - Unmodeled ZIP parts (`vbaProject.bin`, digital signatures `_xmlsignatures/`, ActiveX controls, embedded OLE binaries, customXML).
   - Unknown XML elements and `mc:AlternateContent` / `mc:Ignorable` markup compatibility blocks inside modeled XML streams.
2. Prove deterministic round-trip preservation: opening any `.docx`, `.xlsx`, or `.pptx` and saving it back must retain 100% of relationships, binary parts, and unknown XML attributes.
3. Write >= 25 round-trip test vectors in `crates/ooxml-opc/tests/subphase_1_2_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },

    # ---------------- STAGE 2: EXCEL ENGINE (SPREADSHEETML + REA CALC) ----------------
    @{
        id = "2.1.1"; stage = 2; phase = "2.1"; title = "SharedStringTable, Styles & Parallel Workbook Parser"
        crate = "excel-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Multi-threaded parser for xl/workbook.xml, xl/sharedStrings.xml, and xl/styles.xml."
        validation = "cargo test --package excel-engine --test subphase_2_1_1_test"
        prompt = @'
/execute-subphase 2.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/excel-engine`, implement streaming `quick-xml` parsers and serializers for:
   - `xl/workbook.xml` (sheets, definedNames, calcPr, 1900 vs 1904 date system flag).
   - `xl/sharedStrings.xml` using deduplicated string interning (`Arc<str>` or arena table) capable of parsing 1,000,000 shared strings in `< 120 ms`.
   - `xl/styles.xml` (`numFmts`, `fonts`, `fills`, `borders`, `cellXfs`).
2. Preserve all unmodeled workbook extensions (`extLst`) via `ooxml-opc::RawXmlNodeBuffer`.
3. Create 25 test vectors in `specs/evidence/excel/2.1.1_vectors.json`, verify via `subphase_2_1_1_test.rs`, and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "2.1.2"; stage = 2; phase = "2.1"; title = "Compressed Sparse Page Grid (CSPG) Storage Engine"
        crate = "excel-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "SIMD-aligned 64x64 chunked sparse grid supporting 1,048,576 rows x 16,384 columns under 180MB RSS."
        validation = "cargo test --package excel-engine --test subphase_2_1_2_test"
        prompt = @'
/execute-subphase 2.1.2
Invoke @ecma376-architect under GEC-v1.0 Article III (Memory & Performance SLAs).
1. In `crates/excel-engine/src/grid.rs`, implement the Compressed Sparse Page Grid (CSPG) supporting full Excel sheet bounds (`1,048,576` rows by `16,384` columns, `A1:XFD1048576`):
   - Partition each worksheet into 64x64 cell pages indexed by a sparse `BTreeMap<(u16, u8), PageChunk>`.
   - Store cell payloads in Structure-of-Arrays (SoA) layout (`f64` values, string IDs, formula AST IDs, style indices) for cache-optimal SIMD scanning.
2. Implement parallel `rayon` worksheet XML loading (`xl/worksheets/sheet*.xml`) and serializer.
3. Verify memory footprint and O(1) lookup with >= 25 sparse/dense boundary vectors in `subphase_2_1_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "2.2.1"; stage = 2; phase = "2.2"; title = "Pratt Formula Parser, 3D Ranges & Dynamic Array Spill AST"
        crate = "excel-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Complete Excel formula lexer and AST compiler supporting R1C1, A1, structured tables, and spill (#) operators."
        validation = "cargo test --package excel-engine --test subphase_2_2_1_test"
        prompt = @'
/execute-subphase 2.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/excel-engine/src/formula/`, build a zero-panic Pratt parser and bytecode compiler for ECMA-376 formulas:
   - Cell references (`A1`, `$A$1`, `R1C1`), 3D sheet spans (`Sheet1:Sheet3!A1:B10`), union/intersection operators (`,`, ` `, `:`), structured table refs (`Table1[[#Headers],[Sales]]`), and dynamic array spill refs (`A1#`, `@`).
   - Exact operator precedence (`%`, `^`, `*`/`/`, `+`/`-`, `&`, comparison operators).
2. Store compiled bytecode alongside raw formula strings for lossless XML serialization.
3. Validate against >= 50 complex formula syntax trees in `subphase_2_2_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "2.2.2"; stage = 2; phase = "2.2"; title = "REA Calc Chain DAG & IEEE-754 Snap-to-Zero Parity"
        crate = "excel-engine"; rea_required = $true; skill = "@rea-binary-analyst"
        deliverable = "Clean-room topological dependency graph and exact Excel IEEE-754 floating-point subtraction rounding solver."
        validation = "cargo test --package excel-engine --test subphase_2_2_2_test"
        prompt = @'
/execute-subphase 2.2.2
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting local `EXCEL.EXE`.
1. Invoke @rea-binary-analyst (and `tools/com-ground-truth/generate_truth.py`) to document how `EXCEL.EXE` handles:
   - Dirty-flag propagation, circular reference detection, and iterative calculation convergence (`iterate`, `iterateCount`, `iterateDelta`).
   - Undocumented IEEE-754 subnormal / snap-to-zero rounding where `=1.333-1.225-0.108` vs `=(1.333-1.225-0.108)` diverges due to top-level parenthesis suppression of the 15-significant-digit snap heuristic.
2. Save the clean-room mathematical proof in `specs/evidence/excel/2.2.2.md` and >= 35 COM-verified JSON test vectors in `specs/evidence/excel/2.2.2_vectors.json`.
3. Implement the clean-room DAG evaluator and IEEE-754 snap arithmetic in `crates/excel-engine/src/calc_dag.rs`.
4. Pass 100% of `subphase_2_2_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "2.2.3"; stage = 2; phase = "2.2"; title = "Parallel Evaluation of Core & Financial/Lookup Functions"
        crate = "excel-engine"; rea_required = $true; skill = "@rea-binary-analyst"
        deliverable = "High-speed SIMD/Rayon implementation of Math, Statistical, Lookup (XLOOKUP/VLOOKUP), Logical, Text, and Financial functions."
        validation = "cargo test --package excel-engine --test subphase_2_2_3_test"
        prompt = @'
/execute-subphase 2.2.3
1. Invoke @rea-binary-analyst and COM ground-truth scripts to capture exact boundary/convergence rules for:
   - Lookup & Reference: `XLOOKUP`, `XMATCH`, `VLOOKUP`, `HLOOKUP`, `INDEX`, `MATCH`, `OFFSET`, `INDIRECT`, `FILTER`, `UNIQUE`, `SORT`.
   - Math & Statistical: `SUMIFS`, `COUNTIFS`, `AVERAGEIFS`, `SUBTOTAL`, `AGGREGATE`, `ROUND`, `ROUNDUP`, `ROUNDDOWN`, `MROUND`, `STDEV.S`, `PERCENTILE.INC`.
   - Financial & Date: `PMT`, `IPMT`, `PPMT`, `NPV`, `XNPV`, `IRR`, `XIRR` (Newton-Raphson secant fallback bounds), `WORKDAY.INTL`, `NETWORKDAYS.INTL`, `EDATE`, `EOMONTH` (including Excel's intentional 1900 leap-year bug `1900-02-29` = serial `60`).
   - Modern Lambda/Let: `LET`, `LAMBDA`, `MAP`, `REDUCE`, `SCAN`.
2. Write the mathematical specs and >= 50 test vectors to `specs/evidence/excel/2.2.3_vectors.json`.
3. Implement the function registry in `crates/excel-engine/src/functions/` with `rayon` parallel evaluation across independent DAG levels.
4. Pass `subphase_2_2_3_test.rs` and commit with `[GEC-Verified]`.
'@
    },

    # ---------------- STAGE 3: WORD ENGINE (WORDPROCESSINGML + REA LAYOUT) ----------------
    @{
        id = "3.1.1"; stage = 3; phase = "3.1"; title = "WordprocessingML Piece-Tree Rope DOM & Sub-Document Parser"
        crate = "word-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Immutable B-Tree Piece-Tree DOM for word/document.xml, headers/footers, footnotes, comments, and tracked changes."
        validation = "cargo test --package word-engine --test subphase_3_1_1_test"
        prompt = @'
/execute-subphase 3.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine`, implement parsers and lossless serializers for `word/document.xml`, `word/header*.xml`, `word/footer*.xml`, `word/footnotes.xml`, `word/endnotes.xml`, and `word/comments.xml`.
2. Model paragraphs (`w:p`), runs (`w:r`), bookmarks (`w:bookmarkStart`/`End`), hyperlinks, and revision tracking (`w:ins`, `w:del`) over an immutable B-Tree Piece-Tree / Rope so keystroke insertion/deletion in a 1,000-page document completes in `< 0.5 ms`.
3. Write >= 25 round-trip and edit-latency test vectors in `crates/word-engine/tests/subphase_3_1_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "3.1.2"; stage = 3; phase = "3.1"; title = "ECMA-376 Style Cascade, Numbering Lists & Section Geometry"
        crate = "word-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Complete style inheritance resolver (docDefaults -> table -> paragraph -> character -> direct) and multi-level list state machine."
        validation = "cargo test --package word-engine --test subphase_3_1_2_test"
        prompt = @'
/execute-subphase 3.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine/src/styles.rs` and `numbering.rs`, implement the ECMA-376 Part 1 style hierarchy resolver:
   - `w:docDefaults` -> `w:style` (`basedOn` chain, `next`, `link`) -> conditional `w:tblStylePr` (firstRow, lastRow, band1Horz, etc.) -> direct run/paragraph properties (`w:rPr`, `w:pPr`) including toggle property XOR rules (`w:b`, `w:i`, `w:strike`).
   - Multi-level list numbering engine (`word/numbering.xml`: `w:abstractNum`, `w:num`, `w:lvlOverride`, `w:lvlRestart`) computing exact prefix strings (`1.a.i`, Roman, bullet glyphs).
2. Validate against >= 30 complex nested style and list vectors in `subphase_3_1_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "3.2.1"; stage = 3; phase = "3.2"; title = "REA Line-Breaking Knuth-Plass/Greedy Hybrid & Pagination Solver"
        crate = "word-engine"; rea_required = $true; skill = "@rea-binary-analyst"
        deliverable = "Exact Word line-breaking, kerning, widow/orphan control, and multi-column page-break solver."
        validation = "cargo test --package word-engine --test subphase_3_2_1_test"
        prompt = @'
/execute-subphase 3.2.1
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting `wwlib.dll`.
1. Invoke @rea-binary-analyst on staged `wwlib.dll` (and COM ground-truth bounding boxes) to document:
   - Word's exact line-breaking whitespace compression, CJK/Latin kinsoku shori rules, and soft-hyphen penalty thresholds.
   - Pagination constraints: `w:widowControl` (minimum 2 lines at top/bottom of page), `w:keepNext`, `w:keepLines`, `w:pageBreakBefore`, and continuous vs next-page section breaks (`w:sectPr`).
2. Save the clean-room mathematical spec to `specs/evidence/word/3.2.1.md` and >= 25 pagination bounding-box vectors to `specs/evidence/word/3.2.1_vectors.json`.
3. Implement the deterministic pagination and line-layout engine in `crates/word-engine/src/layout.rs`.
4. Pass `subphase_3_2_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "3.2.2"; stage = 3; phase = "3.2"; title = "REA Table Auto-Fit Widths, Row Splitting & Floating Wrap Polygons"
        crate = "word-engine"; rea_required = $true; skill = "@rea-binary-analyst"
        deliverable = "Table column auto-fit solver, cross-page repeating table headers, and tight polygon text wrapping around floating DrawingML anchors."
        validation = "cargo test --package word-engine --test subphase_3_2_2_test"
        prompt = @'
/execute-subphase 3.2.2
1. Invoke @rea-binary-analyst on `wwlib.dll` to extract the mathematical constraints for:
   - Table Auto-Fit (`w:tblLayout w:type="autofit"`): minimum unbroken word width (`minContentWidth`) vs maximum single-line paragraph width (`maxContentWidth`) proportional distribution across `w:gridSpan` merged cells.
   - Cross-page table row breaking (`w:cantSplit`, `w:tblHeader` repeating header rows) and floating DrawingML anchor (`wp:anchor`) exclusion zones (`wrapSquare`, `wrapTight` polygon intersection with line segments).
2. Document formulas in `specs/evidence/word/3.2.2.md` and >= 25 test vectors in `specs/evidence/word/3.2.2_vectors.json`.
3. Implement the table and float layout solver in `crates/word-engine/src/table_layout.rs`.
4. Pass `subphase_3_2_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },

    # ---------------- STAGE 4: POWERPOINT ENGINE (PRESENTATIONML + REA MORPH) ----------------
    @{
        id = "4.1.1"; stage = 4; phase = "4.1"; title = "SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance Resolver"
        crate = "ppt-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "Complete PresentationML parser and 3-tier property/placeholder/theme cascade engine."
        validation = "cargo test --package ppt-engine --test subphase_4_1_1_test"
        prompt = @'
/execute-subphase 4.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine`, implement streaming parsers and serializers for `ppt/presentation.xml`, `ppt/slideMasters/*.xml`, `ppt/slideLayouts/*.xml`, `ppt/slides/*.xml`, `ppt/notesSlides/*.xml`, and `ppt/theme/*.xml`.
2. Build the 3-tier inheritance solver:
   - Resolve placeholder shapes (`p:ph` matched by `idx` or `type`) inheriting geometry (`a:xfrm`), fill/stroke styles, and `p:txStyles` (`titleStyle`, `bodyStyle`, `otherStyle`) from `SlideMaster` -> `SlideLayout` -> `Slide`.
   - Resolve Theme color schemes (`a:clrScheme` + tint/shade/lumMod/lumOff transforms) and font schemes (`+mj-lt`, `+mn-lt`).
3. Validate with >= 25 inheritance test vectors in `crates/ppt-engine/tests/subphase_4_1_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "4.1.2"; stage = 4; phase = "4.1"; title = "DrawingML 187 Preset Geometries & Guide Formula Compiler"
        crate = "ppt-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "DrawingML vector path evaluator supporting all 187 ECMA-376 preset geometries and interactive adjustment handles."
        validation = "cargo test --package ppt-engine --test subphase_4_1_2_test"
        prompt = @'
/execute-subphase 4.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/drawingml.rs`, implement the ECMA-376 Part 1 DrawingML geometry formula evaluator (`a:avLst`, `a:gdLst`, `a:pathLst`):
   - Support all DrawingML guide operations (`*/`, `+-`, `+/`, `?:`, `abs`, `at2`, `cat2`, `cos`, `sin`, `sqrt`, `tan`, `val`, `min`, `max`, `pin`, `mod`).
   - Compile all 187 standard `a:prstGeom` shapes (`rect`, `roundRect`, `ellipse`, `bentArrow`, `chevron`, `star5`, `flowChart*`, `math*`, etc.) and custom paths (`a:custGeom`) into normalized Bezier path commands (`MoveTo`, `LineTo`, `CubicTo`, `QuadTo`, `ArcTo`, `Close`).
2. Write >= 30 geometry evaluation tests in `subphase_4_1_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "4.2.1"; stage = 4; phase = "4.2"; title = "SMIL Animation Timing Tree (p:timing) State Machine"
        crate = "ppt-engine"; rea_required = $false; skill = "@ecma376-architect"
        deliverable = "120Hz frame-accurate slide animation timeline evaluator (entrance, emphasis, exit, motion paths)."
        validation = "cargo test --package ppt-engine --test subphase_4_2_1_test"
        prompt = @'
/execute-subphase 4.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ppt-engine/src/animation.rs`, implement the PresentationML SMIL animation timing engine (`p:timing` / `p:tnLst`):
   - Parallel (`p:par`) and sequential (`p:seq`) time containers, click triggers (`onClick`), `withPrev`, `afterPrev`, repeat counts, auto-reverse, and acceleration/deceleration bezier easing curves.
   - Property interpolators for `p:anim`, `p:animEffect`, `p:animMotion` (SVG/DrawingML motion paths), `p:animRot`, `p:animScale`, and `p:animClr`.
2. Verify deterministic 120Hz frame sampling (`sample_at(time_ms)`) across >= 25 timeline vectors in `subphase_4_2_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "4.2.2"; stage = 4; phase = "4.2"; title = "REA Morph Transition Vertex Solver & SmartArt Layout Engine"
        crate = "ppt-engine"; rea_required = $true; skill = "@rea-binary-analyst"
        deliverable = "Clean-room Morph transition shape/character interpolator and SmartArt (dgm:layoutDef) constraint solver."
        validation = "cargo test --package ppt-engine --test subphase_4_2_2_test"
        prompt = @'
/execute-subphase 4.2.2
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting `ppcore.dll`. After extracting specs, run `.\removeRea_bootstrap.ps1` to permanently lock the clean room!
1. Invoke @rea-binary-analyst on staged `ppcore.dll` to document:
   - Morph transition (`p159:morph`) correspondence matching rules: explicit `!!` shape naming, automatic shape/text/word/character correspondence scoring, and equal-subdivision cubic Bezier path correspondence when morphing between shapes with different vertex counts.
   - SmartArt (`ppt/diagrams/layout*.xml`) constraint solver (`dgm:alg` types: `lin`, `pyra`, `cycle`, `hierChild`, `snake`, `tx`, `sp`) and fallback synced `p:spTree` cache sync.
2. Save the clean-room mathematical spec to `specs/evidence/ppt/4.2.2.md` and >= 25 test vectors to `specs/evidence/ppt/4.2.2_vectors.json`.
3. Run `powershell -ExecutionPolicy Bypass -File .\removeRea_bootstrap.ps1` to purge REA and staged binaries from the workspace.
4. Implement the clean-room Morph and SmartArt solver in `crates/ppt-engine/src/morph.rs`, pass `subphase_4_2_2_test.rs`, and commit with `[GEC-Verified]`.
'@
    },

    # ---------------- STAGE 5: 120FPS WGPU RENDERER, ADAPTIVE SHELL & 5-OS RELEASE ----------------
    @{
        id = "5.1.1"; stage = 5; phase = "5.1"; title = "Unified 120fps WebGPU / Cosmic-Text Render Core"
        crate = "render-core"; rea_required = $false; skill = "@adaptive-ui-builder"
        deliverable = "Shared GPU display-list compositor and tile virtualizer for Word pages, Excel grids, and PowerPoint slides."
        validation = "cargo test --package render-core --test subphase_5_1_1_test"
        prompt = @'
/execute-subphase 5.1.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article III (120fps / < 2.5ms GPU raster budget).
1. In `crates/render-core`, implement a unified scene-graph and tile-virtualized display list compiler bridging `word-engine`, `excel-engine`, and `ppt-engine`:
   - Viewport culling & spatial R-Tree / grid indexing so only visible Excel cells, Word page slices, or PowerPoint slide layers generate GPU draw primitives (`QuadInstance`, `GlyphRun`, `BezierPathBatch`).
   - High-DPI subpixel glyph shaping and bi-directional (BiDi / RTL / CJK) text layout integration, plus deterministic SVG/PNG/PDF software rasterizer fallback for headless testing and print export.
2. Benchmark and verify `< 2.0 ms` display-list compilation for a 4K viewport in `crates/render-core/tests/subphase_5_1_1_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "5.1.2"; stage = 5; phase = "5.1"; title = "Interactive Hit-Testing, Selection Handles & Undo/Redo Command Bus"
        crate = "render-core"; rea_required = $false; skill = "@adaptive-ui-builder"
        deliverable = "Unified cursor/touch hit-testing, multi-range selection, clipboard (HTML/OOXML/Plain), and atomic Undo/Redo transaction stack."
        validation = "cargo test --package render-core --test subphase_5_1_2_test"
        prompt = @'
/execute-subphase 5.1.2
Invoke @adaptive-ui-builder under GEC-v1.0 compliance.
1. In `crates/render-core/src/interaction.rs`, build the cross-document interaction controller:
   - Sub-millisecond pointer/touch hit-testing mapping `(x, y, zoom, scroll)` coordinates to exact Word character offsets, Excel cell/fill-handle coordinates, or PowerPoint shape/rotation handles.
   - Unified transactional Command Bus with unlimited Undo/Redo history, pinch-to-zoom inertia physics, and multi-format clipboard serialization.
2. Validate with >= 25 interaction, selection, and undo/redo test vectors in `subphase_5_1_2_test.rs` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "5.2.1"; stage = 5; phase = "5.2"; title = "Tauri v2 Desktop & Laptop Shell (Ribbon, Multi-Tab, Formula Bar)"
        crate = "render-core"; rea_required = $false; skill = "@adaptive-ui-builder"
        deliverable = "Complete Tauri v2 Desktop UI (>= 1024px) in apps/unified-shell with multi-document tabs, contextual Ribbon, and keyboard shortcuts."
        validation = "cargo test --workspace --all-targets"
        prompt = @'
/execute-subphase 5.2.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Desktop/Laptop Form Factor).
1. Scaffold the Tauri v2 application inside `apps/unified-shell/` (`package.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) linking `ooxml-opc`, `word-engine`, `excel-engine`, `ppt-engine`, and `render-core`.
2. Implement the Desktop/Laptop UI (`>= 1024px`, fine pointer):
   - Multi-document workspace tab bar allowing `.docx`, `.xlsx`, and `.pptx` files to be edited side-by-side in a single lightweight window.
   - Contextual Ribbon toolbar (Home, Insert, Layout, Formulas, Slide Show, Review, View), Excel Formula Bar with live syntax highlighting, Word Ruler & Navigation Pane, and PowerPoint Slide Sorter filmstrip.
   - Full keyboard shortcut engine (`Ctrl/Cmd+C/V/Z/Y/S/F`, Excel `F2`/`F4`/Arrow navigation, Alt keytips).
3. Verify Tauri IPC commands and workspace tests via `tools/verify-contract.ps1` and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "5.2.2"; stage = 5; phase = "5.2"; title = "Adaptive Tablet & Mobile UI (Touch Drawers, Stylus Ink, Reflow Mode)"
        crate = "render-core"; rea_required = $false; skill = "@adaptive-ui-builder"
        deliverable = "Seamless Tablet (768px-1023px) and Mobile (< 768px) adaptation for Android and iOS."
        validation = "cargo test --workspace --all-targets"
        prompt = @'
/execute-subphase 5.2.2
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Tablet & Mobile Adaptation).
1. In `apps/unified-shell/src/`, implement automatic responsive layout switching based on viewport breakpoints, safe-area insets (`env(safe-area-inset-*)`), and input modality (`pointer: coarse`):
   - **Tablet Mode (`768px - 1023px`):** Collapsible touch-friendly Ribbon, direct stylus/Apple Pencil/S-Pen freehand ink annotation layer (`DrawingML ink`), and split-screen multitasking support.
   - **Mobile Mode (`< 768px`):** Replace top Ribbon with thumb-zone swipeable bottom command sheets, virtual-keyboard viewport height compensation, one-tap **Mobile Reflow Reader Mode** for Word documents, **Row Card Inspector** for rapid mobile Excel data entry, and touch laser-pointer mode for PowerPoint presentations.
2. Verify responsive state transitions and commit with `[GEC-Verified]`.
'@
    },
    @{
        id = "5.3.1"; stage = 5; phase = "5.3"; title = "End-to-End 5-Platform Release Audit & GitHub Artifact Publication"
        crate = "render-core"; rea_required = $false; skill = "@adaptive-ui-builder"
        deliverable = "Verified Windows (.msi/.exe), Linux (.AppImage/.deb), macOS (.dmg), Android (.apk), and iOS (.ipa) release artifacts."
        validation = "powershell -File tools/verify-contract.ps1"
        prompt = @'
/execute-subphase 5.3.1
Execute final Stage 5 Release Gatekeeping under GEC-v1.0:
1. Run `powershell -ExecutionPolicy Bypass -File .\removeRea_bootstrap.ps1 -KeepGlobalNpmPackage` to verify zero proprietary binaries or temp files exist anywhere in the repository.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1` and verify 100% pass rate across all 22 subphase test suites (`ooxml-opc`, `excel-engine`, `word-engine`, `ppt-engine`, `render-core`) and `apps/unified-shell` frontend build.
3. Commit final release metadata, tag `v1.0.0`, push to GitHub to trigger `.github/workflows/build-all-platforms.yml`, and verify that all 5 platform artifacts (`Windows-x64`, `Linux-x64`, `macOS-Universal`, `Android-Universal-APK`, `iOS-IPA`) build and upload cleanly.
'@
    }
)

# 3. Write Individual Subphase Prompt Markdown Files (.agents/stages/stage-X/subphase-X.Y.Z.md)
foreach ($sp in $subphases) {
    $stageFolder = ".agents/stages/stage-$($sp.stage)"
    $filePath = "$stageFolder/subphase-$($sp.id).md"
    $reaBadge = if ($sp.rea_required) { "YES (Run .\rea_bootstrap.ps1 before starting)" } else { "NO (Pure ECMA-376 / Clean-Room)" }

@"
# Subphase $($sp.id): $($sp.title)
- **Stage:** $($sp.stage) | **Phase:** $($sp.phase)
- **Target Crate:** ``$($sp.crate)``
- **Primary Agent Skill:** ``$($sp.skill)``
- **REA Binary Analysis Required:** $reaBadge
- **Deliverable:** $($sp.deliverable)
- **Validation Command:** ``$($sp.validation)``

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

``````text
$($sp.prompt)
``````
"@ | Set-Content -Path $filePath -Encoding UTF8
}

# 4. Generate Stage-Level Antigravity Slash Workflows (.agents/workflows/stage-1.md .. stage-5.md)
$stageDescriptions = @{
    1 = "Stage 1: Foundation, ECMA-376 Open Packaging Conventions (OPC) & Differential Test Harness"
    2 = "Stage 2: Excel Engine (SpreadsheetML Parser, CSPG Sparse Grid, DAG Calc & 450+ Functions)"
    3 = "Stage 3: Word Engine (WordprocessingML Piece-Tree DOM, Styles, Pagination & Tables)"
    4 = "Stage 4: PowerPoint Engine (PresentationML Inheritance, 187 Shapes, SMIL & Morph)"
    5 = "Stage 5: Unified 120fps WebGPU Render Core, Adaptive Desktop/Tablet/Mobile Shell & 5-OS CI/CD"
}

foreach ($s in 1..5) {
    $stageSubphases = $subphases | Where-Object { $_.stage -eq $s }
    $workflowContent = @"
---
description: Automated execution workflow for $($stageDescriptions[$s])
---
# Workflow: Execute Stage $s (/stage-$s)
## Objective: $($stageDescriptions[$s])

When invoked via ``/stage-$s``, execute the following subphases sequentially. Do NOT advance to the next subphase until ``tools/verify-contract.ps1`` exits with code 0 and ``specs/stage-tracker.json`` is updated:

"@
    foreach ($sp in $stageSubphases) {
        $workflowContent += @"

### Subphase $($sp.id): $($sp.title)
- **Prompt Spec File:** ``.agents/stages/stage-$s/subphase-$($sp.id).md``

**Instructions:**
``````text
$($sp.prompt)
``````

**Post-Subphase Gate:**
Run ``powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete $($sp.id)`` to verify contract compliance and advance the state tracker.

"@
    }
    $workflowContent | Set-Content -Path ".agents/workflows/stage-$s.md" -Encoding UTF8
}

# 5. Generate Master /run-next-subphase Workflow (.agents/workflows/run-next-subphase.md)
@'
---
description: Automatically reads specs/stage-tracker.json, loads the next PENDING subphase prompt, executes it under GEC-v1.0, and advances the tracker.
---
# Workflow: Run Next Subphase (/run-next-subphase)

When triggered by the user:

1. Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Status` in the terminal to identify the current active or next PENDING subphase ID.
2. Read the corresponding subphase specification file from `.agents/stages/stage-<N>/subphase-<ID>.md`.
3. If the subphase specifies `REA Binary Analysis Required: YES` and `.agents/skills/rea-binary-analyst.md` is not present, run `powershell -ExecutionPolicy Bypass -File .\rea_bootstrap.ps1`.
4. Execute all steps in `.agents/workflows/execute-subphase.md` for that subphase ID.
5. Once `tools/verify-contract.ps1` passes, mark the subphase complete by running:
   ```powershell
   powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete <ID>
   ```
6. Report the updated project completion percentage and ask if the user wants to trigger `/run-next-subphase` for the next item.
'@ | Set-Content -Path ".agents/workflows/run-next-subphase.md" -Encoding UTF8

# 6. Initialize Persistent Stage Tracker JSON (specs/stage-tracker.json)
$existingStatus = @{}
$existingCompletedAt = @{}
if (Test-Path "specs/stage-tracker.json") {
    try {
        $existing = Get-Content "specs/stage-tracker.json" -Raw | ConvertFrom-Json
        foreach ($sub in $existing.subphases) {
            $existingStatus[$sub.id] = $sub.status
            $existingCompletedAt[$sub.id] = $sub.completed_at
        }
    } catch {}
}

$trackerItems = @()
foreach ($sp in $subphases) {
    $status = if ($existingStatus.ContainsKey($sp.id)) { $existingStatus[$sp.id] } else { "PENDING" }
    $completedAt = if ($existingCompletedAt.ContainsKey($sp.id)) { $existingCompletedAt[$sp.id] } else { $null }

    $trackerItems += [ordered]@{
        id           = $sp.id
        stage        = $sp.stage
        phase        = $sp.phase
        title        = $sp.title
        crate        = $sp.crate
        rea_required = $sp.rea_required
        status       = $status
        prompt_file  = ".agents/stages/stage-$($sp.stage)/subphase-$($sp.id).md"
        completed_at = $completedAt
    }
}
$trackerJson = [ordered]@{
    project         = "OpenOfficeRS"
    contract        = "GEC-v1.0"
    total_stages    = 5
    total_subphases = $subphases.Count
    subphases       = $trackerItems
}
$trackerJson | ConvertTo-Json -Depth 5 | Set-Content -Path "specs/stage-tracker.json" -Encoding UTF8

# 7. Create Interactive Stage Runner & Progress Dispatcher (tools/stage-runner.ps1)
@'
# ==============================================================================
# tools/stage-runner.ps1 - Interactive Stage & Subphase Dispatcher
# Usage:
#   .\tools\stage-runner.ps1 -Status          # Show progress table across all 5 stages
#   .\tools\stage-runner.ps1 -Next            # Print & copy next PENDING subphase prompt to clipboard
#   .\tools\stage-runner.ps1 -Stage 2         # Print all prompts for Stage 2
#   .\tools\stage-runner.ps1 -Complete 1.1.1  # Run GEC-v1.0 audit & mark subphase 1.1.1 COMPLETED
# ==============================================================================
param(
    [switch]$Status,
    [switch]$Next,
    [int]$Stage = 0,
    [string]$Complete = ""
)
$ErrorActionPreference = "Stop"
$trackerPath = "specs/stage-tracker.json"

if (-not (Test-Path $trackerPath)) {
    throw "specs/stage-tracker.json not found. Run .\stagesprompt.ps1 first."
}

$tracker = Get-Content $trackerPath -Raw | ConvertFrom-Json

if ($Complete -ne "") {
    $item = $tracker.subphases | Where-Object { $_.id -eq $Complete }
    if (-not $item) { throw "Subphase ID '$Complete' not found in tracker." }

    Write-Host "=== Verifying Global Engineering Contract (GEC-v1.0) before completing Subphase $Complete ===" -ForegroundColor Cyan
    powershell -ExecutionPolicy Bypass -File "tools/verify-contract.ps1"
    if ($LASTEXITCODE -ne 0) {
        throw "Contract verification failed! Cannot mark Subphase $Complete as COMPLETED."
    }

    $item.status = "COMPLETED"
    $item.completed_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    $tracker | ConvertTo-Json -Depth 5 | Set-Content -Path $trackerPath -Encoding UTF8

    $doneCount = @($tracker.subphases | Where-Object { $_.status -eq "COMPLETED" }).Count
    $pct = [math]::Round(($doneCount / $tracker.total_subphases) * 100, 1)
    Write-Host "=== Subphase $Complete ($($item.title)) marked COMPLETED! Overall Progress: $doneCount/$($tracker.total_subphases) ($pct%) ===" -ForegroundColor Green
    exit 0
}

if ($Stage -gt 0) {
    $items = $tracker.subphases | Where-Object { $_.stage -eq $Stage }
    Write-Host "=== Stage $Stage Subphases ===" -ForegroundColor Cyan
    foreach ($i in $items) {
        Write-Host ("[{0}] {1} - {2} ({3})" -f $i.status, $i.id, $i.title, $i.prompt_file)
    }
    Write-Host "`nTip: In Antigravity IDE chat, type /stage-$Stage to execute this entire stage." -ForegroundColor Yellow
    exit 0
}

if ($Next) {
    $nextItem = $tracker.subphases | Where-Object { $_.status -ne "COMPLETED" } | Select-Object -First 1
    if (-not $nextItem) {
        Write-Host "ALL 22 SUBPHASES ACROSS ALL 5 STAGES ARE COMPLETED!" -ForegroundColor Green
        exit 0
    }

    Write-Host "=== NEXT SUBPHASE TO EXECUTE: $($nextItem.id) - $($nextItem.title) ===" -ForegroundColor Cyan
    if ($nextItem.rea_required) {
        Write-Host "[!] Notice: This subphase uses REA binary analysis. Run .\rea_bootstrap.ps1 if not active." -ForegroundColor Yellow
    }

    $promptContent = Get-Content $nextItem.prompt_file -Raw
    Write-Host "`n$promptContent" -ForegroundColor Gray

    if (Get-Command "Set-Clipboard" -ErrorAction SilentlyContinue) {
        try {
            Set-Clipboard -Value $promptContent
            Write-Host ">>> Prompt automatically copied to your clipboard! Paste it into Antigravity Agent Manager or type `/run-next-subphase`." -ForegroundColor Green
        } catch {
            Write-Host ">>> (Note: Clipboard access unavailable in this session. Copy the prompt above manually.)" -ForegroundColor Yellow
        }
    }
    exit 0
}

# Default or -Status: Display Full Progress Dashboard
$doneCount = @($tracker.subphases | Where-Object { $_.status -eq "COMPLETED" }).Count
$pct = [math]::Round(($doneCount / $tracker.total_subphases) * 100, 1)
Write-Host "=====================================================================================" -ForegroundColor Cyan
Write-Host " OpenOfficeRS Multi-Stage Execution Tracker | Progress: $doneCount / $($tracker.total_subphases) ($pct%)" -ForegroundColor Cyan
Write-Host "=====================================================================================" -ForegroundColor Cyan

foreach ($sp in $tracker.subphases) {
    $color = switch ($sp.status) {
        "COMPLETED" { "Green" }
        "IN_PROGRESS" { "Yellow" }
        default { "Gray" }
    }
    $reaTag = if ($sp.rea_required) { "[REA]" } else { "[OOXML]" }
    Write-Host (" Stage {0} | {1,-6} | {2,-11} | {3,-7} | {4}" -f $sp.stage, $sp.id, $sp.status, $reaTag, $sp.title) -ForegroundColor $color
}
Write-Host "-------------------------------------------------------------------------------------"
Write-Host "Commands:" -ForegroundColor Yellow
Write-Host "  .\tools\stage-runner.ps1 -Next             # Copy next subphase prompt to clipboard"
Write-Host "  .\tools\stage-runner.ps1 -Complete <ID>    # Verify GEC-v1.0 & mark subphase done"
Write-Host "  Antigravity Slash Commands: /run-next-subphase or /stage-1 .. /stage-5"
'@ | Set-Content -Path "tools/stage-runner.ps1" -Encoding UTF8

Write-Host "=== [OpenOfficeRS] Generated 22 Subphase Specs, 6 Slash Workflows & Stage Runner! ===" -ForegroundColor Green
Write-Host "Run .\tools\stage-runner.ps1 -Status to view your full 5-Stage dashboard, or .\tools\stage-runner.ps1 -Next to copy the first prompt!" -ForegroundColor Yellow
