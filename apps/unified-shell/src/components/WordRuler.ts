export class WordRuler {
  private container: HTMLElement;
  private marginLeft = 72; // points or px
  private marginRight = 72;
  private firstLineIndent = 0;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`WordRuler container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setMargins(left: number, right: number, indent = 0) {
    this.marginLeft = left;
    this.marginRight = right;
    this.firstLineIndent = indent;
    this.render();
  }

  private render() {
    this.container.innerHTML = "";

    const rulerBox = document.createElement("div");
    rulerBox.className = "word-ruler-container";

    const track = document.createElement("div");
    track.className = "ruler-track";

    // Left margin block
    const leftMargin = document.createElement("div");
    leftMargin.className = "ruler-margin-box";
    leftMargin.style.left = "0px";
    leftMargin.style.width = `${this.marginLeft}px`;

    // Right margin block
    const rightMargin = document.createElement("div");
    rightMargin.className = "ruler-margin-box";
    rightMargin.style.right = "0px";
    rightMargin.style.width = `${this.marginRight}px`;

    // First line indent slider marker
    const indentSlider = document.createElement("div");
    indentSlider.className = "ruler-indent-slider";
    indentSlider.style.left = `${this.marginLeft + this.firstLineIndent}px`;
    indentSlider.title = "First Line Indent";

    track.appendChild(leftMargin);
    track.appendChild(rightMargin);
    track.appendChild(indentSlider);
    rulerBox.appendChild(track);

    this.container.appendChild(rulerBox);
  }
}
