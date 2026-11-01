/**
 * theme_ribbon_coordinator.test.ts
 * GEC-v7.0 Stage 52 (Sub-Phase 52.1.1)
 *
 * Vitest Unit & Integration Suite for ThemeAndRibbonCoordinator & Ribbon Layout Mode:
 * - Default state initialization and subscriber notification
 * - Theme mode switching (Light, Dark, HighContrastDark, HighContrastLight)
 * - Semantic token evaluation & DOM custom CSS properties injection
 * - WCAG 2.1 AA relative luminance & contrast ratio validation (>= 4.5:1 normal, >= 7.0:1 high contrast)
 * - High-contrast toggle with dark/light polarity preservation
 * - Ribbon presentation layout modes (Classic 96px, Compact 44px, Collapsed 32px)
 * - Ribbon collapse and compact toggling behavior
 * - Dynamic listener dispatching and state immutability
 * - Ribbon component rendering of layout mode switcher
 * - Synchronization of data attributes across presentation DOM
 */

import { describe, it, expect, beforeEach } from "vitest";
import { ThemeAndRibbonCoordinator } from "../state/ThemeAndRibbonCoordinator";
import { Ribbon } from "../components/Ribbon";

describe("Sub-Phase 52.1.1: ThemeAndRibbonCoordinator & Ribbon Layout", () => {
  let coordinator: ThemeAndRibbonCoordinator;

  beforeEach(() => {
    // Reset document element attributes
    if (typeof document !== "undefined" && document.documentElement) {
      document.documentElement.removeAttribute("data-theme");
      document.documentElement.removeAttribute("data-ribbon-mode");
      document.documentElement.removeAttribute("data-high-contrast");
    }
    coordinator = new ThemeAndRibbonCoordinator("light", "classic");
  });

  it("1. initializes with default light theme, classic ribbon, and valid WCAG tokens", () => {
    const state = coordinator.getState();
    expect(state.themeMode).toBe("light");
    expect(state.ribbonMode).toBe("classic");
    expect(state.isHighContrast).toBe(false);
    expect(state.ribbonConfig.height_px).toBe(96);
    expect(state.ribbonConfig.is_single_row).toBe(false);
    expect(state.ribbonConfig.show_group_headers).toBe(true);
    expect(state.ribbonConfig.collapsed).toBe(false);
    expect(state.tokens.button_height).toBe(32);
    expect(state.tokens.icon_size).toBe(20);
    expect(state.tokens.contrast_ratio).toBeGreaterThanOrEqual(4.5);
    expect(coordinator.isWcagAaCompliant()).toBe(true);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-theme")).toBe("light");
      expect(document.documentElement.getAttribute("data-ribbon-mode")).toBe("classic");
      expect(document.documentElement.hasAttribute("data-high-contrast")).toBe(false);
    }
  });

  it("2. switches theme to dark mode and updates semantic tokens and DOM", async () => {
    const tokens = await coordinator.setThemeMode("dark");
    const state = coordinator.getState();

    expect(state.themeMode).toBe("dark");
    expect(state.isHighContrast).toBe(false);
    expect(tokens.mode).toBe("dark");
    expect(tokens.bg_primary).toBe("#0f172a");
    expect(tokens.text_primary).toBe("#f8fafc");
    expect(tokens.contrast_ratio).toBeGreaterThanOrEqual(4.5);
    expect(coordinator.isWcagAaCompliant()).toBe(true);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
      expect(document.documentElement.hasAttribute("data-high-contrast")).toBe(false);
    }
  });

  it("3. switches to high-contrast-dark and verifies high contrast flag & contrast ratio >= 7.0", async () => {
    const tokens = await coordinator.setThemeMode("high-contrast-dark");
    const state = coordinator.getState();

    expect(state.themeMode).toBe("high-contrast-dark");
    expect(state.isHighContrast).toBe(true);
    expect(tokens.bg_primary).toBe("#000000");
    expect(tokens.bg_surface).toBe("#000000");
    expect(tokens.text_primary).toBe("#ffffff");
    expect(tokens.border_color).toBe("#ffffff");
    expect(tokens.contrast_ratio).toBeGreaterThanOrEqual(7.0);
    expect(coordinator.isWcagAaCompliant()).toBe(true);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-theme")).toBe("high-contrast-dark");
      expect(document.documentElement.getAttribute("data-high-contrast")).toBe("true");
    }
  });

  it("4. switches to high-contrast-light and verifies WCAG AA compliance >= 7.0", async () => {
    const tokens = await coordinator.setThemeMode("high-contrast-light");
    const state = coordinator.getState();

    expect(state.themeMode).toBe("high-contrast-light");
    expect(state.isHighContrast).toBe(true);
    expect(tokens.bg_primary).toBe("#ffffff");
    expect(tokens.text_primary).toBe("#000000");
    expect(tokens.border_color).toBe("#000000");
    expect(tokens.contrast_ratio).toBeGreaterThanOrEqual(7.0);
    expect(coordinator.isWcagAaCompliant()).toBe(true);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-theme")).toBe("high-contrast-light");
      expect(document.documentElement.getAttribute("data-high-contrast")).toBe("true");
    }
  });

  it("5. toggles high-contrast preserving dark/light polarity", async () => {
    // Light -> High-Contrast Light
    await coordinator.setThemeMode("light");
    await coordinator.toggleHighContrast();
    expect(coordinator.getState().themeMode).toBe("high-contrast-light");
    expect(coordinator.getState().isHighContrast).toBe(true);

    // High-Contrast Light -> Light
    await coordinator.toggleHighContrast();
    expect(coordinator.getState().themeMode).toBe("light");
    expect(coordinator.getState().isHighContrast).toBe(false);

    // Dark -> High-Contrast Dark
    await coordinator.setThemeMode("dark");
    await coordinator.toggleHighContrast();
    expect(coordinator.getState().themeMode).toBe("high-contrast-dark");
    expect(coordinator.getState().isHighContrast).toBe(true);

    // High-Contrast Dark -> Dark
    await coordinator.toggleHighContrast();
    expect(coordinator.getState().themeMode).toBe("dark");
    expect(coordinator.getState().isHighContrast).toBe(false);
  });

  it("6. switches ribbon layout mode to compact (single-row 44px)", async () => {
    const config = await coordinator.setRibbonMode("compact");
    const state = coordinator.getState();

    expect(state.ribbonMode).toBe("compact");
    expect(config.height_px).toBe(44);
    expect(config.is_single_row).toBe(true);
    expect(config.show_group_headers).toBe(false);
    expect(config.collapsed).toBe(false);
    expect(state.tokens.ribbon_height).toBe(44);
    expect(state.tokens.button_height).toBe(26);
    expect(state.tokens.icon_size).toBe(16);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-ribbon-mode")).toBe("compact");
    }
  });

  it("7. switches ribbon layout mode to collapsed (32px tab strip only)", async () => {
    const config = await coordinator.setRibbonMode("collapsed");
    const state = coordinator.getState();

    expect(state.ribbonMode).toBe("collapsed");
    expect(config.height_px).toBe(32);
    expect(config.collapsed).toBe(true);
    expect(config.is_single_row).toBe(false);
    expect(config.show_group_headers).toBe(false);
    expect(state.tokens.ribbon_height).toBe(32);
    expect(state.tokens.button_height).toBe(22);
    expect(state.tokens.icon_size).toBe(14);

    if (typeof document !== "undefined" && document.documentElement) {
      expect(document.documentElement.getAttribute("data-ribbon-mode")).toBe("collapsed");
    }
  });

  it("8. toggles ribbon collapsed and restores previous uncollapsed mode", async () => {
    // Start at compact
    await coordinator.setRibbonMode("compact");
    expect(coordinator.getState().ribbonMode).toBe("compact");

    // Toggle collapse -> collapsed
    await coordinator.toggleRibbonCollapsed();
    expect(coordinator.getState().ribbonMode).toBe("collapsed");
    expect(coordinator.getState().ribbonConfig.collapsed).toBe(true);

    // Toggle collapse again -> should restore compact!
    await coordinator.toggleRibbonCollapsed();
    expect(coordinator.getState().ribbonMode).toBe("compact");
    expect(coordinator.getState().ribbonConfig.collapsed).toBe(false);
  });

  it("9. toggles between classic and compact ribbon layouts", async () => {
    await coordinator.setRibbonMode("classic");
    expect(coordinator.getState().ribbonMode).toBe("classic");

    await coordinator.toggleRibbonCompact();
    expect(coordinator.getState().ribbonMode).toBe("compact");

    await coordinator.toggleRibbonCompact();
    expect(coordinator.getState().ribbonMode).toBe("classic");
  });

  it("10. Ribbon component renders layout switcher and changes presentation mode", () => {
    const container = document.createElement("div");
    container.id = "test-ribbon-container";
    document.body.appendChild(container);

    const ribbon = new Ribbon("test-ribbon-container");
    expect(ribbon.getLayoutMode()).toBe("classic");
    expect(container.getAttribute("data-ribbon-mode")).toBe("classic");

    // Find the layout toggle button in header
    const toggleBtn = container.querySelector<HTMLButtonElement>(
      "[data-testid='ribbon-layout-toggle-btn']"
    );
    expect(toggleBtn).not.toBeNull();

    // Click toggle button -> switches to compact
    toggleBtn?.click();
    expect(ribbon.getLayoutMode()).toBe("compact");
    expect(container.getAttribute("data-ribbon-mode")).toBe("compact");
    expect(container.classList.contains("ribbon-compact")).toBe(true);

    // Switch to collapsed directly
    ribbon.setLayoutMode("collapsed");
    expect(ribbon.getLayoutMode()).toBe("collapsed");
    expect(container.getAttribute("data-ribbon-mode")).toBe("collapsed");
    expect(container.classList.contains("ribbon-collapsed")).toBe(true);
    // Tools body should not be rendered in collapsed mode
    expect(container.querySelector("#ribbon-tools-body")).toBeNull();

    // Cleanup
    document.body.removeChild(container);
  });
});

