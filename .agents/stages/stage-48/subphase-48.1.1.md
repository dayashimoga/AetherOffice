# Sub-Phase 48.1.1: Interactive Slide Master & Layout Designer View

## Objective
Implement industrial-grade, interactive Slide Master and Layout Designer capabilities in `ppt-engine` and the unified presentation shell (`apps/unified-shell`). Enable authoritative template hierarchy management (`SlideMaster -> SlideLayout -> Slide`), customizable placeholder layouts, dynamic template switching on existing slides, and a dedicated Master View editing surface with zero data loss and clean-room provenance.

## Scope of Features

### 1. Slide Master & Layout Hierarchy Model (`crates/ppt-engine/src/master.rs`)
- **Serializable Master & Layout DTOs (`MasterLayoutTreeDto`, `SlideMasterDto`, `SlideLayoutDto`, `PlaceholderDto`):**
  - Represents the complete hierarchical tree of masters, layouts, placeholders, and associated themes.
  - Captures placeholder type (`title`, `body`, `ctrTitle`, `subTitle`, `dt`, `ftr`, `sldNum`, `pic`, `chart`, `tbl`), shape ID, coordinates in EMUs (`x, y, cx, cy`), and default prompt text.
- **Hierarchical Tree Extraction (`SlideMasterDesigner::extract_master_tree`):**
  - Walks `presentation.slide_masters`, discovers all child `slide_layouts` via OPC relationship traversal, resolves theme color schemes and font schemes, and exports complete structured tree metadata.

### 2. Placeholder Inheritance & Layout Templating Engine
- **Standard Layout Templates:**
  - `title`: Title Slide (Title + Subtitle)
  - `titleAndContent`: Title and Content (Title + Body)
  - `sectionHeader`: Section Header (Title + Text)
  - `twoContent`: Two Content (Title + Two Side-by-Side Bodies)
  - `comparison`: Comparison (Title + Two Subheaders + Two Bodies)
  - `titleOnly`: Title Only
  - `blank`: Blank Slide (Header/Footer placeholders only)
  - `contentWithCaption`: Content with Caption
  - `pictureWithCaption`: Picture with Caption
- **Placeholder Inheritance Harmonization:**
  - Shapes on slides seamlessly inherit positions, sizes, default geometry, and text styles from layout placeholders, which in turn inherit from master placeholders.

### 3. Layout Designer Operations (`SlideMasterDesigner`)
- **Layout Creation (`create_layout`):**
  - Generates new `SlideLayout` parts, registers OPC relationships to the parent master and content types, and initializes canonical placeholders according to the chosen template preset.
- **Layout Duplication (`duplicate_layout`):**
  - Clones an existing layout with its shapes, placeholders, and formatting under a new name.
- **Layout Deletion (`delete_layout`):**
  - Safely deletes unused layouts and cleans up relationship entries.
- **Placeholder Management (`add_placeholder`, `remove_placeholder`, `update_placeholder_transform`):**
  - Adds, updates, or removes placeholders on masters or layouts.
- **Slide Layout Reassignment (`apply_layout_to_slide`):**
  - Updates a slide's relationship to reference a new layout part, realigns existing slide placeholders with new layout targets, and recomputes the cascade.

### 4. Desktop Tauri IPC Commands (`apps/unified-shell/src-tauri/src/binary_commands.rs`)
- `ppt_get_slide_master_tree(package_bytes)`: returns `MasterLayoutTreeDto`.
- `ppt_apply_slide_master_layout(package_bytes, slide_idx, layout_part_name)`: updates slide layout and returns updated `.pptx` binary bytes.
- `ppt_create_slide_layout(package_bytes, master_part_name, layout_type, name)`: creates layout and returns updated `.pptx` binary bytes.
- `ppt_update_placeholder_transform(package_bytes, target_part, shape_id, x, y, cx, cy)`: updates layout/master placeholder geometry.

### 5. Presentation Shell Slide Master Coordinator (`apps/unified-shell/src/state/SlideMasterCoordinator.ts`)
- Manages Slide Master view state:
  - `isMasterView: boolean`
  - `activeMasterPartName: string | null`
  - `activeLayoutPartName: string | null`
  - `masterTree: MasterLayoutTreeDto | null`
- Live layout switching for active slides.
- Integration with `slidesState`:
  - `[✓ Close Master View]` floating exit button returning to normal slide editing.
  - Contextual layout picker displaying available templates from the master.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all Rust crates.
- Zero panics, unwrap, or expect in non-test library code.
- Clean-room implementation of ECMA-376 Part 1 §19 & §20.
- 100% test pass rate across Rust workspace and frontend Vitest suites.

