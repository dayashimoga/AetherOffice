---
name: adaptive-ui-builder
description: Builds cross-platform responsive UI components in Tauri v2 that adapt across Desktop, Tablet, and Mobile.
---
1. Detect viewport width and pointer accuracy (`pointer: fine` vs `pointer: coarse`):
   - Desktop (`>= 1024px`): Full Ribbon toolbar, multi-window tabs, status bar, keyboard shortcuts.
   - Tablet (`768px - 1023px`): Collapsible touch ribbon, stylus ink canvas overlay.
   - Mobile (`< 768px`): Thumb-friendly bottom drawers, virtual-keyboard viewport avoidance, Word Reflow Reader, Excel Card Inspector.
2. Route all canvas rendering through `crates/render-core` at 120fps.
