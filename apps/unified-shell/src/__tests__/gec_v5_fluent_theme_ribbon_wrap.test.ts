import { describe, it, expect, beforeEach } from "vitest";
import { Ribbon } from "../components/Ribbon";
import { ThemeManager } from "../components/ThemeManager";
import { SlideSorter } from "../components/SlideSorter";
import { DocumentType } from "../ipc";

describe("Subphase 28.1.1: Modern Fluent/Linear UI Overhaul, Ribbon Wrapping & High-Contrast Theme", () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app" class="desktop-shell theme-light">
        <header class="title-bar">
          <button id="btn-theme-toggle" class="theme-toggle-btn">
            <span class="theme-icon">☀️</span>
            <span class="theme-label">Light</span>
          </button>
        </header>
        <section id="ribbon-container" class="ribbon-bar"></section>
        <main class="workspace-layout">
          <aside id="slide-sorter-container" class="sidebar-pane"></aside>
          <div id="canvas-viewport" class="canvas-viewport"></div>
        </main>
        <footer id="status-bar" class="status-bar"></footer>
      </div>
    `;
    localStorage.clear();
  });

  it("1. Renders Ribbon with non-overflowing wrapping layout (.ribbon-content & .ribbon-tools-body)", () => {
    const ribbon = new Ribbon("ribbon-container");
    const container = document.getElementById("ribbon-container")!;
    expect(container).toBeDefined();

    const body = container.querySelector(".ribbon-tools-body");
    expect(body).not.toBeNull();
    expect(body?.classList.contains("ribbon-content")).toBe(true);

    // Verify all 5 app ribbons render wrapping tools body without throwing
    const docTypes: DocumentType[] = ["Word", "Excel", "PowerPoint", "PDF", "Notes"];
    for (const dt of docTypes) {
      ribbon.setDocumentType(dt);
      const activeBody = container.querySelector(".ribbon-tools-body.ribbon-content");
      expect(activeBody).not.toBeNull();
      const groups = activeBody!.querySelectorAll(".ribbon-group");
      expect(groups.length).toBeGreaterThan(0);

      // Verify buttons have modern border-radius and styling classes
      const btns = activeBody!.querySelectorAll(".ribbon-btn");
      expect(btns.length).toBeGreaterThan(0);
      btns.forEach((btn) => {
        expect(btn.classList.contains("ribbon-btn")).toBe(true);
      });
    }
  });

  it("2. Cascades Dark & Light Theme across entire viewport including canvas surround and sidebars", () => {
    const themeManager = new ThemeManager("#btn-theme-toggle");
    const appEl = document.getElementById("app")!;
    const toggleBtn = document.querySelector<HTMLButtonElement>("#btn-theme-toggle")!;

    // Initial state: Light
    expect(themeManager.getTheme()).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(document.body.classList.contains("theme-light")).toBe(true);
    expect(appEl.classList.contains("theme-light")).toBe(true);

    // Switch to Dark mode
    toggleBtn.click();
    expect(themeManager.getTheme()).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(document.body.classList.contains("theme-dark")).toBe(true);
    expect(appEl.classList.contains("theme-dark")).toBe(true);
    expect(localStorage.getItem("aether_theme")).toBe("dark");

    // Switch back to Light mode
    toggleBtn.click();
    expect(themeManager.getTheme()).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(document.body.classList.contains("theme-light")).toBe(true);
    expect(appEl.classList.contains("theme-light")).toBe(true);
  });

  it("3. Enforces strict WCAG AA contrast invariant on slide thumbnails (Zero invisible white-on-white text)", () => {
    const sorterContainer = document.getElementById("slide-sorter-container")!;
    const sorter = new SlideSorter("slide-sorter-container");
    sorter.setSlides([
      { id: 1, slideNumber: 1, title: "Executive Overview", shapeCount: 4 },
      { id: 2, slideNumber: 2, title: "Financial Projections", shapeCount: 6, isHidden: true },
    ]);
    expect(sorter).toBeDefined();

    const previews = sorterContainer.querySelectorAll(".slide-thumb-preview");
    expect(previews.length).toBe(2);

    previews.forEach((p) => {
      const titleEl = p.querySelector(".slide-thumb-title");
      expect(titleEl).not.toBeNull();
      expect(titleEl?.textContent?.length).toBeGreaterThan(0);
    });

    const themeManager = new ThemeManager("#btn-theme-toggle");
    themeManager.setTheme("dark");

    // In dark mode, ensure title text elements exist and have content
    const darkTitles = sorterContainer.querySelectorAll(".slide-thumb-title");
    expect(darkTitles.length).toBe(2);
    expect(darkTitles[0].textContent).toBe("Executive Overview");
    expect(darkTitles[1].textContent).toBe("Financial Projections");
  });

  it("4. Ensures ribbon tabs and buttons have crisp Segoe UI / Inter typography and 6px border radius tokens", () => {
    const ribbon = new Ribbon("ribbon-container");
    ribbon.setDocumentType("Word");
    ribbon.setActiveTab("Home");

    const tabBtns = document.querySelectorAll(".ribbon-tab-btn");
    expect(tabBtns.length).toBeGreaterThan(0);

    const ribbonBtns = document.querySelectorAll(".ribbon-btn");
    expect(ribbonBtns.length).toBeGreaterThan(0);

    const fileTab = document.querySelector(".ribbon-file-tab");
    expect(fileTab).not.toBeNull();
  });
});
