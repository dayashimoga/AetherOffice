export type RibbonActionEmitter = (action: string, payload?: unknown) => void;

export interface AppRibbonComponent {
  getTabs(): string[];
  getKeytip(tab: string): string;
  renderTab(tab: string, body: HTMLElement, emit: RibbonActionEmitter): void;
}

export function createRibbonGroup(title: string): HTMLElement {
  const group = document.createElement("div");
  group.className = "ribbon-group";
  const titleEl = document.createElement("span");
  titleEl.className = "ribbon-group-title";
  titleEl.textContent = title;
  group.appendChild(titleEl);
  return group;
}

export function createRibbonButton(
  label: string,
  icon: string,
  onClick: () => void,
  isToggle = false,
  testId?: string
): HTMLElement {
  const btn = document.createElement("button");
  btn.className = "ribbon-btn";
  btn.title = label;
  btn.textContent = icon;
  if (testId) {
    btn.setAttribute("data-testid", testId);
  }
  btn.onmousedown = (e) => e.preventDefault();
  btn.onclick = () => {
    if (isToggle) btn.classList.toggle("active");
    onClick();
  };
  return btn;
}

export function createRibbonSelect(
  options: (string | number)[],
  defaultValue: string | number,
  onChange: (val: string) => void,
  testId?: string
): HTMLElement {
  const select = document.createElement("select");
  select.className = "ribbon-select";
  if (testId) {
    select.setAttribute("data-testid", testId);
  }
  options.forEach((opt) => {
    const el = document.createElement("option");
    el.value = String(opt);
    el.textContent = String(opt);
    if (String(opt) === String(defaultValue)) {
      el.selected = true;
    }
    select.appendChild(el);
  });
  select.onchange = () => onChange(select.value);
  return select;
}

