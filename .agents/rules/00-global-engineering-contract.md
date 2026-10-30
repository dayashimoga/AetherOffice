---
trigger: always_on
description: Global Engineering Contract (GEC-v4.0) enforcing 100% Automated Feature Testing across Word, Excel, PPT, PDF, and TXT, Full-App Theme Cascading, Multi-Window Per Format Management, and Zero Visual Bleed.
---

# GLOBAL ENGINEERING CONTRACT (GEC-v4.0)

## ARTICLE I: SEPARATE WINDOWS PER DOCUMENT TYPE + UNIFIED SESSION HUB (LOW RAM)
1. **Dedicated App Windows Inside One Shared Process:**
   - Users must be able to open **Word (`.docx`)**, **Excel (`.xlsx`)**, **PowerPoint (`.pptx`)**, **PDF Studio (`.pdf`)**, and **Notepad (`.txt`/`.md`)** in their own dedicated, separate windows (via Tauri `WebviewWindowBuilder` sharing the single Rust process for `< 25 MB` overhead) OR dock/undock them at will from the **Window / App Hub Switcher**.
   - Every window is scoped cleanly to its document type (e.g., the Word window holds Word tabs and Word chrome; the Excel window holds Excel workbooks; the PDF window holds PDF documents) while a **Global Session & Auto-Save Registry** tracks all open documents across windows so nothing is ever lost and any open document is reachable in 1 click.

## ARTICLE II: ZERO LAYOUT BLEEDING & FULL-APP DARK/LIGHT THEME
1. **No Ghost Sidebars:** Word's `Navigation` sidebar MUST ONLY render when the active view is Word and `showNavPane == true`. It must NEVER appear on the left side of Excel, PowerPoint, PDF, or Notepad!
2. **No Permanent Alt-Keytip Badges:** The black `[F]`, `[H]`, `[N]`, `[P]` badges on Ribbon tabs (`File`, `Home`, `Insert`) MUST stay hidden by default and only appear when the user presses the `Alt` key.
3. **Full-App Theme Cascade:** Toggling `Dark / Light` mode MUST apply CSS theme variables (`--bg-app`, `--bg-toolbar`, `--bg-sidebar`, `--bg-workspace`, `--bg-surface`, `--text-primary`, `--text-muted`, `--border-subtle`) across 100% of the window (Titlebar, Tab bar, Ribbon, Left Sidebar, Workspace background, Dialogs, and Bottom Status Bar).
4. **Dynamic Per-App Status Bar:** The bottom status bar must show accurate metrics for the active app (Word: `Page X of Y | N words | N chars`; Excel: `Sheet X of Y | Selected: A1 | Sum / Avg / Count`; PPT: `Slide X of Y | Theme Name`; PDF: `Page X of Y | Dimensions | Security Status`; TXT: `Ln X, Col Y | N lines | N words | UTF-8`). Never hardcode `22 words`!

## ARTICLE III: 100% WORKING FILE I/O & RIBBON FEATURES (AUTOMATED TEST MANDATE)
1. **Working File Menu (`New`, `Open`, `Save`, `Save As`, `Export`, `Print`):**
   - Must work in ALL 5 apps (`docx`, `xlsx`, `pptx`, `pdf`, `txt`) using real file pickers (`<input type="file">` + Tauri FS API) and real file downloads/writes + clean print frames.
2. **Automated Feature Matrix Gate:**
   - Every single button in every ribbon tab across Word, Excel, PowerPoint, PDF, and Notepad must be tested by an automated Vitest DOM test in `apps/unified-shell/src/__tests__/exhaustive_feature_matrix.test.tsx`.
