/**
 * ThemeAndRibbonCoordinator.ts
 * GEC-v7.0 Stage 52 (Sub-Phase 52.1.1)
 *
 * Presentation Shell Coordinator for:
 * - Semantic design tokens & 100% theme coverage (Light, Dark, High-Contrast Dark, High-Contrast Light)
 * - Ribbon presentation layouts (Classic multi-row, Compact single-row, Collapsed)
 * - WCAG 2.1 AA contrast ratio auditing & compliance validation
 * - Dynamic CSS custom variable injection and DOM attribute synchronization
 */

import {
  IPC,
  type RibbonLayoutConfigDto,
  type RibbonLayoutMode,
  type ThemeMode,
  type ThemeTokensDto,
} from "../ipc";

export {
  type RibbonLayoutConfigDto,
  type RibbonLayoutMode,
  type ThemeMode,
  type ThemeTokensDto,
};

export interface ThemeAndRibbonState {
  themeMode: ThemeMode;
  ribbonMode: RibbonLayoutMode;
  ribbonConfig: RibbonLayoutConfigDto;
  tokens: ThemeTokensDto;
  isHighContrast: boolean;
  isLoading: boolean;
  errorMessage: string | null;
}

export type ThemeAndRibbonListener = (state: Readonly<ThemeAndRibbonState>) => void;

function createDefaultTokens(theme: ThemeMode = "light", ribbonMode: RibbonLayoutMode = "classic"): ThemeTokensDto {
  const isDark = theme === "dark";
  const isHcDark = theme === "high-contrast-dark";
  const isHcLight = theme === "high-contrast-light";

  let bg_primary = "#f8fafc";
  let bg_surface = "#ffffff";
  let bg_shell = "#f1f5f9";
  let bg_canvas_backdrop = "#e2e8f0";
  let bg_panel = "#ffffff";
  let bg_toolbar = "#f8fafc";
  let border_color = "#cbd5e1";
  let border_subtle = "#e2e8f0";
  let text_primary = "#0f172a";
  let text_secondary = "#475569";
  let text_muted = "#64748b";
  let primary_accent = "#0f6cbd";
  let focus_ring = "#0f6cbd";
  let contrast_ratio = 16.5;

  if (isDark) {
    bg_primary = "#0f172a";
    bg_surface = "#1e293b";
    bg_shell = "#0b1120";
    bg_canvas_backdrop = "#020617";
    bg_panel = "#1e293b";
    bg_toolbar = "#1e293b";
    border_color = "#334155";
    border_subtle = "#1e293b";
    text_primary = "#f8fafc";
    text_secondary = "#94a3b8";
    text_muted = "#64748b";
    primary_accent = "#38bdf8";
    focus_ring = "#38bdf8";
    contrast_ratio = 12.3;
  } else if (isHcDark) {
    bg_primary = "#000000";
    bg_surface = "#000000";
    bg_shell = "#000000";
    bg_canvas_backdrop = "#000000";
    bg_panel = "#000000";
    bg_toolbar = "#000000";
    border_color = "#ffffff";
    border_subtle = "#ffffff";
    text_primary = "#ffffff";
    text_secondary = "#ffff00";
    text_muted = "#00ffff";
    primary_accent = "#00ffff";
    focus_ring = "#00ffff";
    contrast_ratio = 21.0;
  } else if (isHcLight) {
    bg_primary = "#ffffff";
    bg_surface = "#ffffff";
    bg_shell = "#ffffff";
    bg_canvas_backdrop = "#ffffff";
    bg_panel = "#ffffff";
    bg_toolbar = "#ffffff";
    border_color = "#000000";
    border_subtle = "#000000";
    text_primary = "#000000";
    text_secondary = "#000080";
    text_muted = "#333333";
    primary_accent = "#0000ee";
    focus_ring = "#0000ee";
    contrast_ratio = 21.0;
  }

  const ribbon_height = ribbonMode === "compact" ? 44 : ribbonMode === "collapsed" ? 32 : 96;
  const button_height = ribbonMode === "compact" ? 26 : ribbonMode === "collapsed" ? 22 : 32;
  const icon_size = ribbonMode === "compact" ? 16 : ribbonMode === "collapsed" ? 14 : 20;

  return {
    mode: theme,
    ribbon_mode: ribbonMode,
    bg_primary,
    bg_surface,
    bg_shell,
    bg_canvas_backdrop,
    bg_panel,
    bg_toolbar,
    border_color,
    border_subtle,
    text_primary,
    text_secondary,
    text_muted,
    primary_accent,
    focus_ring,
    ribbon_height,
    title_bar_height: 38,
    tab_bar_height: 34,
    status_bar_height: 26,
    button_height,
    icon_size,
    contrast_ratio,
  };
}

export class ThemeAndRibbonCoordinator {
  private state: ThemeAndRibbonState;
  private listeners = new Set<ThemeAndRibbonListener>();
  private previousUncollapsedRibbonMode: RibbonLayoutMode = "classic";

  constructor(initialTheme: ThemeMode = "light", initialRibbonMode: RibbonLayoutMode = "classic") {
    const initialTokens = createDefaultTokens(initialTheme, initialRibbonMode);
    const isHc = initialTheme.startsWith("high-contrast");

    this.state = {
      themeMode: initialTheme,
      ribbonMode: initialRibbonMode,
      ribbonConfig: {
        mode: initialRibbonMode,
        collapsed: initialRibbonMode === "collapsed",
        height_px: initialTokens.ribbon_height,
        is_single_row: initialRibbonMode === "compact",
        show_group_headers: initialRibbonMode === "classic",
      },
      tokens: initialTokens,
      isHighContrast: isHc,
      isLoading: false,
      errorMessage: null,
    };

    if (initialRibbonMode !== "collapsed") {
      this.previousUncollapsedRibbonMode = initialRibbonMode;
    }

    this.applyToDom(this.state.tokens);
  }

  /**
   * Returns an immutable snapshot of the current coordinator state.
   */
  public getState(): Readonly<ThemeAndRibbonState> {
    return this.state;
  }

  /**
   * Subscribes a listener to reactive state modifications.
   */
  public subscribe(listener: ThemeAndRibbonListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Notifies all registered subscribers of state updates.
   */
  private notify(): void {
    const snapshot = Object.freeze({ ...this.state });
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("[ThemeAndRibbonCoordinator] Listener threw error:", err);
      }
    }
  }

  /**
   * Synchronizes design tokens and layout attributes directly into the presentation DOM.
   */
  public applyToDom(tokens: ThemeTokensDto): void {
    if (typeof document === "undefined" || !document.documentElement) {
      return;
    }

    const root = document.documentElement;
    root.setAttribute("data-theme", tokens.mode);
    root.setAttribute("data-ribbon-mode", tokens.ribbon_mode);

    const isHc = tokens.mode.startsWith("high-contrast");
    if (isHc) {
      root.setAttribute("data-high-contrast", "true");
    } else {
      root.removeAttribute("data-high-contrast");
    }

    root.style.setProperty("--bg-primary", tokens.bg_primary);
    root.style.setProperty("--bg-surface", tokens.bg_surface);
    root.style.setProperty("--bg-shell", tokens.bg_shell);
    root.style.setProperty("--bg-canvas-backdrop", tokens.bg_canvas_backdrop);
    root.style.setProperty("--bg-panel", tokens.bg_panel);
    root.style.setProperty("--bg-toolbar", tokens.bg_toolbar);
    root.style.setProperty("--border-color", tokens.border_color);
    root.style.setProperty("--border-subtle", tokens.border_subtle);
    root.style.setProperty("--text-primary", tokens.text_primary);
    root.style.setProperty("--text-secondary", tokens.text_secondary);
    root.style.setProperty("--text-muted", tokens.text_muted);
    root.style.setProperty("--primary-accent", tokens.primary_accent);
    root.style.setProperty("--focus-ring", tokens.focus_ring);
    root.style.setProperty("--ribbon-height", `${tokens.ribbon_height}px`);
    root.style.setProperty("--title-bar-height", `${tokens.title_bar_height}px`);
    root.style.setProperty("--tab-bar-height", `${tokens.tab_bar_height}px`);
    root.style.setProperty("--status-bar-height", `${tokens.status_bar_height}px`);
    root.style.setProperty("--button-height", `${tokens.button_height}px`);
    root.style.setProperty("--icon-size", `${tokens.icon_size}px`);
  }

  /**
   * Initializes theme tokens from the backend Tauri IPC engine or fallback.
   */
  public async refreshTokens(): Promise<ThemeTokensDto> {
    this.state = { ...this.state, isLoading: true, errorMessage: null };
    this.notify();

    try {
      const tokens = await IPC.themeGetTokens(this.state.themeMode, this.state.ribbonMode);
      const isHc = tokens.mode.startsWith("high-contrast");

      this.state = {
        ...this.state,
        tokens,
        isHighContrast: isHc,
        isLoading: false,
        errorMessage: null,
      };

      this.applyToDom(tokens);
      this.notify();
      return tokens;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, isLoading: false, errorMessage: message };
      this.notify();
      throw err;
    }
  }

  /**
   * Changes the active theme mode (Light, Dark, High-Contrast Dark, High-Contrast Light).
   */
  public async setThemeMode(mode: ThemeMode): Promise<ThemeTokensDto> {
    this.state = { ...this.state, isLoading: true, errorMessage: null };
    this.notify();

    try {
      const tokens = await IPC.themeSetMode(mode);
      const isHc = mode.startsWith("high-contrast");

      this.state = {
        ...this.state,
        themeMode: mode,
        tokens,
        isHighContrast: isHc,
        isLoading: false,
        errorMessage: null,
      };

      this.applyToDom(tokens);
      this.notify();
      return tokens;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, isLoading: false, errorMessage: message };
      this.notify();
      throw err;
    }
  }

  /**
   * Sets the ribbon layout mode (Classic, Compact, Collapsed).
   */
  public async setRibbonMode(mode: RibbonLayoutMode): Promise<RibbonLayoutConfigDto> {
    this.state = { ...this.state, isLoading: true, errorMessage: null };
    this.notify();

    try {
      const ribbonConfig = await IPC.ribbonSetLayoutMode(mode);
      const tokens = await IPC.themeGetTokens(this.state.themeMode, mode);

      if (mode !== "collapsed") {
        this.previousUncollapsedRibbonMode = mode;
      }

      this.state = {
        ...this.state,
        ribbonMode: mode,
        ribbonConfig,
        tokens,
        isLoading: false,
        errorMessage: null,
      };

      this.applyToDom(tokens);
      this.notify();
      return ribbonConfig;
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      this.state = { ...this.state, isLoading: false, errorMessage: message };
      this.notify();
      throw err;
    }
  }

  /**
   * Toggles the ribbon between collapsed and previous uncollapsed mode.
   */
  public async toggleRibbonCollapsed(): Promise<RibbonLayoutConfigDto> {
    const isCurrentlyCollapsed = this.state.ribbonMode === "collapsed" || this.state.ribbonConfig.collapsed;
    const targetMode = isCurrentlyCollapsed ? this.previousUncollapsedRibbonMode : "collapsed";
    return this.setRibbonMode(targetMode);
  }

  /**
   * Toggles between Classic (multi-row) and Compact (single-row) ribbon presentation.
   */
  public async toggleRibbonCompact(): Promise<RibbonLayoutConfigDto> {
    const targetMode = this.state.ribbonMode === "compact" ? "classic" : "compact";
    return this.setRibbonMode(targetMode);
  }

  /**
   * Toggles high-contrast mode, preserving dark/light polarity.
   */
  public async toggleHighContrast(enabled?: boolean): Promise<ThemeTokensDto> {
    const currentMode = this.state.themeMode;
    const isCurrentlyHc = currentMode.startsWith("high-contrast");
    const shouldEnable = enabled !== undefined ? enabled : !isCurrentlyHc;

    let targetMode: ThemeMode;
    if (shouldEnable) {
      targetMode = currentMode === "dark" || currentMode === "high-contrast-dark"
        ? "high-contrast-dark"
        : "high-contrast-light";
    } else {
      targetMode = currentMode === "dark" || currentMode === "high-contrast-dark"
        ? "dark"
        : "light";
    }

    return this.setThemeMode(targetMode);
  }

  /**
   * Returns whether the active theme meets WCAG 2.1 AA contrast ratio requirements.
   * Standard WCAG 2.1 AA requires >= 4.5:1. High-contrast mode requires >= 7.0:1.
   */
  public isWcagAaCompliant(): boolean {
    const ratio = this.state.tokens.contrast_ratio;
    if (this.state.isHighContrast) {
      return ratio >= 7.0;
    }
    return ratio >= 4.5;
  }

  /**
   * Returns the current contrast ratio.
   */
  public getContrastRatio(): number {
    return this.state.tokens.contrast_ratio;
  }
}

export const themeAndRibbonCoordinator = new ThemeAndRibbonCoordinator();

