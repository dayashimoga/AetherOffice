export interface HeadingItem {
  level: number;
  text: string;
}

export class NavigationPane {
  private container: HTMLElement;
  private headings: HeadingItem[] = [];
  private onSelectHeadingCallback?: (index: number) => void;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`NavigationPane container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setHeadings(headings: HeadingItem[]) {
    this.headings = headings;
    this.render();
  }

  public setOnSelectHeading(callback: (index: number) => void) {
    this.onSelectHeadingCallback = callback;
  }

  private render() {
    this.container.innerHTML = "";

    const header = document.createElement("div");
    header.className = "sidebar-heading";
    header.textContent = "Navigation";
    this.container.appendChild(header);

    const list = document.createElement("ul");
    list.className = "nav-outline-list";

    if (this.headings.length === 0) {
      const empty = document.createElement("li");
      empty.style.color = "#8a8a8a";
      empty.style.padding = "8px";
      empty.textContent = "No headings found in document.";
      list.appendChild(empty);
    } else {
      this.headings.forEach((h, idx) => {
        const item = document.createElement("li");
        item.className = `nav-outline-item nav-outline-level-${Math.min(h.level, 3)}`;
        item.textContent = h.text;
        item.onclick = () => {
          if (this.onSelectHeadingCallback) this.onSelectHeadingCallback(idx);
        };
        list.appendChild(item);
      });
    }

    this.container.appendChild(list);
  }
}
