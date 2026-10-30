import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";

export class PdfRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return [
      "Home",
      "Edit PDF",
      "Forms",
      "Comment & Annotate",
      "Page Tools",
      "Sign & Protect",
      "View",
    ];
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Edit PDF":
        return "E";
      case "Forms":
        return "F";
      case "Comment & Annotate":
        return "C";
      case "Page Tools":
        return "P";
      case "Sign & Protect":
        return "S";
      case "View":
        return "W";
      default:
        return "H";
    }
  }

  public renderTab(tab: string, body: HTMLElement, emit: RibbonActionEmitter): void {
    body.innerHTML = "";
    switch (tab) {
      case "Home":
        this.renderHomeTab(body, emit);
        break;
      case "Edit PDF":
        this.renderEditPdfTab(body, emit);
        break;
      case "Forms":
        this.renderFormsTab(body, emit);
        break;
      case "Comment & Annotate":
        this.renderCommentTab(body, emit);
        break;
      case "Page Tools":
        this.renderPageToolsTab(body, emit);
        break;
      case "Sign & Protect":
        this.renderSignProtectTab(body, emit);
        break;
      case "View":
        this.renderViewTab(body, emit);
        break;
      default:
        this.renderHomeTab(body, emit);
    }
  }

  private renderHomeTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Navigation & Tools
    const toolGroup = createRibbonGroup("Tools");
    const toolRow = document.createElement("div");
    toolRow.className = "ribbon-group-row";
    toolRow.appendChild(createRibbonButton("Open PDF", "📂 Open", () => emit("OpenPdf"), false, "pdf-btn-open-pdf"));
    toolRow.appendChild(createRibbonButton("Hand Tool", "✋ Hand", () => emit("SetPdfTool", "hand"), true, "pdf-btn-hand"));
    toolRow.appendChild(createRibbonButton("Select Text", "↖ Select", () => emit("SetPdfTool", "select"), true, "pdf-btn-select"));
    toolRow.appendChild(createRibbonButton("Snapshot Tool", "📷 Snapshot", () => emit("SetPdfTool", "snapshot"), false, "pdf-btn-snapshot"));
    toolGroup.appendChild(toolRow);
    body.appendChild(toolGroup);

    // 2. Zoom & Navigation
    const navGroup = createRibbonGroup("Zoom & Navigation");
    const navRow1 = document.createElement("div");
    navRow1.className = "ribbon-group-row";
    navRow1.appendChild(createRibbonButton("100% Size", "🔍 100%", () => emit("Zoom100"), false, "pdf-btn-zoom-100"));
    navRow1.appendChild(createRibbonButton("Fit Page", "↕ Fit Page", () => emit("ZoomFitPage"), false, "pdf-btn-fit-page"));
    navRow1.appendChild(createRibbonButton("Fit Width", "↔ Fit Width", () => emit("ZoomFitWidth"), false, "pdf-btn-fit-width"));
    navGroup.appendChild(navRow1);

    const navRow2 = document.createElement("div");
    navRow2.className = "ribbon-group-row";
    navRow2.appendChild(createRibbonButton("Previous Page", "⏮ Prev", () => emit("PrevPage"), false, "pdf-btn-prev"));
    navRow2.appendChild(createRibbonButton("Next Page", "⏭ Next", () => emit("NextPage"), false, "pdf-btn-next"));
    navRow2.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn"), false, "pdf-btn-zoom-in"));
    navRow2.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut"), false, "pdf-btn-zoom-out"));
    navGroup.appendChild(navRow2);
    body.appendChild(navGroup);

    // 3. Find & Print
    const findGroup = createRibbonGroup("Search & Output");
    const findRow = document.createElement("div");
    findRow.className = "ribbon-group-row";
    findRow.appendChild(createRibbonButton("Search PDF", "🔍 Search", () => emit("Find"), false, "pdf-btn-search"));
    findRow.appendChild(createRibbonButton("Print Document", "🖨️ Print", () => emit("PrintDocument"), false, "pdf-btn-print"));
    findRow.appendChild(createRibbonButton("Quick Save", "💾 Save", () => emit("SaveDocument"), false, "pdf-btn-save"));
    findGroup.appendChild(findRow);
    body.appendChild(findGroup);
  }

  private renderEditPdfTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Text Editing
    const textGroup = createRibbonGroup("Text Editing");
    const textRow1 = document.createElement("div");
    textRow1.className = "ribbon-group-row";
    textRow1.appendChild(createRibbonButton("Add Text Box", "🔤 Add Text", () => emit("AddPdfText"), false, "pdf-btn-add-text"));
    textRow1.appendChild(createRibbonButton("Edit Text", "✏️ Edit Text", () => emit("EditPdfText"), true, "pdf-btn-edit-text"));
    textRow1.appendChild(createRibbonButton("Bold", "𝗕 Bold", () => emit("TogglePdfBold"), true, "pdf-btn-bold"));
    textRow1.appendChild(createRibbonButton("Italic", "𝘪 Italic", () => emit("TogglePdfItalic"), true, "pdf-btn-italic"));
    textGroup.appendChild(textRow1);

    const textRow2 = document.createElement("div");
    textRow2.className = "ribbon-group-row";
    textRow2.appendChild(
      createRibbonSelect(["Segoe UI", "Helvetica", "Times", "Courier", "Arial"], "Segoe UI", (val) =>
        emit("SetPdfFontFamily", val)
      , "pdf-select-font-family")
    );
    textRow2.appendChild(
      createRibbonSelect([10, 12, 14, 16, 18, 24, 32], 14, (val) =>
        emit("SetPdfFontSize", parseFloat(val))
      , "pdf-select-font-size")
    );
    textRow2.appendChild(createRibbonButton("Text Color", "🎨 Color", () => emit("SetPdfTextColor", "#000000"), false, "pdf-btn-color"));
    textGroup.appendChild(textRow2);
    body.appendChild(textGroup);

    // 2. Image Editing
    const imgGroup = createRibbonGroup("Image Editing");
    const imgRow = document.createElement("div");
    imgRow.className = "ribbon-group-row";
    imgRow.appendChild(createRibbonButton("Add Image", "🖼️ Add Image", () => emit("AddPdfImage"), false, "pdf-btn-add-image"));
    imgRow.appendChild(createRibbonButton("Replace Image", "🔄 Replace", () => emit("ReplacePdfImage"), false, "pdf-btn-replace-image"));
    imgRow.appendChild(createRibbonButton("Crop Image", "✂️ Crop", () => emit("CropPdfImage"), false, "pdf-btn-crop-image"));
    imgRow.appendChild(createRibbonButton("Rotate Image", "⟳ Rotate", () => emit("RotatePdfImage"), false, "pdf-btn-rotate-image"));
    imgGroup.appendChild(imgRow);
    body.appendChild(imgGroup);

    // 3. Watermark & Headers
    const wmkGroup = createRibbonGroup("Watermark & Headers");
    const wmkRow = document.createElement("div");
    wmkRow.className = "ribbon-group-row";
    wmkRow.appendChild(createRibbonButton("Add Watermark", "🌊 Watermark", () => emit("AddWatermark", "CONFIDENTIAL"), false, "pdf-btn-add-watermark"));
    wmkRow.appendChild(createRibbonButton("Header & Footer", "📑 Header/Footer", () => emit("AddHeaderFooter"), false, "pdf-btn-header-footer"));
    wmkRow.appendChild(createRibbonButton("Page Color", "🎨 Page Color", () => emit("SetPdfBackgroundColor", "#f8fafc"), false, "pdf-btn-bg-color"));
    wmkGroup.appendChild(wmkRow);
    body.appendChild(wmkGroup);

    // 4. Links
    const linkGroup = createRibbonGroup("Links");
    const linkRow = document.createElement("div");
    linkRow.className = "ribbon-group-row";
    linkRow.appendChild(createRibbonButton("Add Web Link", "🔗 Web Link", () => emit("AddWebLink"), false, "pdf-btn-web-link"));
    linkRow.appendChild(createRibbonButton("Page Jump Link", "📄 Page Link", () => emit("AddPageJumpLink"), false, "pdf-btn-page-link"));
    linkGroup.appendChild(linkRow);
    body.appendChild(linkGroup);
  }

  private renderFormsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Form Fields
    const fieldGroup = createRibbonGroup("Form Fields");
    const fldRow1 = document.createElement("div");
    fldRow1.className = "ribbon-group-row";
    fldRow1.appendChild(createRibbonButton("Text Field", "🔤 Text Field", () => emit("AddFormField", "text"), false, "pdf-btn-form-text"));
    fldRow1.appendChild(createRibbonButton("Checkbox", "☑ Checkbox", () => emit("AddFormField", "checkbox"), false, "pdf-btn-form-checkbox"));
    fldRow1.appendChild(createRibbonButton("Radio Button", "🔘 Radio", () => emit("AddFormField", "radio"), false, "pdf-btn-form-radio"));
    fldRow1.appendChild(createRibbonButton("Date Field", "📅 Date", () => emit("AddFormField", "date"), false, "pdf-btn-form-date"));
    fieldGroup.appendChild(fldRow1);

    const fldRow2 = document.createElement("div");
    fldRow2.className = "ribbon-group-row";
    fldRow2.appendChild(createRibbonButton("Dropdown", "▼ Dropdown", () => emit("AddFormField", "dropdown"), false, "pdf-btn-form-dropdown"));
    fldRow2.appendChild(createRibbonButton("List Box", "≡ List Box", () => emit("AddFormField", "listbox"), false, "pdf-btn-form-listbox"));
    fldRow2.appendChild(createRibbonButton("Signature Field", "✒️ Signature", () => emit("AddFormField", "signature"), false, "pdf-btn-form-sig"));
    fldRow2.appendChild(createRibbonButton("Button", "🔲 Button", () => emit("AddFormField", "button"), false, "pdf-btn-form-btn"));
    fieldGroup.appendChild(fldRow2);
    body.appendChild(fieldGroup);

    // 2. Field Tools
    const fldToolsGroup = createRibbonGroup("Field Tools");
    const fldToolRow = document.createElement("div");
    fldToolRow.className = "ribbon-group-row";
    fldToolRow.appendChild(createRibbonButton("Align Left", "Align Left", () => emit("AlignFormFields", "left"), false, "pdf-btn-form-align"));
    fldToolRow.appendChild(createRibbonButton("Distribute", "Distribute", () => emit("DistributeFormFields"), false, "pdf-btn-form-distribute"));
    fldToolRow.appendChild(createRibbonButton("Clear Form", "🧹 Clear Form", () => emit("ClearFormData"), false, "pdf-btn-form-clear"));
    fldToolRow.appendChild(createRibbonButton("Import Data", "📥 Import", () => emit("ImportFormData"), false, "pdf-btn-form-import"));
    fldToolRow.appendChild(createRibbonButton("Export Data", "📤 Export", () => emit("ExportFormData"), false, "pdf-btn-form-export"));
    fldToolsGroup.appendChild(fldToolRow);
    body.appendChild(fldToolsGroup);
  }

  private renderCommentTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Annotations
    const annotGroup = createRibbonGroup("Annotations");
    const annotRow1 = document.createElement("div");
    annotRow1.className = "ribbon-group-row";
    annotRow1.appendChild(createRibbonButton("Highlight", "🖍️ Highlight", () => emit("SetAnnotTool", "highlight"), true, "pdf-btn-highlight"));
    annotRow1.appendChild(createRibbonButton("Underline", "_ Underline", () => emit("SetAnnotTool", "underline"), true, "pdf-btn-underline"));
    annotRow1.appendChild(createRibbonButton("Strikethrough", "~ Strike", () => emit("SetAnnotTool", "strike"), true, "pdf-btn-strike"));
    annotRow1.appendChild(createRibbonButton("Sticky Note", "📝 Note", () => emit("SetAnnotTool", "note"), false, "pdf-btn-note"));
    annotGroup.appendChild(annotRow1);

    const annotRow2 = document.createElement("div");
    annotRow2.className = "ribbon-group-row";
    annotRow2.appendChild(createRibbonButton("Pencil Draw", "✏️ Pencil", () => emit("SetAnnotTool", "pencil"), true, "pdf-btn-pencil"));
    annotRow2.appendChild(createRibbonButton("Eraser", "🧹 Eraser", () => emit("SetAnnotTool", "eraser"), true, "pdf-btn-eraser"));
    annotRow2.appendChild(createRibbonButton("Callout Box", "💬 Callout", () => emit("SetAnnotTool", "callout"), false, "pdf-btn-callout"));
    annotRow2.appendChild(createRibbonButton("Approved Stamp", "✅ Approved", () => emit("AddStamp", "approved"), false, "pdf-btn-stamp-approved"));
    annotRow2.appendChild(createRibbonButton("Confidential", "🔒 Confidential", () => emit("AddStamp", "confidential"), false, "pdf-btn-stamp-confidential"));
    annotRow2.appendChild(createRibbonButton("Draft", "📝 Draft", () => emit("AddStamp", "draft"), false, "pdf-btn-stamp-draft"));
    annotRow2.appendChild(createRibbonButton("Reviewed", "👀 Reviewed", () => emit("AddStamp", "reviewed"), false, "pdf-btn-stamp-reviewed"));
    annotRow2.appendChild(createRibbonButton("Final", "🏁 Final", () => emit("AddStamp", "final"), false, "pdf-btn-stamp-final"));
    annotRow2.appendChild(createRibbonButton("Void", "🚫 Void", () => emit("AddStamp", "void"), false, "pdf-btn-stamp-void"));
    annotGroup.appendChild(annotRow2);
    body.appendChild(annotGroup);

    // 2. Drawing Shapes
    const drawGroup = createRibbonGroup("Drawing");
    const drawRow = document.createElement("div");
    drawRow.className = "ribbon-group-row";
    drawRow.appendChild(createRibbonButton("Line", "/ Line", () => emit("InsertShape", "line"), false, "pdf-btn-shape-line"));
    drawRow.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow"), false, "pdf-btn-shape-arrow"));
    drawRow.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect"), false, "pdf-btn-shape-rect"));
    drawRow.appendChild(createRibbonButton("Oval", "⚪ Oval", () => emit("InsertShape", "ellipse"), false, "pdf-btn-shape-oval"));
    drawGroup.appendChild(drawRow);
    body.appendChild(drawGroup);

    // 3. Redaction
    const redGroup = createRibbonGroup("Redaction");
    const redRow = document.createElement("div");
    redRow.className = "ribbon-group-row";
    redRow.appendChild(createRibbonButton("Mark Redaction", "⬛ Mark Redact", () => emit("MarkRedaction"), true, "pdf-btn-redact"));
    redRow.appendChild(createRibbonButton("Apply Redactions", "Apply Redactions", () => emit("ApplyRedactions"), false, "pdf-btn-apply-redact"));
    redGroup.appendChild(redRow);
    body.appendChild(redGroup);
  }

  private renderPageToolsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Management
    const pageGroup = createRibbonGroup("Page Management");
    const pageRow1 = document.createElement("div");
    pageRow1.className = "ribbon-group-row";
    pageRow1.appendChild(createRibbonButton("Rotate 90° CCW", "↶ Rotate Left", () => emit("RotatePageCCW"), false, "pdf-btn-rotate-ccw"));
    pageRow1.appendChild(createRibbonButton("Rotate 90° CW", "↷ Rotate Right", () => emit("RotatePageCW"), false, "pdf-btn-rotate-cw"));
    pageRow1.appendChild(createRibbonButton("Delete Page", "🗑️ Delete Page", () => emit("DeletePage"), false, "pdf-btn-delete-page"));
    pageRow1.appendChild(createRibbonButton("Insert Page", "➕ Insert Page", () => emit("InsertBlankPage"), false, "pdf-btn-add-page"));
    pageRow1.appendChild(createRibbonButton("Duplicate Page", "📋 Duplicate", () => emit("DuplicatePage"), false, "pdf-btn-duplicate-page"));
    pageGroup.appendChild(pageRow1);

    const pageRow2 = document.createElement("div");
    pageRow2.className = "ribbon-group-row";
    pageRow2.appendChild(createRibbonButton("Move Page Up", "▲ Move Up", () => emit("MovePageUp"), false, "pdf-btn-move-page-up"));
    pageRow2.appendChild(createRibbonButton("Move Page Down", "▼ Move Down", () => emit("MovePageDown"), false, "pdf-btn-move-page-down"));
    pageRow2.appendChild(createRibbonButton("Extract Pages", "📤 Extract", () => emit("ExtractPage"), false, "pdf-btn-extract-page"));
    pageRow2.appendChild(createRibbonButton("Split PDF", "✂️ Split PDF", () => emit("SplitPdf"), false, "pdf-btn-split-pdf"));
    pageRow2.appendChild(createRibbonButton("Merge PDF", "📑 Merge PDF", () => emit("MergePdf"), false, "pdf-btn-merge-pdf"));
    pageRow2.appendChild(createRibbonButton("Create Blank PDF", "📄 Blank PDF", () => emit("CreateBlankPdf"), false, "pdf-btn-create-blank"));
    pageGroup.appendChild(pageRow2);
    body.appendChild(pageGroup);

    // 2. Page Size & Dimensions
    const sizeGroup = createRibbonGroup("Page Size");
    const sizeRow1 = document.createElement("div");
    sizeRow1.className = "ribbon-group-row";
    sizeRow1.appendChild(createRibbonButton("Letter (8.5×11\")", "📐 Letter", () => emit("SetPageSize", "Letter"), false, "pdf-btn-page-size-letter"));
    sizeRow1.appendChild(createRibbonButton("A4 (210×297mm)", "📐 A4", () => emit("SetPageSize", "A4"), false, "pdf-btn-page-size-a4"));
    sizeRow1.appendChild(createRibbonButton("Legal (8.5×14\")", "📐 Legal", () => emit("SetPageSize", "Legal"), false, "pdf-btn-page-size-legal"));
    sizeGroup.appendChild(sizeRow1);

    const sizeRow2 = document.createElement("div");
    sizeRow2.className = "ribbon-group-row";
    sizeRow2.appendChild(createRibbonButton("A3", "📐 A3", () => emit("SetPageSize", "A3"), false, "pdf-btn-page-size-a3"));
    sizeRow2.appendChild(createRibbonButton("A5", "📐 A5", () => emit("SetPageSize", "A5"), false, "pdf-btn-page-size-a5"));
    sizeRow2.appendChild(createRibbonButton("Resize Page...", "⚙️ Custom Size", () => emit("ResizePageModal"), false, "pdf-btn-resize-page"));
    sizeGroup.appendChild(sizeRow2);
    body.appendChild(sizeGroup);
  }

  private renderSignProtectTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Fill & Sign
    const signGroup = createRibbonGroup("Fill & Sign");
    const signRow = document.createElement("div");
    signRow.className = "ribbon-group-row";
    signRow.appendChild(createRibbonButton("Add Signature", "✍️ Add Signature", () => emit("AddSignature"), false, "pdf-btn-add-signature"));
    signRow.appendChild(createRibbonButton("Add Initials", "Initials", () => emit("AddInitials"), false, "pdf-btn-add-initials"));
    signRow.appendChild(createRibbonButton("Sign Document", "✒️ Sign", () => emit("SignPdf"), false, "pdf-btn-sign"));
    signRow.appendChild(createRibbonButton("Export PDF", "💾 Export PDF", () => emit("ExportPdf"), false, "pdf-btn-export-pdf"));
    signGroup.appendChild(signRow);
    body.appendChild(signGroup);

    // 2. Security & Encryption
    const secGroup = createRibbonGroup("Security");
    const secRow = document.createElement("div");
    secRow.className = "ribbon-group-row";
    secRow.appendChild(createRibbonButton("Password Protect", "🔒 Encrypt Password", () => emit("EncryptPassword"), false, "pdf-btn-encrypt"));
    secRow.appendChild(createRibbonButton("Remove Security", "🔓 Decrypt Password", () => emit("DecryptPassword"), false, "pdf-btn-decrypt"));
    secRow.appendChild(createRibbonButton("Compress PDF", "🗜️ Compress", () => emit("CompressPdf"), false, "pdf-btn-compress"));
    secRow.appendChild(createRibbonButton("Restrict Editing", "Restrict Editing", () => emit("RestrictEditing"), false, "pdf-btn-restrict-editing"));
    secGroup.appendChild(secRow);
    body.appendChild(secGroup);

    // 3. Redaction
    const redGroup = createRibbonGroup("Redaction");
    const redRow = document.createElement("div");
    redRow.className = "ribbon-group-row";
    redRow.appendChild(createRibbonButton("Mark Redaction", "⬛ Mark Redact", () => emit("MarkRedaction"), true, "pdf-btn-redact"));
    redRow.appendChild(createRibbonButton("Apply Redactions", "Apply Redactions", () => emit("ApplyRedactions"), false, "pdf-btn-apply-redact"));
    redGroup.appendChild(redRow);
    body.appendChild(redGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Display
    const dispGroup = createRibbonGroup("Page Display");
    const dispRow = document.createElement("div");
    dispRow.className = "ribbon-group-row";
    dispRow.appendChild(createRibbonButton("Single Page", "📄 Single Page", () => emit("SetPageLayout", "single"), false, "pdf-btn-view-single"));
    dispRow.appendChild(createRibbonButton("Continuous", "📜 Continuous", () => emit("SetPageLayout", "continuous"), false, "pdf-btn-view-continuous"));
    dispRow.appendChild(createRibbonButton("Two Pages", "📄📄 Two Pages", () => emit("SetPageLayout", "two-page"), false, "pdf-btn-view-twopage"));
    dispGroup.appendChild(dispRow);
    body.appendChild(dispGroup);

    // 2. Rotation
    const rotGroup = createRibbonGroup("Rotate View");
    const rotRow = document.createElement("div");
    rotRow.className = "ribbon-group-row";
    rotRow.appendChild(createRibbonButton("Rotate View CW", "↷ View CW", () => emit("RotateView", 90), false, "pdf-btn-view-rot-cw"));
    rotRow.appendChild(createRibbonButton("Rotate View CCW", "↶ View CCW", () => emit("RotateView", -90), false, "pdf-btn-view-rot-ccw"));
    rotGroup.appendChild(rotRow);
    body.appendChild(rotGroup);

    // 3. Panes
    const paneGroup = createRibbonGroup("Navigation Panes");
    const paneRow = document.createElement("div");
    paneRow.className = "ribbon-group-row";
    paneRow.appendChild(createRibbonButton("Bookmarks", "📑 Bookmarks", () => emit("ToggleBookmarks"), true, "pdf-btn-view-bookmarks"));
    paneRow.appendChild(createRibbonButton("Thumbnails", "🖼️ Thumbnails", () => emit("ToggleThumbnails"), true, "pdf-btn-view-thumbnails"));
    paneRow.appendChild(createRibbonButton("Attachments", "📎 Attachments", () => emit("ToggleAttachments"), true, "pdf-btn-view-attachments"));
    paneRow.appendChild(createRibbonButton("Full Screen", "⛶ Full Screen", () => emit("ToggleFullScreen"), false, "pdf-btn-view-fullscreen"));
    paneGroup.appendChild(paneRow);
    body.appendChild(paneGroup);
  }
}

