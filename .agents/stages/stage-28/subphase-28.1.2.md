# Subphase 28.1.2: Multi-Document Per Format Manager, Side-by-Side Split View & Real Binary File Open/Save
- **Stage:** 28 | **Phase:** 28.1
- **Target Subsystem:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **Deliverable:** Open unlimited simultaneous .docx, .xlsx, .pptx, .pdf, and .txt files, filter by app or view all, edit side-by-side in Split View, and open/save real binary files.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 28.1.2
Invoke @ecma376-architect and @adaptive-ui-builder under GEC-v5.0 Article I.
1. **Seamless Multi-Document Management (Multiple .docx, .xlsx, .pptx, .pdf, .txt at the same time):**
   - Upgrade the Left App Rail (Home, W, X, P, PDF, N):
     - Clicking an app icon (W, X, P, PDF, N) shows a badge with the count of open files of that type (e.g., W (3), X (2)). If none are open, it creates one; if already viewing that app, clicking + on the tab bar or clicking the app icon's + New / ðŸ“‚ Open quick menu opens **additional simultaneous documents** of that format (Untitled 1.docx, Untitled 2.docx, Project.docx, Budget.xlsx, Forecast.xlsx, etc.)!
     - Add a tab bar filter toggle: **Current App Tabs vs All Open Tabs (N)** so users can either focus strictly on their open Excel files OR see all open documents across all formats in one unified tab strip.
     - Add a **â—« Split View (Side-by-Side Workspace)** button in the top bar: when enabled, splits the main workspace into **Left Pane** and **Right Pane** with independent tab selectors so a user can edit two Word docs, two Excel sheets, or a Word doc + PDF side-by-side simultaneously!
2. **Fix Real Local File Open & Save for .docx, .xlsx, .pptx, .pdf, and .txt:**
   - Use JSZip / ooxml-opc + FileReader.readAsArrayBuffer() so when a user clicks Open (Ctrl+O) and selects a real .docx, .xlsx, .pptx, .pdf, or .txt file from their computer:
     - **.docx:** Unzips word/document.xml, parses paragraphs (<w:p>), runs (<w:r>, <w:t>, bold/italic/color/size), and tables (<w:tbl>) into editable HTML pages!
     - **.xlsx / .csv:** Unzips xl/sharedStrings.xml and xl/worksheets/sheet*.xml (or parses CSV) and populates all sheets, cell values, and formulas into the grid!
     - **.pptx:** Unzips ppt/slides/slide*.xml, extracts text shapes (<p:sp>, <a:t>) and geometry into editable slides!
     - **.pdf:** Reads the binary PDF buffer, parses page count, dimensions, and text streams (via pdf-engine / PDF parser), and renders every page onto the PDF canvas!
   - When clicking Save (Ctrl+S), package and download/write a valid binary .docx, .xlsx, .pptx, .pdf, or .txt file!
3. Write automated tests verifying opening 3 simultaneous .docx and 2 .xlsx files, enabling Split View, and round-trip binary parsing/saving for all 5 formats. Commit with [GEC-Verified].
```
