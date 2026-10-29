import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";

export class SlidesRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return [
      "Home",
      "Insert",
      "Design",
      "Transitions",
      "Animations",
      "Slide Show",
      "Review",
      "View",
    ];
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Insert":
        return "N";
      case "Design":
        return "G";
      case "Transitions":
        return "T";
      case "Animations":
        return "K";
      case "Slide Show":
        return "S";
      case "Review":
        return "R";
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
      case "Insert":
        this.renderInsertTab(body, emit);
        break;
      case "Design":
        this.renderDesignTab(body, emit);
        break;
      case "Transitions":
        this.renderTransitionsTab(body, emit);
        break;
      case "Animations":
        this.renderAnimationsTab(body, emit);
        break;
      case "Slide Show":
        this.renderSlideShowTab(body, emit);
        break;
      case "Review":
        this.renderReviewTab(body, emit);
        break;
      case "View":
        this.renderViewTab(body, emit);
        break;
      default:
        this.renderHomeTab(body, emit);
    }
  }

  private renderHomeTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Clipboard Group
    const clipGroup = createRibbonGroup("Clipboard");
    clipGroup.appendChild(
      createRibbonButton("Paste", "📋", () => emit("Paste"), false, "slides-btn-paste")
    );
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️", () => emit("Cut")));
    clipRow.appendChild(createRibbonButton("Copy", "📄", () => emit("Copy")));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 2. Slides Group
    const slidesGroup = createRibbonGroup("Slides");
    const slidesRow = document.createElement("div");
    slidesRow.className = "ribbon-group-row";
    slidesRow.appendChild(createRibbonButton("New Slide", "➕ New Slide", () => emit("NewSlide")));
    slidesRow.appendChild(createRibbonButton("Layout", "▦ Layout", () => emit("SlideLayout")));
    slidesRow.appendChild(createRibbonButton("Reset", "🔄 Reset", () => emit("ResetSlide")));
    slidesRow.appendChild(createRibbonButton("Delete Slide", "🗑️ Delete", () => emit("DeleteSlide")));
    slidesGroup.appendChild(slidesRow);

    const slidesRow2 = document.createElement("div");
    slidesRow2.className = "ribbon-group-row";
    slidesRow2.appendChild(createRibbonButton("Duplicate", "📋 Duplicate", () => emit("DuplicateSlide")));
    slidesRow2.appendChild(createRibbonButton("Move Up", "⬆️ Move Up", () => emit("MoveSlideUp")));
    slidesRow2.appendChild(createRibbonButton("Move Down", "⬇️ Move Down", () => emit("MoveSlideDown")));
    slidesGroup.appendChild(slidesRow2);
    body.appendChild(slidesGroup);

    // 3. Font Group
    const fontGroup = createRibbonGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(["Arial", "Calibri", "Century Gothic", "Segoe UI", "Georgia"], "Arial", (val) =>
        emit("SetFontFamily", val)
      )
    );
    fontRow1.appendChild(
      createRibbonSelect([14, 18, 24, 28, 32, 40, 48, 60, 72], 24, (val) =>
        emit("SetFontSize", parseFloat(val))
      )
    );
    fontGroup.appendChild(fontRow1);

    const fontRow2 = document.createElement("div");
    fontRow2.className = "ribbon-group-row";
    fontRow2.appendChild(createRibbonButton("Bold", "B", () => emit("ToggleBold"), true));
    fontRow2.appendChild(createRibbonButton("Italic", "I", () => emit("ToggleItalic"), true));
    fontRow2.appendChild(createRibbonButton("Underline", "U", () => emit("ToggleUnderline"), true));
    fontRow2.appendChild(createRibbonButton("Strikethrough", "S", () => emit("ToggleStrikethrough"), true));
    fontRow2.appendChild(createRibbonButton("Font Color", "🎨 Color", () => emit("SetFontColor", "#c43e1c")));
    fontRow2.appendChild(createRibbonButton("Clear Formatting", "🧹 Clear", () => emit("ClearFormatting")));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 4. Paragraph Group
    const paraGroup = createRibbonGroup("Paragraph");
    const paraRow1 = document.createElement("div");
    paraRow1.className = "ribbon-group-row";
    paraRow1.appendChild(createRibbonButton("Align Left", "Left", () => emit("AlignLeft")));
    paraRow1.appendChild(createRibbonButton("Center", "Center", () => emit("AlignCenter")));
    paraRow1.appendChild(createRibbonButton("Align Right", "Right", () => emit("AlignRight")));
    paraRow1.appendChild(createRibbonButton("Justify", "Justify", () => emit("AlignJustify")));
    paraGroup.appendChild(paraRow1);

    const paraRow2 = document.createElement("div");
    paraRow2.className = "ribbon-group-row";
    paraRow2.appendChild(createRibbonButton("Bullets", "•≡", () => emit("ToggleBulletList")));
    paraRow2.appendChild(createRibbonButton("Numbering", "1≡", () => emit("ToggleNumberedList")));
    paraRow2.appendChild(createRibbonButton("1.0", "1.0", () => emit("SetLineSpacing", 1.0)));
    paraRow2.appendChild(createRibbonButton("1.5", "1.5", () => emit("SetLineSpacing", 1.5)));
    paraGroup.appendChild(paraRow2);
    body.appendChild(paraGroup);

    // 5. Drawing & Shapes (NO Word Normal/Heading styles!)
    const drawGroup = createRibbonGroup("Drawing");
    const drawRow1 = document.createElement("div");
    drawRow1.className = "ribbon-group-row";
    drawRow1.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect")));
    drawRow1.appendChild(createRibbonButton("Rounded", "▢ Round", () => emit("InsertShape", "roundRect")));
    drawRow1.appendChild(createRibbonButton("Oval", "⚪ Oval", () => emit("InsertShape", "ellipse")));
    drawRow1.appendChild(createRibbonButton("Triangle", "▲ Tri", () => emit("InsertShape", "triangle")));
    drawRow1.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow")));
    drawRow1.appendChild(createRibbonButton("Star", "★ Star", () => emit("InsertShape", "star")));
    drawRow1.appendChild(createRibbonButton("Callout", "💬 Callout", () => emit("InsertShape", "callout")));
    drawRow1.appendChild(createRibbonButton("Diamond", "🔶 Diamond", () => emit("InsertShape", "diamond")));
    drawRow1.appendChild(createRibbonButton("Heart", "❤️ Heart", () => emit("InsertShape", "heart")));
    drawGroup.appendChild(drawRow1);

    const drawRow2 = document.createElement("div");
    drawRow2.className = "ribbon-group-row";
    drawRow2.appendChild(createRibbonButton("Blue Fill", "🟦 Blue", () => emit("SetFillColor", "#2563eb")));
    drawRow2.appendChild(createRibbonButton("Red Fill", "🟥 Red", () => emit("SetFillColor", "#c43e1c")));
    drawRow2.appendChild(createRibbonButton("Green Fill", "🟩 Green", () => emit("SetFillColor", "#10b981")));
    drawRow2.appendChild(createRibbonButton("Yellow Fill", "🟨 Yellow", () => emit("SetFillColor", "#eab308")));
    drawRow2.appendChild(createRibbonButton("Dark Fill", "⬛ Dark", () => emit("SetFillColor", "#1e293b")));
    drawRow2.appendChild(createRibbonButton("White Fill", "⬜ White", () => emit("SetFillColor", "#ffffff")));
    drawRow2.appendChild(createRibbonButton("Outline", "✏️ Outline", () => emit("SetShapeOutline", "#000000")));
    drawRow2.appendChild(
      createRibbonSelect([1, 2, 3, 4, 6], 2, (val) =>
        emit("SetOutlineWeight", parseFloat(val))
      )
    );
    drawGroup.appendChild(drawRow2);
    body.appendChild(drawGroup);

    // 6. Arrange Group
    const arrangeGroup = createRibbonGroup("Arrange");
    const arrangeRow1 = document.createElement("div");
    arrangeRow1.className = "ribbon-group-row";
    arrangeRow1.appendChild(createRibbonButton("Bring to Front", "⬆ Front", () => emit("BringToFront")));
    arrangeRow1.appendChild(createRibbonButton("Send to Back", "⬇ Back", () => emit("SendToBack")));
    arrangeRow1.appendChild(createRibbonButton("Duplicate", "📋 Dup", () => emit("DuplicateShape")));
    arrangeRow1.appendChild(createRibbonButton("Delete", "🗑 Del", () => emit("DeleteShape")));
    arrangeGroup.appendChild(arrangeRow1);

    const arrangeRow2 = document.createElement("div");
    arrangeRow2.className = "ribbon-group-row";
    arrangeRow2.appendChild(createRibbonButton("Align Left", "⇤ Left", () => emit("AlignLeft")));
    arrangeRow2.appendChild(createRibbonButton("Align Center", "⇥⇤ Center", () => emit("AlignCenter")));
    arrangeRow2.appendChild(createRibbonButton("Align Right", "⇥ Right", () => emit("AlignRight")));
    arrangeRow2.appendChild(createRibbonButton("Align Middle", "↕ Middle", () => emit("AlignMiddle")));
    arrangeGroup.appendChild(arrangeRow2);
    body.appendChild(arrangeGroup);

    // 7. Editing
    const editGroup = createRibbonGroup("Editing");
    editGroup.appendChild(createRibbonButton("Find", "🔍 Find", () => emit("Find")));
    editGroup.appendChild(createRibbonButton("Replace", "🔄 Replace", () => emit("Replace")));
    editGroup.appendChild(createRibbonButton("Select All", "Select All", () => emit("SelectAll")));
    body.appendChild(editGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Slides
    const slidesGroup = createRibbonGroup("Slides");
    slidesGroup.appendChild(createRibbonButton("New Slide", "➕ New Slide", () => emit("NewSlide")));
    slidesGroup.appendChild(createRibbonButton("Reuse Slides", "🔄 Reuse", () => emit("ReuseSlides")));
    body.appendChild(slidesGroup);

    // 2. Tables
    const tblGroup = createRibbonGroup("Tables");
    tblGroup.appendChild(
      createRibbonButton("Insert Table", "📊 Table", () => emit("InsertTable", { rows: 3, cols: 3 }))
    );
    body.appendChild(tblGroup);

    // 3. Images
    const imgGroup = createRibbonGroup("Images");
    const imgRow = document.createElement("div");
    imgRow.className = "ribbon-group-row";
    imgRow.appendChild(createRibbonButton("Pictures", "🖼️ Picture", () => emit("InsertPicture")));
    imgRow.appendChild(createRibbonButton("Screenshot", "📸 Screenshot", () => emit("InsertScreenshot")));
    imgGroup.appendChild(imgRow);
    body.appendChild(imgGroup);

    // 4. Illustrations
    const illGroup = createRibbonGroup("Illustrations");
    const illRow = document.createElement("div");
    illRow.className = "ribbon-group-row";
    illRow.appendChild(createRibbonButton("Shapes", "⬛ Shapes", () => emit("InsertShape", "rect")));
    illRow.appendChild(createRibbonButton("Icons", "💡 Icons", () => emit("InsertIcons")));
    illRow.appendChild(createRibbonButton("3D Models", "🧊 3D Models", () => emit("Insert3DModels")));
    illRow.appendChild(createRibbonButton("Chart", "📈 Chart", () => emit("InsertChart")));
    illGroup.appendChild(illRow);
    body.appendChild(illGroup);

    // 5. Media
    const mediaGroup = createRibbonGroup("Media");
    const mediaRow = document.createElement("div");
    mediaRow.className = "ribbon-group-row";
    mediaRow.appendChild(createRibbonButton("Video", "🎬 Video", () => emit("InsertVideo")));
    mediaRow.appendChild(createRibbonButton("Audio", "🔊 Audio", () => emit("InsertAudio")));
    mediaRow.appendChild(createRibbonButton("Record Screen", "📹 Record", () => emit("RecordScreen")));
    mediaGroup.appendChild(mediaRow);
    body.appendChild(mediaGroup);

    // 6. Text
    const textGroup = createRibbonGroup("Text");
    const textRow = document.createElement("div");
    textRow.className = "ribbon-group-row";
    textRow.appendChild(createRibbonButton("Text Box", "🔤 Text Box", () => emit("InsertTextBox")));
    textRow.appendChild(createRibbonButton("WordArt", "🎨 WordArt", () => emit("InsertWordArt")));
    textRow.appendChild(createRibbonButton("Header & Footer", "Header/Footer", () => emit("InsertHeaderFooter")));
    textRow.appendChild(createRibbonButton("Slide Number", "# Slide No", () => emit("InsertSlideNumber")));
    textGroup.appendChild(textRow);
    body.appendChild(textGroup);
  }

  private renderDesignTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Themes Group (8 Built-in Themes)
    const themesGroup = createRibbonGroup("Themes");
    const themeRow1 = document.createElement("div");
    themeRow1.className = "ribbon-group-row";
    themeRow1.appendChild(createRibbonButton("Office Default", "🎨 Office", () => emit("SetTheme", "OfficeDefault")));
    themeRow1.appendChild(createRibbonButton("Dark Mode", "🌙 Dark", () => emit("SetTheme", "DarkMode")));
    themeRow1.appendChild(createRibbonButton("Vibrant Coral", "🔥 Coral", () => emit("SetTheme", "VibrantCoral")));
    themeRow1.appendChild(createRibbonButton("High Contrast", "⚡ Contrast", () => emit("SetTheme", "HighContrast")));
    themesGroup.appendChild(themeRow1);

    const themeRow2 = document.createElement("div");
    themeRow2.className = "ribbon-group-row";
    themeRow2.appendChild(createRibbonButton("Emerald Forest", "🌲 Emerald", () => emit("SetTheme", "EmeraldForest")));
    themeRow2.appendChild(createRibbonButton("Ocean Breeze", "🌊 Ocean", () => emit("SetTheme", "OceanBreeze")));
    themeRow2.appendChild(createRibbonButton("Sunset Glow", "🌅 Sunset", () => emit("SetTheme", "SunsetGlow")));
    themeRow2.appendChild(createRibbonButton("Slate Minimal", "🏛️ Slate", () => emit("SetTheme", "SlateMinimal")));
    themesGroup.appendChild(themeRow2);
    body.appendChild(themesGroup);

    // 2. Variants Group
    const variantsGroup = createRibbonGroup("Variants");
    const varRow = document.createElement("div");
    varRow.className = "ribbon-group-row";
    varRow.appendChild(createRibbonButton("Blue", "🟦 Blue", () => emit("SetVariant", "Blue")));
    varRow.appendChild(createRibbonButton("Emerald", "🟩 Emerald", () => emit("SetVariant", "Emerald")));
    varRow.appendChild(createRibbonButton("Coral", "🔥 Coral", () => emit("SetVariant", "Coral")));
    varRow.appendChild(createRibbonButton("Purple", "🟪 Purple", () => emit("SetVariant", "Purple")));
    variantsGroup.appendChild(varRow);
    body.appendChild(variantsGroup);

    // 3. Customize Group (Slide Size & Background)
    const custGroup = createRibbonGroup("Customize");
    const custRow = document.createElement("div");
    custRow.className = "ribbon-group-row";
    custRow.appendChild(createRibbonButton("Standard 4:3", "📐 Standard 4:3", () => emit("SetSlideSize", "4:3")));
    custRow.appendChild(createRibbonButton("Widescreen 16:9", "📐 Widescreen 16:9", () => emit("SetSlideSize", "16:9")));
    custRow.appendChild(createRibbonButton("Format Background", "🎨 Background", () => emit("FormatBackground")));
    custGroup.appendChild(custRow);
    body.appendChild(custGroup);
  }

  private renderTransitionsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Transition to This Slide
    const transGroup = createRibbonGroup("Transition to This Slide");
    const transRow1 = document.createElement("div");
    transRow1.className = "ribbon-group-row";
    ["None", "Morph", "Fade", "Push", "Wipe"].forEach((t) => {
      transRow1.appendChild(createRibbonButton(t, t, () => emit("SetTransition", t)));
    });
    transGroup.appendChild(transRow1);

    const transRow2 = document.createElement("div");
    transRow2.className = "ribbon-group-row";
    ["Split", "Reveal", "Random Bars", "Shape", "Cover"].forEach((t) => {
      transRow2.appendChild(createRibbonButton(t, t, () => emit("SetTransition", t)));
    });
    transGroup.appendChild(transRow2);
    body.appendChild(transGroup);

    // 2. Effect Options
    const effectGroup = createRibbonGroup("Effect Options");
    const effRow = document.createElement("div");
    effRow.className = "ribbon-group-row";
    effRow.appendChild(createRibbonButton("From Left", "⬅ Left", () => emit("SetEffectOption", "left")));
    effRow.appendChild(createRibbonButton("From Right", "➔ Right", () => emit("SetEffectOption", "right")));
    effRow.appendChild(createRibbonButton("From Top", "⬆ Top", () => emit("SetEffectOption", "top")));
    effRow.appendChild(createRibbonButton("From Bottom", "⬇ Bottom", () => emit("SetEffectOption", "bottom")));
    effectGroup.appendChild(effRow);
    body.appendChild(effectGroup);

    // 3. Timing Group
    const timingGroup = createRibbonGroup("Timing");
    const timeRow = document.createElement("div");
    timeRow.className = "ribbon-group-row";
    timeRow.appendChild(createRibbonButton("Apply To All", "🔄 Apply To All", () => emit("ApplyTransitionToAll")));
    timeRow.appendChild(createRibbonButton("On Mouse Click", "☑ Click", () => emit("ToggleAdvanceOnClick"), true));
    timeRow.appendChild(createRibbonButton("After Delay", "⏱ After 3s", () => emit("SetAdvanceDelay", 3)));
    timingGroup.appendChild(timeRow);
    body.appendChild(timingGroup);
  }

  private renderAnimationsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Animation Group
    const animGroup = createRibbonGroup("Animation");
    const animRow1 = document.createElement("div");
    animRow1.className = "ribbon-group-row";
    ["None", "Appear", "Fade", "Fly In", "Float In"].forEach((a) => {
      animRow1.appendChild(createRibbonButton(a, a, () => emit("SetAnimation", a)));
    });
    animGroup.appendChild(animRow1);

    const animRow2 = document.createElement("div");
    animRow2.className = "ribbon-group-row";
    ["Split", "Wipe", "Shape", "Wheel", "Bounce"].forEach((a) => {
      animRow2.appendChild(createRibbonButton(a, a, () => emit("SetAnimation", a)));
    });
    animGroup.appendChild(animRow2);
    body.appendChild(animGroup);

    // 2. Advanced Animation
    const advGroup = createRibbonGroup("Advanced Animation");
    const advRow = document.createElement("div");
    advRow.className = "ribbon-group-row";
    advRow.appendChild(createRibbonButton("Add Animation", "➕ Add Anim", () => emit("AddAnimation")));
    advRow.appendChild(createRibbonButton("Animation Pane", "🎞️ Anim Pane", () => emit("ToggleAnimationPane"), true));
    advRow.appendChild(createRibbonButton("Trigger", "⚡ Trigger", () => emit("SetAnimationTrigger")));
    advGroup.appendChild(advRow);
    body.appendChild(advGroup);

    // 3. Timing
    const timeGroup = createRibbonGroup("Timing");
    const timeRow = document.createElement("div");
    timeRow.className = "ribbon-group-row";
    timeRow.appendChild(createRibbonButton("Start: On Click", "On Click", () => emit("SetAnimStart", "click")));
    timeRow.appendChild(createRibbonButton("With Previous", "With Prev", () => emit("SetAnimStart", "withPrev")));
    timeRow.appendChild(createRibbonButton("After Previous", "After Prev", () => emit("SetAnimStart", "afterPrev")));
    timeGroup.appendChild(timeRow);
    body.appendChild(timeGroup);
  }

  private renderSlideShowTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Start Slide Show
    const showGroup = createRibbonGroup("Start Slide Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(
      createRibbonButton("From Beginning", "▶️ Beginning (F5)", () => emit("StartShowBeginning"))
    );
    showRow.appendChild(
      createRibbonButton("From Current Slide", "⏩ Current (Shift+F5)", () => emit("StartShowCurrent"))
    );
    showRow.appendChild(createRibbonButton("Present Online", "🌐 Online", () => emit("PresentOnline")));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 2. Set Up
    const setupGroup = createRibbonGroup("Set Up");
    const setupRow = document.createElement("div");
    setupRow.className = "ribbon-group-row";
    setupRow.appendChild(createRibbonButton("Set Up Show", "⚙️ Set Up Show", () => emit("SetUpShow")));
    setupRow.appendChild(createRibbonButton("Hide Slide", "👁️‍🗨️ Hide Slide", () => emit("HideSlide")));
    setupRow.appendChild(createRibbonButton("Rehearse Timings", "⏱️ Rehearse", () => emit("RehearseTimings")));
    setupRow.appendChild(createRibbonButton("Record Show", "🔴 Record", () => emit("RecordShow")));
    setupGroup.appendChild(setupRow);
    body.appendChild(setupGroup);

    // 3. Monitors
    const monGroup = createRibbonGroup("Monitors");
    monGroup.appendChild(
      createRibbonButton("Use Presenter View", "☑ Presenter View", () => emit("TogglePresenterView"), true)
    );
    body.appendChild(monGroup);
  }

  private renderReviewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Proofing
    const proofGroup = createRibbonGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(createRibbonButton("Spelling", "✓ Spelling", () => emit("CheckSpelling")));
    proofRow.appendChild(createRibbonButton("Thesaurus", "📖 Thesaurus", () => emit("Thesaurus")));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    // 2. Comments
    const commGroup = createRibbonGroup("Comments");
    commGroup.appendChild(createRibbonButton("New Comment", "💬 New Comment", () => emit("NewComment")));
    commGroup.appendChild(createRibbonButton("Show Comments", "Show Comments", () => emit("ShowComments"), true));
    body.appendChild(commGroup);

    // 3. Compare
    const compGroup = createRibbonGroup("Compare");
    compGroup.appendChild(createRibbonButton("Compare Presentations", "📑 Compare", () => emit("Compare")));
    body.appendChild(compGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Presentation Views
    const viewsGroup = createRibbonGroup("Presentation Views");
    const viewsRow = document.createElement("div");
    viewsRow.className = "ribbon-group-row";
    viewsRow.appendChild(createRibbonButton("Normal View", "📄 Normal", () => emit("ViewNormal")));
    viewsRow.appendChild(createRibbonButton("Slide Sorter", "▦ Sorter", () => emit("ViewSlideSorter")));
    viewsRow.appendChild(createRibbonButton("Notes Page", "📝 Notes", () => emit("ViewNotesPage")));
    viewsRow.appendChild(createRibbonButton("Reading View", "📖 Reading", () => emit("ViewReadingView")));
    viewsGroup.appendChild(viewsRow);
    body.appendChild(viewsGroup);

    // 2. Master Views
    const masterGroup = createRibbonGroup("Master Views");
    const masterRow = document.createElement("div");
    masterRow.className = "ribbon-group-row";
    masterRow.appendChild(createRibbonButton("Slide Master", "Slide Master", () => emit("ViewSlideMaster")));
    masterRow.appendChild(createRibbonButton("Handout Master", "Handout Master", () => emit("ViewHandoutMaster")));
    masterGroup.appendChild(masterRow);
    body.appendChild(masterGroup);

    // 3. Show
    const showGroup = createRibbonGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(createRibbonButton("Ruler", "📏 Ruler", () => emit("ToggleRuler"), true));
    showRow.appendChild(createRibbonButton("Gridlines", "🔲 Gridlines", () => emit("ToggleGridlines"), true));
    showRow.appendChild(createRibbonButton("Guides", "Guides", () => emit("ToggleGuides"), true));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 4. Color / Grayscale
    const colorGroup = createRibbonGroup("Color / Grayscale");
    const colRow = document.createElement("div");
    colRow.className = "ribbon-group-row";
    colRow.appendChild(createRibbonButton("Color", "🎨 Color", () => emit("SetColorMode", "color")));
    colRow.appendChild(createRibbonButton("Grayscale", "Grayscale", () => emit("SetColorMode", "grayscale")));
    colRow.appendChild(createRibbonButton("Black & White", "B&W", () => emit("SetColorMode", "bw")));
    colorGroup.appendChild(colRow);
    body.appendChild(colorGroup);

    // 5. Zoom
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(createRibbonButton("Fit Window", "↔️ Fit Window", () => emit("ZoomFit")));
    zoomRow.appendChild(createRibbonButton("100%", "🔍 100%", () => emit("Zoom100")));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);
  }
}

