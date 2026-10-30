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
    slidesRow.appendChild(createRibbonButton("New Slide", "➕ New Slide", () => emit("NewSlide"), false, "slides-btn-new-slide"));
    slidesRow.appendChild(createRibbonButton("Layout", "▦ Layout", () => emit("SlideLayout"), false, "slides-btn-layout"));
    slidesRow.appendChild(createRibbonButton("Reset", "🔄 Reset", () => emit("ResetSlide"), false, "slides-btn-reset"));
    slidesRow.appendChild(createRibbonButton("Delete Slide", "🗑️ Delete", () => emit("DeleteSlide"), false, "slides-btn-delete-slide"));
    slidesGroup.appendChild(slidesRow);

    const slidesRow2 = document.createElement("div");
    slidesRow2.className = "ribbon-group-row";
    slidesRow2.appendChild(createRibbonButton("Duplicate", "📋 Duplicate", () => emit("DuplicateSlide"), false, "slides-btn-dup-slide"));
    slidesRow2.appendChild(createRibbonButton("Move Up", "⬆️ Move Up", () => emit("MoveSlideUp"), false, "slides-btn-move-up"));
    slidesRow2.appendChild(createRibbonButton("Move Down", "⬇️ Move Down", () => emit("MoveSlideDown"), false, "slides-btn-move-down"));
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
    fontRow2.appendChild(createRibbonButton("Bold", "B", () => emit("ToggleBold"), true, "slides-btn-bold"));
    fontRow2.appendChild(createRibbonButton("Italic", "I", () => emit("ToggleItalic"), true, "slides-btn-italic"));
    fontRow2.appendChild(createRibbonButton("Underline", "U", () => emit("ToggleUnderline"), true, "slides-btn-underline"));
    fontRow2.appendChild(createRibbonButton("Strikethrough", "S", () => emit("ToggleStrikethrough"), true, "slides-btn-strike"));
    fontRow2.appendChild(createRibbonButton("Font Color", "🎨 Color", () => emit("SetFontColor", "#c43e1c"), false, "slides-btn-font-color"));
    fontRow2.appendChild(createRibbonButton("Clear Formatting", "🧹 Clear", () => emit("ClearFormatting"), false, "slides-btn-clear-format"));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 4. Paragraph Group
    const paraGroup = createRibbonGroup("Paragraph");
    const paraRow1 = document.createElement("div");
    paraRow1.className = "ribbon-group-row";
    paraRow1.appendChild(createRibbonButton("Align Left", "Left", () => emit("AlignLeft"), false, "slides-btn-align-left"));
    paraRow1.appendChild(createRibbonButton("Center", "Center", () => emit("AlignCenter"), false, "slides-btn-align-center"));
    paraRow1.appendChild(createRibbonButton("Align Right", "Right", () => emit("AlignRight"), false, "slides-btn-align-right"));
    paraRow1.appendChild(createRibbonButton("Justify", "Justify", () => emit("AlignJustify"), false, "slides-btn-align-justify"));
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
    drawRow1.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect"), false, "slides-shape-rect"));
    drawRow1.appendChild(createRibbonButton("Rounded", "▢ Round", () => emit("InsertShape", "roundRect"), false, "slides-shape-round-rect"));
    drawRow1.appendChild(createRibbonButton("Oval", "⚪ Oval", () => emit("InsertShape", "ellipse"), false, "slides-shape-circle"));
    drawRow1.appendChild(createRibbonButton("Triangle", "▲ Tri", () => emit("InsertShape", "triangle"), false, "slides-shape-triangle"));
    drawRow1.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow"), false, "slides-shape-arrow"));
    drawRow1.appendChild(createRibbonButton("Star", "★ Star", () => emit("InsertShape", "star"), false, "slides-shape-star"));
    drawRow1.appendChild(createRibbonButton("Callout", "💬 Callout", () => emit("InsertShape", "callout"), false, "slides-shape-callout"));
    drawRow1.appendChild(createRibbonButton("Diamond", "🔶 Diamond", () => emit("InsertShape", "diamond"), false, "slides-shape-diamond"));
    drawRow1.appendChild(createRibbonButton("Heart", "❤️ Heart", () => emit("InsertShape", "heart"), false, "slides-shape-heart"));
    drawGroup.appendChild(drawRow1);

    const drawRow2 = document.createElement("div");
    drawRow2.className = "ribbon-group-row";
    drawRow2.appendChild(createRibbonButton("Blue Fill", "🟦 Blue", () => emit("SetFillColor", "#2563eb"), false, "slides-fill-blue"));
    drawRow2.appendChild(createRibbonButton("Red Fill", "🟥 Red", () => emit("SetFillColor", "#c43e1c"), false, "slides-fill-red"));
    drawRow2.appendChild(createRibbonButton("Green Fill", "🟩 Green", () => emit("SetFillColor", "#10b981"), false, "slides-fill-green"));
    drawRow2.appendChild(createRibbonButton("Yellow Fill", "🟨 Yellow", () => emit("SetFillColor", "#eab308"), false, "slides-fill-yellow"));
    drawRow2.appendChild(createRibbonButton("Dark Fill", "⬛ Dark", () => emit("SetFillColor", "#1e293b"), false, "slides-fill-dark"));
    drawRow2.appendChild(createRibbonButton("White Fill", "⬜ White", () => emit("SetFillColor", "#ffffff"), false, "slides-fill-white"));
    drawRow2.appendChild(createRibbonButton("Outline", "✏️ Outline", () => emit("SetShapeOutline", "#000000"), false, "slides-btn-outline"));
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
    arrangeRow1.appendChild(createRibbonButton("Bring to Front", "⬆ Front", () => emit("BringToFront"), false, "slides-btn-bring-to-front"));
    arrangeRow1.appendChild(createRibbonButton("Send to Back", "⬇ Back", () => emit("SendToBack"), false, "slides-btn-send-to-back"));
    arrangeRow1.appendChild(createRibbonButton("Duplicate", "📋 Dup", () => emit("DuplicateShape"), false, "slides-btn-dup-shape"));
    arrangeRow1.appendChild(createRibbonButton("Delete", "🗑 Del", () => emit("DeleteShape"), false, "slides-btn-del-shape"));
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

    // 8. Copilot (Subphase 20.1.3)
    const copilotGroup = createRibbonGroup("Copilot");
    copilotGroup.appendChild(
      createRibbonButton("Aether Copilot", "✨ Copilot", () => emit("ToggleCopilot"), false, "slides-btn-copilot")
    );
    body.appendChild(copilotGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Slides
    const slidesGroup = createRibbonGroup("Slides");
    slidesGroup.appendChild(createRibbonButton("New Slide", "➕ New Slide", () => emit("NewSlide"), false, "slides-btn-insert-slide"));
    slidesGroup.appendChild(createRibbonButton("Reuse Slides", "🔄 Reuse", () => emit("ReuseSlides")));
    body.appendChild(slidesGroup);

    // 2. Tables
    const tblGroup = createRibbonGroup("Tables");
    tblGroup.appendChild(
      createRibbonButton("Insert Table", "📊 Table", () => emit("InsertTable", { rows: 3, cols: 3 }), false, "slides-btn-insert-table")
    );
    body.appendChild(tblGroup);

    // 3. Images
    const imgGroup = createRibbonGroup("Images");
    const imgRow = document.createElement("div");
    imgRow.className = "ribbon-group-row";
    imgRow.appendChild(createRibbonButton("Pictures", "🖼️ Picture", () => emit("InsertPicture"), false, "slides-btn-insert-picture"));
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
    mediaRow.appendChild(createRibbonButton("Video", "🎬 Video", () => emit("InsertVideo"), false, "slides-btn-insert-video"));
    mediaRow.appendChild(createRibbonButton("Audio", "🔊 Audio", () => emit("InsertAudio"), false, "slides-btn-insert-audio"));
    mediaRow.appendChild(createRibbonButton("Record Screen", "📹 Record", () => emit("RecordScreen")));
    mediaGroup.appendChild(mediaRow);
    body.appendChild(mediaGroup);

    // 6. Text
    const textGroup = createRibbonGroup("Text");
    const textRow = document.createElement("div");
    textRow.className = "ribbon-group-row";
    textRow.appendChild(createRibbonButton("Text Box", "🔤 Text Box", () => emit("InsertTextBox"), false, "slides-btn-insert-textbox"));
    textRow.appendChild(createRibbonButton("WordArt", "🎨 WordArt", () => emit("InsertWordArt")));
    textRow.appendChild(createRibbonButton("Header & Footer", "Header/Footer", () => emit("InsertHeaderFooter")));
    textRow.appendChild(createRibbonButton("Slide Number", "# Slide No", () => emit("InsertSlideNumber")));
    textGroup.appendChild(textRow);
    body.appendChild(textGroup);
  }

  private renderDesignTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Themes Group (8 Curated Slide Themes)
    const themesGroup = createRibbonGroup("Themes");
    const themeRow1 = document.createElement("div");
    themeRow1.className = "ribbon-group-row";
    themeRow1.appendChild(createRibbonButton("Dark Slate", "🌙 Dark Slate", () => emit("SetTheme", "Dark Slate"), false, "slides-theme-dark-slate"));
    themeRow1.appendChild(createRibbonButton("Ocean Blue", "🌊 Ocean Blue", () => emit("SetTheme", "Ocean Blue"), false, "slides-theme-ocean-blue"));
    themeRow1.appendChild(createRibbonButton("Emerald", "🌲 Emerald", () => emit("SetTheme", "Emerald"), false, "slides-theme-emerald"));
    themeRow1.appendChild(createRibbonButton("Sunset", "🌅 Sunset", () => emit("SetTheme", "Sunset"), false, "slides-theme-sunset"));
    themesGroup.appendChild(themeRow1);

    const themeRow2 = document.createElement("div");
    themeRow2.className = "ribbon-group-row";
    themeRow2.appendChild(createRibbonButton("Minimal White", "⚪ Minimal White", () => emit("SetTheme", "Minimal White"), false, "slides-theme-minimal-white"));
    themeRow2.appendChild(createRibbonButton("Executive Navy", "🏛️ Executive Navy", () => emit("SetTheme", "Executive Navy"), false, "slides-theme-executive-navy"));
    themeRow2.appendChild(createRibbonButton("Warm Sand", "🏜️ Warm Sand", () => emit("SetTheme", "Warm Sand"), false, "slides-theme-warm-sand"));
    themeRow2.appendChild(createRibbonButton("Cyber Neon", "⚡ Cyber Neon", () => emit("SetTheme", "Cyber Neon"), false, "slides-theme-cyber-neon"));
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
    custRow.appendChild(createRibbonButton("Standard 4:3", "📐 Standard 4:3", () => emit("SetSlideSize", "4:3"), false, "slides-btn-size-4-3"));
    custRow.appendChild(createRibbonButton("Widescreen 16:9", "📐 Widescreen 16:9", () => emit("SetSlideSize", "16:9"), false, "slides-btn-size-16-9"));
    custRow.appendChild(createRibbonButton("Format Background", "🎨 Background", () => emit("FormatBackground"), false, "slides-btn-format-bg"));
    custRow.appendChild(createRibbonButton("Solid Fill", "⬛ Solid", () => emit("SetSlideBackground", "#0f172a"), false, "slides-btn-solid-bg"));
    custRow.appendChild(createRibbonButton("Gradient Fill", "🌈 Gradient", () => emit("SetSlideBackground", "linear-gradient(135deg, #1e3a8a 0%, #0f172a 100%)"), false, "slides-btn-gradient-bg"));
    custGroup.appendChild(custRow);
    body.appendChild(custGroup);
  }

  private renderTransitionsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Preview Group
    const prevGroup = createRibbonGroup("Preview");
    prevGroup.appendChild(
      createRibbonButton("Preview", "▶ Preview", () => emit("PreviewTransition"), false, "slides-btn-preview-trans")
    );
    body.appendChild(prevGroup);

    // 2. Transition to This Slide
    const transGroup = createRibbonGroup("Transition to This Slide");
    const transRow1 = document.createElement("div");
    transRow1.className = "ribbon-group-row";
    transRow1.appendChild(createRibbonButton("None", "None", () => emit("SetTransition", "None"), false, "slides-trans-none"));
    transRow1.appendChild(createRibbonButton("Fade", "Fade", () => emit("SetTransition", "Fade"), false, "slides-trans-fade"));
    transRow1.appendChild(createRibbonButton("Slide Left", "Slide Left", () => emit("SetTransition", "Slide Left"), false, "slides-trans-slide-left"));
    transRow1.appendChild(createRibbonButton("Zoom", "Zoom", () => emit("SetTransition", "Zoom"), false, "slides-trans-zoom"));
    transRow1.appendChild(createRibbonButton("Flip", "Flip", () => emit("SetTransition", "Flip"), false, "slides-trans-flip"));
    transGroup.appendChild(transRow1);

    const transRow2 = document.createElement("div");
    transRow2.className = "ribbon-group-row";
    ["Push", "Wipe", "Split", "Reveal", "Shape", "Cover"].forEach((t) => {
      transRow2.appendChild(createRibbonButton(t, t, () => emit("SetTransition", t)));
    });
    transGroup.appendChild(transRow2);

    const transRow3 = document.createElement("div");
    transRow3.className = "ribbon-group-row";
    ["Cube", "Vortex", "Honeycomb", "Origami", "Ripple"].forEach((t) => {
      transRow3.appendChild(createRibbonButton(t, `🎲 ${t}`, () => emit("SetTransition", t)));
    });
    transGroup.appendChild(transRow3);
    body.appendChild(transGroup);

    // 3. Effect Options
    const effectGroup = createRibbonGroup("Effect Options");
    const effRow = document.createElement("div");
    effRow.className = "ribbon-group-row";
    effRow.appendChild(createRibbonButton("From Left", "⬅ Left", () => emit("SetEffectOption", "left")));
    effRow.appendChild(createRibbonButton("From Right", "➔ Right", () => emit("SetEffectOption", "right")));
    effRow.appendChild(createRibbonButton("From Top", "⬆ Top", () => emit("SetEffectOption", "top")));
    effRow.appendChild(createRibbonButton("From Bottom", "⬇ Bottom", () => emit("SetEffectOption", "bottom")));
    effectGroup.appendChild(effRow);
    body.appendChild(effectGroup);

    // 4. Timing Group
    const timingGroup = createRibbonGroup("Timing");
    const timeRow1 = document.createElement("div");
    timeRow1.className = "ribbon-group-row";
    timeRow1.appendChild(createRibbonButton("Duration: 0.5s", "0.5s", () => emit("SetTransitionDuration", 0.5)));
    timeRow1.appendChild(createRibbonButton("Duration: 1.0s", "1.0s", () => emit("SetTransitionDuration", 1.0)));
    timeRow1.appendChild(createRibbonButton("Duration: 1.5s", "1.5s", () => emit("SetTransitionDuration", 1.5)));
    timeRow1.appendChild(createRibbonButton("Duration: 2.0s", "2.0s", () => emit("SetTransitionDuration", 2.0)));
    timingGroup.appendChild(timeRow1);

    const timeRow2 = document.createElement("div");
    timeRow2.className = "ribbon-group-row";
    timeRow2.appendChild(createRibbonButton("Apply To All", "🔄 Apply To All", () => emit("ApplyTransitionToAll")));
    timeRow2.appendChild(createRibbonButton("On Mouse Click", "☑ Click", () => emit("ToggleAdvanceOnClick"), true));
    timeRow2.appendChild(createRibbonButton("After Delay", "⏱ After 3s", () => emit("SetAdvanceDelay", 3)));
    timingGroup.appendChild(timeRow2);
    body.appendChild(timingGroup);
  }

  private renderAnimationsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Preview Group
    const prevGroup = createRibbonGroup("Preview");
    prevGroup.appendChild(
      createRibbonButton("Preview", "▶ Preview", () => emit("PreviewAnimation"), false, "slides-btn-preview-anim")
    );
    body.appendChild(prevGroup);

    // 2. Animation Group
    const animGroup = createRibbonGroup("Animation");
    const animRow1 = document.createElement("div");
    animRow1.className = "ribbon-group-row";
    animRow1.appendChild(createRibbonButton("None", "None", () => emit("SetAnimation", "none"), false, "slides-anim-none"));
    animRow1.appendChild(createRibbonButton("Fade In", "🟢 Fade In", () => emit("SetAnimation", "fadeIn"), false, "slides-anim-fade-in"));
    animRow1.appendChild(createRibbonButton("Fly Up", "🟢 Fly Up", () => emit("SetAnimation", "flyUp"), false, "slides-anim-fly-up"));
    animRow1.appendChild(createRibbonButton("Pop", "🟢 Pop", () => emit("SetAnimation", "pop"), false, "slides-anim-pop"));
    animRow1.appendChild(createRibbonButton("Spin", "🟡 Spin", () => emit("SetAnimation", "spin"), false, "slides-anim-spin"));
    animGroup.appendChild(animRow1);

    const animRow2 = document.createElement("div");
    animRow2.className = "ribbon-group-row";
    animRow2.appendChild(createRibbonButton("Appear", "🟢 Appear", () => emit("SetAnimation", "appear")));
    animRow2.appendChild(createRibbonButton("Pulse", "🟡 Pulse", () => emit("SetAnimation", "pulse")));
    animRow2.appendChild(createRibbonButton("Motion Path", "➰ Motion", () => emit("AddMotionPath")));
    animRow2.appendChild(createRibbonButton("Fade Out", "🔴 Fade Out", () => emit("SetAnimation", "fadeOut")));
    animRow2.appendChild(createRibbonButton("Fly Out", "🔴 Fly Out", () => emit("SetAnimation", "flyOut")));
    animGroup.appendChild(animRow2);
    body.appendChild(animGroup);
    animRow2.appendChild(createRibbonButton("Pulse", "🟡 Pulse", () => emit("SetAnimation", "pulse")));
    animRow2.appendChild(createRibbonButton("Spin", "🟡 Spin", () => emit("SetAnimation", "spin")));
    animRow2.appendChild(createRibbonButton("Motion Path", "➰ Motion", () => emit("AddMotionPath")));
    animRow2.appendChild(createRibbonButton("Fade Out", "🔴 Fade Out", () => emit("SetAnimation", "fadeOut")));
    animRow2.appendChild(createRibbonButton("Fly Out", "🔴 Fly Out", () => emit("SetAnimation", "flyOut")));
    animRow2.appendChild(createRibbonButton("None", "None", () => emit("SetAnimation", "none")));
    animGroup.appendChild(animRow2);
    body.appendChild(animGroup);

    // 3. Advanced Animation
    const advGroup = createRibbonGroup("Advanced Animation");
    const advRow = document.createElement("div");
    advRow.className = "ribbon-group-row";
    advRow.appendChild(createRibbonButton("Add Animation", "➕ Add Anim", () => emit("AddAnimation")));
    advRow.appendChild(createRibbonButton("Animation Pane", "🎞️ Anim Pane", () => emit("ToggleAnimationPane"), true));
    advRow.appendChild(createRibbonButton("Trigger", "⚡ Trigger", () => emit("SetAnimationTrigger")));
    advGroup.appendChild(advRow);
    body.appendChild(advGroup);

    // 4. Timing
    const timeGroup = createRibbonGroup("Timing");
    const timeRow1 = document.createElement("div");
    timeRow1.className = "ribbon-group-row";
    timeRow1.appendChild(createRibbonButton("Start: On Click", "On Click", () => emit("SetAnimStart", "click")));
    timeRow1.appendChild(createRibbonButton("With Previous", "With Prev", () => emit("SetAnimStart", "withPrev")));
    timeRow1.appendChild(createRibbonButton("After Previous", "After Prev", () => emit("SetAnimStart", "afterPrev")));
    timeGroup.appendChild(timeRow1);

    const timeRow2 = document.createElement("div");
    timeRow2.className = "ribbon-group-row";
    timeRow2.appendChild(createRibbonButton("Duration: 0.5s", "0.5s", () => emit("SetAnimationDuration", 0.5)));
    timeRow2.appendChild(createRibbonButton("Duration: 1.0s", "1.0s", () => emit("SetAnimationDuration", 1.0)));
    timeRow2.appendChild(createRibbonButton("Delay: 0.25s", "+0.25s", () => emit("SetAnimationDelay", 0.25)));
    timeRow2.appendChild(createRibbonButton("Move Earlier", "▲ Earlier", () => emit("ReorderAnimationEarlier")));
    timeRow2.appendChild(createRibbonButton("Move Later", "▼ Later", () => emit("ReorderAnimationLater")));
    timeGroup.appendChild(timeRow2);
    body.appendChild(timeGroup);
  }

  private renderSlideShowTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Start Slide Show
    const showGroup = createRibbonGroup("Start Slide Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(
      createRibbonButton("From Beginning", "▶️ Beginning (F5)", () => emit("StartShowBeginning"), false, "slides-btn-show-begin")
    );
    showRow.appendChild(
      createRibbonButton("From Current Slide", "⏩ Current (Shift+F5)", () => emit("StartShowCurrent"), false, "slides-btn-show-curr")
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
    const monRow = document.createElement("div");
    monRow.className = "ribbon-group-row";
    monRow.appendChild(
      createRibbonButton("Presenter View", "🖥️ Presenter View", () => emit("StartPresenterView"), false, "slides-btn-presenter-view")
    );
    monRow.appendChild(
      createRibbonButton("Presenter Studio", "🎛️ Studio Dual-Screen", () => emit("StartDualScreenPresenter"), false, "slides-btn-presenter-studio")
    );
    monRow.appendChild(
      createRibbonButton("Use Presenter View", "☑ Presenter View", () => emit("TogglePresenterView"), true)
    );
    monGroup.appendChild(monRow);
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

    // 4. Copilot (Subphase 20.1.3)
    const copilotReviewGroup = createRibbonGroup("Copilot");
    copilotReviewGroup.appendChild(
      createRibbonButton("Aether Copilot", "✨ Copilot", () => emit("ToggleCopilot"), false, "slides-btn-copilot-review")
    );
    body.appendChild(copilotReviewGroup);
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

