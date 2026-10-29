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
    toolRow.appendChild(createRibbonButton("Hand Tool", "✋ Hand", () => emit("SetPdfTool", "hand"), true, "pdf-btn-hand"));
    toolRow.appendChild(createRibbonButton("Select Text", "↖ Select", () => emit("SetPdfTool", "select"), true));
    toolRow.appendChild(createRibbonButton("Snapshot Tool", "📷 Snapshot", () => emit("SetPdfTool", "snapshot")));
    toolGroup.appendChild(toolRow);
    body.appendChild(toolGroup);

    // 2. Zoom & Navigation
    const navGroup = createRibbonGroup("Zoom & Navigation");
    const navRow1 = document.createElement("div");
    navRow1.className = "ribbon-group-row";
    navRow1.appendChild(createRibbonButton("100% Size", "🔍 100%", () => emit("Zoom100")));
    navRow1.appendChild(createRibbonButton("Fit Page", "↕ Fit Page", () => emit("ZoomFitPage")));
    navRow1.appendChild(createRibbonButton("Fit Width", "↔ Fit Width", () => emit("ZoomFitWidth")));
    navGroup.appendChild(navRow1);

    const navRow2 = document.createElement("div");
    navRow2.className = "ribbon-group-row";
    navRow2.appendChild(createRibbonButton("Previous Page", "⏮ Prev", () => emit("PrevPage")));
    navRow2.appendChild(createRibbonButton("Next Page", "⏭ Next", () => emit("NextPage")));
    navRow2.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn")));
    navRow2.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut")));
    navGroup.appendChild(navRow2);
    body.appendChild(navGroup);

    // 3. Find & Print
    const findGroup = createRibbonGroup("Search & Output");
    const findRow = document.createElement("div");
    findRow.className = "ribbon-group-row";
    findRow.appendChild(createRibbonButton("Search PDF", "🔍 Search", () => emit("Find")));
    findRow.appendChild(createRibbonButton("Print Document", "🖨️ Print", () => emit("PrintDocument")));
    findRow.appendChild(createRibbonButton("Quick Save", "💾 Save", () => emit("SaveDocument")));
    findGroup.appendChild(findRow);
    body.appendChild(findGroup);
  }

  private renderEditPdfTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Text Editing
    const textGroup = createRibbonGroup("Text Editing");
    const textRow1 = document.createElement("div");
    textRow1.className = "ribbon-group-row";
    textRow1.appendChild(createRibbonButton("Add Text Box", "🔤 Add Text", () => emit("AddPdfText")));
    textRow1.appendChild(createRibbonButton("Edit Text", "✏️ Edit Text", () => emit("EditPdfText"), true));
    textGroup.appendChild(textRow1);

    const textRow2 = document.createElement("div");
    textRow2.className = "ribbon-group-row";
    textRow2.appendChild(
      createRibbonSelect(["Helvetica", "Times", "Courier", "Arial"], "Helvetica", (val) =>
        emit("SetPdfFontFamily", val)
      )
    );
    textRow2.appendChild(
      createRibbonSelect([10, 12, 14, 16, 18, 24], 12, (val) =>
        emit("SetPdfFontSize", parseFloat(val))
      )
    );
    textRow2.appendChild(createRibbonButton("Text Color", "🎨 Color", () => emit("SetPdfTextColor", "#000000")));
    textGroup.appendChild(textRow2);
    body.appendChild(textGroup);

    // 2. Image Editing
    const imgGroup = createRibbonGroup("Image Editing");
    const imgRow = document.createElement("div");
    imgRow.className = "ribbon-group-row";
    imgRow.appendChild(createRibbonButton("Add Image", "🖼️ Add Image", () => emit("AddPdfImage")));
    imgRow.appendChild(createRibbonButton("Replace Image", "🔄 Replace", () => emit("ReplacePdfImage")));
    imgRow.appendChild(createRibbonButton("Crop Image", "✂️ Crop", () => emit("CropPdfImage")));
    imgRow.appendChild(createRibbonButton("Rotate Image", "⟳ Rotate", () => emit("RotatePdfImage")));
    imgGroup.appendChild(imgRow);
    body.appendChild(imgGroup);

    // 3. Links
    const linkGroup = createRibbonGroup("Links");
    linkGroup.appendChild(createRibbonButton("Add Web Link", "🔗 Web Link", () => emit("AddWebLink")));
    linkGroup.appendChild(createRibbonButton("Page Jump Link", "📄 Page Link", () => emit("AddPageJumpLink")));
    body.appendChild(linkGroup);
  }

  private renderFormsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Form Fields
    const fieldGroup = createRibbonGroup("Form Fields");
    const fldRow1 = document.createElement("div");
    fldRow1.className = "ribbon-group-row";
    fldRow1.appendChild(createRibbonButton("Text Field", "🔤 Text Field", () => emit("AddFormField", "text")));
    fldRow1.appendChild(createRibbonButton("Checkbox", "☑ Checkbox", () => emit("AddFormField", "checkbox")));
    fldRow1.appendChild(createRibbonButton("Radio Button", "🔘 Radio", () => emit("AddFormField", "radio")));
    fieldGroup.appendChild(fldRow1);

    const fldRow2 = document.createElement("div");
    fldRow2.className = "ribbon-group-row";
    fldRow2.appendChild(createRibbonButton("Dropdown", "▼ Dropdown", () => emit("AddFormField", "dropdown")));
    fldRow2.appendChild(createRibbonButton("List Box", "≡ List Box", () => emit("AddFormField", "listbox")));
    fldRow2.appendChild(createRibbonButton("Signature Field", "✒️ Signature", () => emit("AddFormField", "signature")));
    fldRow2.appendChild(createRibbonButton("Button", "🔲 Button", () => emit("AddFormField", "button")));
    fieldGroup.appendChild(fldRow2);
    body.appendChild(fieldGroup);

    // 2. Field Tools
    const fldToolsGroup = createRibbonGroup("Field Tools");
    const fldToolRow = document.createElement("div");
    fldToolRow.className = "ribbon-group-row";
    fldToolRow.appendChild(createRibbonButton("Align Left", "Align Left", () => emit("AlignFormFields", "left")));
    fldToolRow.appendChild(createRibbonButton("Distribute", "Distribute", () => emit("DistributeFormFields")));
    fldToolRow.appendChild(createRibbonButton("Clear Form", "🧹 Clear Form", () => emit("ClearFormData")));
    fldToolRow.appendChild(createRibbonButton("Import Data", "📥 Import", () => emit("ImportFormData")));
    fldToolRow.appendChild(createRibbonButton("Export Data", "📤 Export", () => emit("ExportFormData")));
    fldToolsGroup.appendChild(fldToolRow);
    body.appendChild(fldToolsGroup);
  }

  private renderCommentTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Annotations
    const annotGroup = createRibbonGroup("Annotations");
    const annotRow1 = document.createElement("div");
    annotRow1.className = "ribbon-group-row";
    annotRow1.appendChild(createRibbonButton("Highlight", "🖍️ Highlight", () => emit("SetAnnotTool", "highlight"), true));
    annotRow1.appendChild(createRibbonButton("Underline", "_ Underline", () => emit("SetAnnotTool", "underline"), true));
    annotRow1.appendChild(createRibbonButton("Strikethrough", "~ Strike", () => emit("SetAnnotTool", "strike"), true));
    annotRow1.appendChild(createRibbonButton("Sticky Note", "📝 Note", () => emit("SetAnnotTool", "note")));
    annotGroup.appendChild(annotRow1);

    const annotRow2 = document.createElement("div");
    annotRow2.className = "ribbon-group-row";
    annotRow2.appendChild(createRibbonButton("Pencil Draw", "✏️ Pencil", () => emit("SetAnnotTool", "pencil"), true));
    annotRow2.appendChild(createRibbonButton("Eraser", "🧹 Eraser", () => emit("SetAnnotTool", "eraser"), true));
    annotRow2.appendChild(createRibbonButton("Callout Box", "💬 Callout", () => emit("SetAnnotTool", "callout")));
    annotRow2.appendChild(createRibbonButton("Approved Stamp", "✅ Approved", () => emit("AddStamp", "approved")));
    annotRow2.appendChild(createRibbonButton("Confidential", "🔒 Confidential", () => emit("AddStamp", "confidential")));
    annotGroup.appendChild(annotRow2);
    body.appendChild(annotGroup);

    // 2. Drawing Shapes
    const drawGroup = createRibbonGroup("Drawing");
    const drawRow = document.createElement("div");
    drawRow.className = "ribbon-group-row";
    drawRow.appendChild(createRibbonButton("Line", "/ Line", () => emit("InsertShape", "line")));
    drawRow.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow")));
    drawRow.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect")));
    drawRow.appendChild(createRibbonButton("Oval", "⚪ Oval", () => emit("InsertShape", "ellipse")));
    drawGroup.appendChild(drawRow);
    body.appendChild(drawGroup);
  }

  private renderPageToolsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Manipulation
    const pageGroup = createRibbonGroup("Page Management");
    const pageRow1 = document.createElement("div");
    pageRow1.className = "ribbon-group-row";
    pageRow1.appendChild(createRibbonButton("Rotate 90° CCW", "↶ Rotate Left", () => emit("RotatePage", -90)));
    pageRow1.appendChild(createRibbonButton("Rotate 90° CW", "↷ Rotate Right", () => emit("RotatePage", 90)));
    pageRow1.appendChild(createRibbonButton("Delete Page", "🗑️ Delete Page", () => emit("DeletePage")));
    pageRow1.appendChild(createRibbonButton("Insert Page", "➕ Insert Page", () => emit("InsertPage")));
    pageGroup.appendChild(pageRow1);

    const pageRow2 = document.createElement("div");
    pageRow2.className = "ribbon-group-row";
    pageRow2.appendChild(createRibbonButton("Extract Pages", "📤 Extract", () => emit("ExtractPages")));
    pageRow2.appendChild(createRibbonButton("Split Document", "✂️ Split Doc", () => emit("SplitPdf")));
    pageRow2.appendChild(createRibbonButton("Reorder Pages", "🔀 Reorder", () => emit("ReorderPages")));
    pageRow2.appendChild(createRibbonButton("Crop Page", "📐 Crop Page", () => emit("CropPage")));
    pageGroup.appendChild(pageRow2);
    body.appendChild(pageGroup);
  }

  private renderSignProtectTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Fill & Sign
    const signGroup = createRibbonGroup("Fill & Sign");
    const signRow = document.createElement("div");
    signRow.className = "ribbon-group-row";
    signRow.appendChild(createRibbonButton("Add Signature", "✍️ Add Signature", () => emit("AddSignature")));
    signRow.appendChild(createRibbonButton("Add Initials", "Initials", () => emit("AddInitials")));
    signRow.appendChild(createRibbonButton("Sign Document", "✒️ Sign", () => emit("SignPdf")));
    signGroup.appendChild(signRow);
    body.appendChild(signGroup);

    // 2. Security & Encryption
    const secGroup = createRibbonGroup("Security");
    const secRow = document.createElement("div");
    secRow.className = "ribbon-group-row";
    secRow.appendChild(createRibbonButton("Password Protect", "🔒 Encrypt Password", () => emit("EncryptPassword")));
    secRow.appendChild(createRibbonButton("Remove Security", "🔓 Remove Security", () => emit("RemoveSecurity")));
    secRow.appendChild(createRibbonButton("Restrict Editing", "Restrict Editing", () => emit("RestrictEditing")));
    secGroup.appendChild(secRow);
    body.appendChild(secGroup);

    // 3. Redaction
    const redGroup = createRibbonGroup("Redaction");
    const redRow = document.createElement("div");
    redRow.className = "ribbon-group-row";
    redRow.appendChild(createRibbonButton("Mark Redaction", "⬛ Mark Redact", () => emit("MarkRedaction"), true));
    redRow.appendChild(createRibbonButton("Apply Redactions", "Apply Redactions", () => emit("ApplyRedactions")));
    redGroup.appendChild(redRow);
    body.appendChild(redGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Display
    const dispGroup = createRibbonGroup("Page Display");
    const dispRow = document.createElement("div");
    dispRow.className = "ribbon-group-row";
    dispRow.appendChild(createRibbonButton("Single Page", "📄 Single Page", () => emit("SetPageLayout", "single")));
    dispRow.appendChild(createRibbonButton("Continuous", "📜 Continuous", () => emit("SetPageLayout", "continuous")));
    dispRow.appendChild(createRibbonButton("Two Pages", "📄📄 Two Pages", () => emit("SetPageLayout", "two-page")));
    dispGroup.appendChild(dispRow);
    body.appendChild(dispGroup);

    // 2. Rotation
    const rotGroup = createRibbonGroup("Rotate View");
    const rotRow = document.createElement("div");
    rotRow.className = "ribbon-group-row";
    rotRow.appendChild(createRibbonButton("Rotate View CW", "↷ View CW", () => emit("RotateView", 90)));
    rotRow.appendChild(createRibbonButton("Rotate View CCW", "↶ View CCW", () => emit("RotateView", -90)));
    rotGroup.appendChild(rotRow);
    body.appendChild(rotGroup);

    // 3. Panes
    const paneGroup = createRibbonGroup("Navigation Panes");
    const paneRow = document.createElement("div");
    paneRow.className = "ribbon-group-row";
    paneRow.appendChild(createRibbonButton("Bookmarks", "📑 Bookmarks", () => emit("ToggleBookmarks"), true));
    paneRow.appendChild(createRibbonButton("Thumbnails", "🖼️ Thumbnails", () => emit("ToggleThumbnails"), true));
    paneRow.appendChild(createRibbonButton("Attachments", "📎 Attachments", () => emit("ToggleAttachments"), true));
    paneRow.appendChild(createRibbonButton("Full Screen", "⛶ Full Screen", () => emit("ToggleFullScreen")));
    paneGroup.appendChild(paneRow);
    body.appendChild(paneGroup);
  }
}

