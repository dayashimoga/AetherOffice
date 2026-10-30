---
description: Automated execution workflow for Stage 32: PDF Studio (.pdf) Complete Overhaul - Open/Save, Edit Text, Page Resize and Password Encrypt/Decrypt
---

# Workflow: Execute Stage 32 (/stage-32)

## Objective: Stage 32: PDF Studio (.pdf) Complete Overhaul - Open/Save, Edit Text, Page Resize and Password Encrypt/Decrypt
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

### Subphase 32.1.1: PDF Open/Save, Multi-Page Organizer, Page Resize & Password Encrypt/Decrypt
- **Spec File:** `.agents/stages/stage-32/subphase-32.1.1.md`
/execute-subphase 32.1.1
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Fix the completely non-responsive PDF workspace so every single button works:
   - **Open & Render Existing PDFs + Create Blank PDFs:** Clicking Open (in Home or File) opens any .pdf file from disk, parses its pages/text/dimensions via pdf-engine (and browser PDF canvas/object renderer), and renders every page interactively in both the main viewport and the left Pages thumbnail strip!
   - **Page Tools Tab (Full Page Manipulation):**
     - + Insert Blank Page (inserts a new page before/after current page and updates page count).
     - Delete Page (removes selected page, with confirmation if >1 page).
     - Duplicate Page (clones selected page and all its elements).
     - Rotate CW 90Â° & Rotate CCW 90Â° (rotates the page dimensions and content).
     - Move Page Up / Down & drag-and-drop thumbnail reordering.
     - **Resize PDF Page Size (Requested Feature):** Modal & toolbar selector to change page dimensions (A4 210Ã—297mm, Letter 8.5Ã—11in, Legal 8.5Ã—14in, A3, A5, or Custom Width Ã— Height mm/px) with option to Scale Content to Fit New Page Size and apply to Current Page or All Pages!
     - Split PDF (extract page range into a new PDF) & Merge PDF (append another .pdf file's pages into the current document).
   - **Sign & Protect Tab (Password Encryption & Decryption â€” Requested Feature):**
     - **Encrypt with Password:** Opens a security modal allowing the user to set an **Open Password (User Password)** and **Permissions Password (Owner Password â€” restrict printing/editing/copying)**. When saved or reopened, the PDF requires the password to unlock!
     - **Decrypt / Remove Password:** Allows unlocking a password-protected PDF by entering its password and removing encryption restrictions.
     - **Compress / Optimize PDF Size:** Reduces image resolution / strips unused objects and displays the before/after file size in KB.
2. Write automated tests verifying PDF Open, + Insert Page, Delete Page, Rotate, Resize PDF Page Size, and Encrypt / Decrypt Password. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 32.1.1`

### Subphase 32.1.2: PDF Direct Text/Image Editing, Annotations, Redaction, Forms & Signatures
- **Spec File:** `.agents/stages/stage-32/subphase-32.1.2.md`
/execute-subphase 32.1.2
Invoke @pdf-architect and @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** across PDF Studio's Home, Edit PDF, Forms, Comment & Annotate, Sign & Protect, and View tabs:
   - **Home Tab:** Make Open, Hand (drag-pan canvas), Select (select text/objects), Snapshot (captures selected region or current page as PNG to clipboard/download), 100%, Fit Page, Fit Width, Prev Page, Next Page, + In (Zoom In), - Out (Zoom Out), Search (opens live PDF text search bar highlighting matches across pages), Print, and Save (generates and downloads valid .pdf file) 100% functional!
   - **Edit PDF Tab:** Edit Text & Objects mode (click any text block on the PDF page to edit its text, font, size, or color in-place, drag to reposition, or press Delete), + Add Text (click anywhere on the page to create a new formatted text block), + Add Image (upload and resize image on the PDF page), Add Links, Header & Footer (stamps page numbers/dates across all pages), Watermark (diagonal or centered text/image watermark with opacity slider), and Background Color.
   - **Comment & Annotate Tab:** Highlight, Underline, Strikethrough, Sticky Note (places interactive comment marker + syncs with left Notes tab), Callout Text Box, Freehand Draw / Pencil (smooth ink drawing on the PDF canvas with color/thickness picker), Eraser, Shapes (Rect, Circle, Arrow, Line), Stamps (APPROVED, CONFIDENTIAL, DRAFT, REVIEWED, FINAL, VOID), and Redact Content (draws blackout redaction box that permanently strips underlying text/objects).
   - **Forms & Sign Tabs:** + Text Field, + Checkbox, + Radio Button, + Dropdown, + Date Field, + Clear Form, Fill Form Mode, and Place Digital Signature (draw signature on canvas pad or type cursive signature + date stamp, and place resizable signature onto the PDF).
2. Verify every single PDF tool and ribbon button with automated DOM tests. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 32.1.2`

