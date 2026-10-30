---
trigger: always_on
description: Global Engineering Contract (GEC-v3.0) enforcing Single-Process <45MB Memory Architecture, Zero Prepopulated Fake Text, Native Window Controls, Dark/Light Theme, and 5 Working Modules (Word, Excel, PPT, PDF, Notepad).
---

# GLOBAL ENGINEERING CONTRACT (GEC-v3.0)

## ARTICLE I: SINGLE-EXECUTABLE, LOW-MEMORY ARCHITECTURE (< 45 MB RAM)
1. **Single Unified Binary (`AetherOffice`):** The suite MUST compile as a SINGLE desktop/mobile application containing an internal App Switcher Rail and multi-tab workspace supporting 5 modules: **Home Hub**, **Word (`.docx`)**, **Excel (`.xlsx`)**, **PowerPoint (`.pptx`)**, **PDF Studio (`.pdf`)**, and **Notepad/Notes (`.txt`/`.md`)**.
2. **Strict Memory Budget (Zero System Hang):**
   - Never pre-allocate 1,000,000 DOM nodes or massive Rust vectors on startup.
   - Excel grids MUST use lazy `HashMap<(u32, u16), Cell>` storage and render ONLY the visible viewport cells (`~40 rows x 15 cols`) in the DOM.
   - Word and PDF documents MUST lazily render pages and use clean native/virtualized scrolling.

## ARTICLE II: ABSOLUTE BAN ON PREPOPULATED FAKE / DEMO TEXT
1. **100% Blank New Documents:**
   - Creating or opening a new Word document MUST open a completely blank editable page (`""` empty string).
   - Creating a new Excel workbook MUST open a completely empty grid (`Sheet1` with zero prepopulated numbers).
   - Creating a new PowerPoint MUST open a clean blank slide with empty placeholder prompts (`Click to add title`).
   - Creating a new Note MUST open an empty text buffer.
2. **Banned Strings:** The strings `"Executive Technical Summary"`, `"Performance SLAs"`, `"Market Opportunity"`, `"Revenue"`, and `"Live Demo & Architecture"` are STRICTLY BANNED from `apps/unified-shell/src/` outside of tests. `tools/verify-contract.ps1` will fail if any fake demo text is present.

## ARTICLE III: UI/UX POLISH, NATIVE WINDOW CONTROLS & DARK/LIGHT THEME
1. **Window Controls:** Minimize, Maximize/Restore, and Close buttons MUST call Tauri's native window API (`appWindow.minimize()`, `appWindow.toggleMaximize()`, `appWindow.close()`) AND native OS window decorations must remain responsive.
2. **Zero Ribbon Scrollbars:** The Ribbon toolbar MUST use clean CSS Flexbox (`flex-wrap: nowrap`, `overflow: visible` for dropdowns, compact icon+label buttons) with ZERO horizontal or vertical scrollbars clipping the controls.
3. **Dark & Light Theme Toggle:** Every screen and document workspace must support a 1-click **Dark / Light Theme Toggle** persisted in `localStorage` using CSS custom properties (`--bg-primary`, `--bg-surface`, `--text-primary`, `--border-color`, `--accent-color`).
