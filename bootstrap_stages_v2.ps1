# bootstrap_stages_v2.ps1 - Generates Complete Individual Format Stages & Workflows
$ErrorActionPreference = "Stop"
Write-Host "=== [3/3] Generating Detailed Individual Format Stages & Prompts ===" -ForegroundColor Cyan

$stageDirs = 1..7 | ForEach-Object { ".agents/stages/stage-$_" }
foreach ($d in $stageDirs) { New-Item -ItemType Directory -Force -Path $d | Out-Null }
New-Item -ItemType Directory -Force -Path ".agents/workflows", "specs" | Out-Null

$detailedSubphases = @(
    # STAGE 1: CORE OPC & WORKSPACE
    @{ id = "1.1.1"; stage = 1; phase = "1.1"; title = "Package Engine & Multi-App Cargo Architecture"; crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "1.1.2"; stage = 1; phase = "1.1"; title = "Differential COM Ground-Truth & Test Fixture Matrix"; crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "1.2.1"; stage = 1; phase = "1.2"; title = "Zero-Copy ZIP Streaming & OPC Content_Types Resolver"; crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "1.2.2"; stage = 1; phase = "1.2"; title = "Lossless Round-Trip UnparsedPartStore Engine"; crate = "ooxml-opc"; skill = "@ecma376-architect"; rea = $false },

    # STAGE 2: STANDALONE EXCEL (CALC) ENGINE & VIRTUAL GRID
    @{ id = "2.1.1"; stage = 2; phase = "2.1"; title = "SpreadsheetML Streaming Parser & Interned SharedStrings"; crate = "excel-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "2.1.2"; stage = 2; phase = "2.1"; title = "Compressed Sparse Page Grid (CSPG) Memory Model (1M Rows)"; crate = "excel-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "2.2.1"; stage = 2; phase = "2.2"; title = "Pratt Formula Lexer, AST Compiler & Spill Range Resolvers"; crate = "excel-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "2.2.2"; stage = 2; phase = "2.2"; title = "Excel Calc Chain DAG, Circular Ref Trapper & IEEE-754 Parity"; crate = "excel-engine"; skill = "@rea-binary-analyst"; rea = $true },
    @{ id = "2.2.3"; stage = 2; phase = "2.2"; title = "Parallel SIMD Evaluation of 450+ Built-in Excel Functions"; crate = "excel-engine"; skill = "@rea-binary-analyst"; rea = $true },
    @{ id = "2.3.1"; stage = 2; phase = "2.3"; title = "Virtualized 120fps Grid Viewport (Rows 1..1048576, Cols A..XFD)"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "2.3.2"; stage = 2; phase = "2.3"; title = "Interactive Formula Bar, Multi-Range Cell Selection & Auto-Fill"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },

    # STAGE 3: STANDALONE WORD (WRITER) ENGINE & FLOW LAYOUT
    @{ id = "3.1.1"; stage = 3; phase = "3.1"; title = "WordprocessingML B-Tree Piece-Tree DOM & Fast Keystroke Buffer"; crate = "word-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "3.1.2"; stage = 3; phase = "3.1"; title = "Style Inheritance Hierarchy, Numbering Engines & Section Layout"; crate = "word-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "3.2.1"; stage = 3; phase = "3.2"; title = "Knuth-Plass Line Breaking, Widow/Orphan & Deterministic Pagination"; crate = "word-engine"; skill = "@rea-binary-analyst"; rea = $true },
    @{ id = "3.2.2"; stage = 3; phase = "3.2"; title = "Table Auto-Fit Columns, Cross-Page Rows & Floating Image Wrap"; crate = "word-engine"; skill = "@rea-binary-analyst"; rea = $true },
    @{ id = "3.3.1"; stage = 3; phase = "3.3"; title = "Paginated Document Viewport, Subpixel Text Layout & Caret Blinking"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "3.3.2"; stage = 3; phase = "3.3"; title = "Full Home/Insert/Layout Ribbon Toolbar Binding & Live Formatting"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },

    # STAGE 4: STANDALONE POWERPOINT (SLIDES) ENGINE & SCENE GRAPH
    @{ id = "4.1.1"; stage = 4; phase = "4.1"; title = "SlideMaster -> SlideLayout -> Slide 3-Tier Inheritance Resolver"; crate = "ppt-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "4.1.2"; stage = 4; phase = "4.1"; title = "DrawingML Geometry Compiler: 187 Preset Shapes & Bezier Paths"; crate = "ppt-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "4.2.1"; stage = 4; phase = "4.2"; title = "SMIL Animation State Machine (p:timing) & Timeline Progressions"; crate = "ppt-engine"; skill = "@ecma376-architect"; rea = $false },
    @{ id = "4.2.2"; stage = 4; phase = "4.2"; title = "Morph Transition Vertex Matcher & SmartArt Constraint Solver"; crate = "ppt-engine"; skill = "@rea-binary-analyst"; rea = $true },
    @{ id = "4.3.1"; stage = 4; phase = "4.3"; title = "Interactive Vector Slide Stage, Transform Gizmos & Shape Editing"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "4.3.2"; stage = 4; phase = "4.3"; title = "Slide Sorter Filmstrip, Master Theme Switcher & Fullscreen F5 Show"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },

    # STAGE 5: STANDALONE PDF STUDIO (ISO 32000-1/2)
    @{ id = "5.1.1"; stage = 5; phase = "5.1"; title = "Streaming PDF Object Parser, Linearized XRef & Catalog Tree"; crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false },
    @{ id = "5.1.2"; stage = 5; phase = "5.1"; title = "Content Stream Filter Decompression & Font CMap Unicode Resolver"; crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false },
    @{ id = "5.2.1"; stage = 5; phase = "5.2"; title = "Incremental Update Writer & Digital Signature Preservation"; crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false },
    @{ id = "5.2.2"; stage = 5; phase = "5.2"; title = "Interactive AcroForms Engine, Annotations & Ink Path Layers"; crate = "pdf-engine"; skill = "@pdf-architect"; rea = $false },
    @{ id = "5.3.1"; stage = 5; phase = "5.3"; title = "Tile-Virtualized Multi-Page PDF Canvas with Zoom & Text Select"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "5.3.2"; stage = 5; phase = "5.3"; title = "In-Place PDF Text/Image Editor, Page Organiser & Form Filler UI"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },

    # STAGE 6: GLOBAL INTEGRATION & UNIFIED COMMAND BUS
    @{ id = "6.1.1"; stage = 6; phase = "6.1"; title = "Transactional Command Bus with Global Undo/Redo & Clipboard"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "6.1.2"; stage = 6; phase = "6.1"; title = "Multi-Window Independent Process Launcher & Tab Detachment"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },

    # STAGE 7: 5-OS ADAPTIVE PACKAGING & CI/CD
    @{ id = "7.1.1"; stage = 7; phase = "7.1"; title = "Desktop / Tablet / Mobile Responsive Breakpoint Adaptation"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false },
    @{ id = "7.1.2"; stage = 7; phase = "7.1"; title = "Windows, Linux, macOS, Android, and iOS Release Matrix CI/CD"; crate = "render-core"; skill = "@adaptive-ui-builder"; rea = $false }
)

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)

# Load existing tracker if available to preserve completed statuses
$existingTrackerMap = @{}
$trackerPath = "specs/stage-tracker.json"
if (Test-Path $trackerPath) {
    try {
        $rawText = [System.IO.File]::ReadAllText((Resolve-Path $trackerPath).Path, [System.Text.Encoding]::UTF8)
        $cleanText = $rawText.TrimStart([char]0xFEFF)
        $existing = $cleanText | ConvertFrom-Json
        foreach ($sub in $existing.subphases) {
            $existingTrackerMap[$sub.id] = $sub
        }
    } catch {
        Write-Host "Warning: Could not parse existing tracker; starting fresh." -ForegroundColor Yellow
    }
}

# 1. Write Detailed Subphase Prompts (.agents/stages/stage-X/subphase-X.Y.Z.md)
foreach ($sp in $detailedSubphases) {
    $stageDir = ".agents/stages/stage-$($sp.stage)"
    $filePath = "$stageDir/subphase-$($sp.id).md"
    $reaBadge = if ($sp.rea) { "YES (Run .\rea_bootstrap.ps1)" } else { "NO (ECMA-376 / ISO Standard)" }

    if (-not (Test-Path $filePath)) {
$promptContent = @"
# Subphase $($sp.id): $($sp.title)
- **Target Subsystem:** ``$($sp.crate)``
- **Skill:** ``$($sp.skill)``
- **REA Binary Analysis:** $reaBadge

## Instructions for Antigravity Agent
1. Implement the subsystem in ``crates/$($sp.crate)`` and ``apps/`` according to GEC-v1.0.
2. Ensure every UI button, input, or formula field dispatches through the Command Bus.
3. Validate with unit tests in ``crates/$($sp.crate)/tests/subphase_$($sp.id -replace '\.','_')_test.rs``.
4. Run ``powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`` and commit with ``[GEC-Verified]``.
"@
        [System.IO.File]::WriteAllText($filePath, $promptContent, $utf8NoBom)
    }
}

# 2. Generate Workflows for Dedicated Apps
$workflows = @{
    "stage-word"  = @{ title = "Writer (Word) Standalone Engine"; stages = @(3) }
    "stage-excel" = @{ title = "Calc (Excel) Standalone Engine"; stages = @(2) }
    "stage-ppt"   = @{ title = "Slides (PowerPoint) Standalone Engine"; stages = @(4) }
    "stage-pdf"   = @{ title = "PDF Studio Standalone Engine"; stages = @(5) }
}

foreach ($wf in $workflows.Keys) {
    $cfg = $workflows[$wf]
    $stagesIncluded = $cfg.stages
    $matching = $detailedSubphases | Where-Object { $stagesIncluded -contains $_.stage }
    $content = @"
---
description: Automated workflow for $($cfg.title)
---
# Dedicated Workflow: $($cfg.title)
Execute the following subphases sequentially under GEC-v1.0:

"@
    foreach ($m in $matching) {
        $content += "`n- **Subphase $($m.id)**: $($m.title) (`.agents/stages/stage-$($m.stage)/subphase-$($m.id).md`)"
    }
    [System.IO.File]::WriteAllText(".agents/workflows/$wf.md", $content, $utf8NoBom)
}

# 3. Write Updated specs/stage-tracker.json (preserving completed statuses without overwriting)
$trackerItems = @()
foreach ($sp in $detailedSubphases) {
    $existingItem = $existingTrackerMap[$sp.id]
    $status = "PENDING"
    $completedAt = $null

    # If already completed in the existing tracker and not in a replaced/new phase, preserve status
    if ($existingItem -and $existingItem.status -eq "COMPLETED") {
        if ($sp.stage -le 4 -and $sp.phase -ne "2.3" -and $sp.phase -ne "3.3" -and $sp.phase -ne "4.3") {
            $status = "COMPLETED"
            $completedAt = $existingItem.completed_at
        }
    }

    $trackerItems += [ordered]@{
        id           = $sp.id
        stage        = $sp.stage
        phase        = $sp.phase
        title        = $sp.title
        crate        = $sp.crate
        rea_required = $sp.rea
        status       = $status
        prompt_file  = ".agents/stages/stage-$($sp.stage)/subphase-$($sp.id).md"
        completed_at = $completedAt
    }
}

$trackerJson = [ordered]@{
    project         = "OpenOfficeRS"
    contract        = "GEC-v1.0"
    total_stages    = 7
    total_subphases = $detailedSubphases.Count
    subphases       = $trackerItems
}
$jsonString = $trackerJson | ConvertTo-Json -Depth 5
[System.IO.File]::WriteAllText("specs/stage-tracker.json", $jsonString, $utf8NoBom)

Write-Host "Updated $($detailedSubphases.Count) subphases across 7 stages. Created /stage-word, /stage-excel, /stage-ppt, and /stage-pdf workflows." -ForegroundColor Green
