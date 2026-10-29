export interface DetailedWordCount {
  pages: number;
  words: number;
  charactersNoSpaces: number;
  charactersWithSpaces: number;
  paragraphs: number;
  lines: number;
}

export class WordCountDialog {
  private element: HTMLElement;
  private isVisible = false;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "word-count-modal";
    this.element.id = "word-count-modal";
    this.applyStyles();
    document.body.appendChild(this.element);
  }

  public show(stats: DetailedWordCount) {
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render(stats);
  }

  public hide() {
    this.isVisible = false;
    this.element.style.display = "none";
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  private applyStyles() {
    this.element.style.display = "none";
    this.element.style.position = "fixed";
    this.element.style.top = "0";
    this.element.style.left = "0";
    this.element.style.width = "100vw";
    this.element.style.height = "100vh";
    this.element.style.backgroundColor = "rgba(0, 0, 0, 0.4)";
    this.element.style.zIndex = "2000";
    this.element.style.alignItems = "center";
    this.element.style.justifyContent = "center";
    this.element.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
  }

  private render(stats: DetailedWordCount) {
    this.element.innerHTML = `
      <div class="word-count-card" style="background: #ffffff; border-radius: 8px; width: 340px; box-shadow: 0 12px 32px rgba(0,0,0,0.25); overflow: hidden; border: 1px solid #e5e7eb;">
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 14px 18px; border-bottom: 1px solid #f3f4f6; background: #f9fafb;">
          <span style="font-weight: 600; font-size: 14px; color: #111827;">Word Count Statistics</span>
          <button class="btn-close-wc" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
        </div>
        <div style="padding: 16px 20px; display: flex; flex-direction: column; gap: 10px; font-size: 13px; color: #374151;">
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Pages</span><strong class="wc-pages">${stats.pages}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Words</span><strong class="wc-words">${stats.words}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Characters (no spaces)</span><strong class="wc-chars-no-space">${stats.charactersNoSpaces}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Characters (with spaces)</span><strong class="wc-chars-with-space">${stats.charactersWithSpaces}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Paragraphs</span><strong class="wc-paragraphs">${stats.paragraphs}</strong></div>
          <div style="display: flex; justify-content: space-between;"><span style="color: #6b7280;">Lines</span><strong class="wc-lines">${stats.lines}</strong></div>
        </div>
        <div style="display: flex; justify-content: flex-end; padding: 12px 18px; border-top: 1px solid #f3f4f6; background: #f9fafb;">
          <button class="btn-wc-ok" style="padding: 6px 16px; border: 1px solid #d1d5db; border-radius: 4px; background: #2563eb; color: #ffffff; cursor: pointer; font-size: 13px; font-weight: 500;">Close</button>
        </div>
      </div>
    `;

    const closeBtn = this.element.querySelector<HTMLButtonElement>(".btn-close-wc")!;
    const okBtn = this.element.querySelector<HTMLButtonElement>(".btn-wc-ok")!;
    closeBtn.onclick = () => this.hide();
    okBtn.onclick = () => this.hide();
  }
}
