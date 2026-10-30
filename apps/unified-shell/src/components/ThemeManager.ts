export type AppTheme = "light" | "dark";

export class ThemeManager {
  private currentTheme: AppTheme = "light";
  private toggleBtn: HTMLButtonElement | null = null;
  private buttonSelector: string;
  private static readonly STORAGE_KEY = "aether_theme";

  constructor(buttonSelector: string = "#btn-theme-toggle") {
    this.buttonSelector = buttonSelector;
    this.init(buttonSelector);
  }

  public init(buttonSelector?: string) {
    if (buttonSelector) {
      this.buttonSelector = buttonSelector;
    }
    if (typeof document !== "undefined") {
      this.toggleBtn = document.querySelector<HTMLButtonElement>(this.buttonSelector);
    }
    let saved = "light";
    try {
      if (typeof window !== "undefined" && window.localStorage) {
        saved = window.localStorage.getItem(ThemeManager.STORAGE_KEY) || "light";
      }
    } catch {
      saved = "light";
    }

    this.setTheme(saved === "dark" ? "dark" : "light");

    if (this.toggleBtn) {
      this.toggleBtn.onclick = () => {
        this.toggleTheme();
      };
    }
  }

  public getTheme(): AppTheme {
    return this.currentTheme;
  }

  public setTheme(theme: AppTheme) {
    this.currentTheme = theme;
    if (typeof document !== "undefined") {
      document.documentElement.dataset.theme = theme;
      if (theme === "dark") {
        document.documentElement.classList.add("theme-dark");
        document.documentElement.classList.remove("theme-light");
        document.body.classList.add("theme-dark");
        document.body.classList.remove("theme-light");
      } else {
        document.documentElement.classList.add("theme-light");
        document.documentElement.classList.remove("theme-dark");
        document.body.classList.add("theme-light");
        document.body.classList.remove("theme-dark");
      }
      const appEl = document.getElementById("app");
      if (appEl) {
        if (theme === "dark") {
          appEl.classList.add("theme-dark");
          appEl.classList.remove("theme-light");
        } else {
          appEl.classList.add("theme-light");
          appEl.classList.remove("theme-dark");
        }
      }
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("themechange", { detail: { theme } }));
      }
    }
    try {
      if (typeof window !== "undefined" && window.localStorage) {
        window.localStorage.setItem(ThemeManager.STORAGE_KEY, theme);
      }
    } catch {
      // Ignored in test/sandboxed env
    }
    this.updateButtonUI();
  }

  public toggleTheme(): AppTheme {
    const nextTheme: AppTheme = this.currentTheme === "light" ? "dark" : "light";
    this.setTheme(nextTheme);
    return nextTheme;
  }

  private updateButtonUI() {
    if (!this.toggleBtn) return;
    const isDark = this.currentTheme === "dark";
    this.toggleBtn.innerHTML = `
      <span class="theme-icon">${isDark ? "🌙" : "☀️"}</span>
      <span class="theme-label">${isDark ? "Dark" : "Light"}</span>
    `;
    this.toggleBtn.title = isDark ? "Switch to Light Mode" : "Switch to Dark Mode";
    this.toggleBtn.setAttribute("aria-label", this.toggleBtn.title);
  }
}

export const themeManager = new ThemeManager();

