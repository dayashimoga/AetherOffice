import { pluginHost, PluginHost, PluginManifest } from "../plugins/pluginHost";

export class PluginManagerModal {
  private overlay: HTMLElement;
  private currentTab: "installed" | "marketplace" = "installed";
  private searchQuery = "";
  private visible = false;

  constructor(
    parentContainerId = "app",
    private host: PluginHost = pluginHost
  ) {
    let parent = document.getElementById(parentContainerId);
    if (!parent) {
      parent = document.body;
    }

    const overlay = document.createElement("div");
    overlay.id = "plugin-manager-modal";
    overlay.className = "plugin-manager-modal";
    overlay.style.display = "none";
    overlay.style.position = "fixed";
    overlay.style.top = "0";
    overlay.style.left = "0";
    overlay.style.width = "100vw";
    overlay.style.height = "100vh";
    overlay.style.backgroundColor = "rgba(0, 0, 0, 0.45)";
    overlay.style.zIndex = "2500";
    overlay.style.alignItems = "center";
    overlay.style.justifyContent = "center";
    overlay.style.fontFamily = "Segoe UI, -apple-system, sans-serif";

    parent.appendChild(overlay);
    this.overlay = overlay;

    window.addEventListener("keydown", (e) => {
      if (this.visible && e.key === "Escape") {
        this.close();
      }
    });

    this.host.onLifecycle(() => {
      if (this.visible) {
        this.render();
      }
    });
  }

  public open(tab: "installed" | "marketplace" = "installed"): void {
    this.visible = true;
    this.currentTab = tab;
    this.searchQuery = "";
    this.overlay.style.display = "flex";
    this.render();
  }

  public close(): void {
    this.visible = false;
    this.overlay.style.display = "none";
  }

  public isOpen(): boolean {
    return this.visible;
  }

  public render(): void {
    const installedPlugins = this.host.getAllPlugins();
    const marketplacePlugins = this.host.getMarketplaceCatalog();

    this.overlay.innerHTML = `
      <div class="plugin-manager-dialog" style="background: #ffffff; border-radius: 8px; width: 680px; max-height: 85vh; box-shadow: 0 20px 48px rgba(0,0,0,0.28); display: flex; flex-direction: column; overflow: hidden; border: 1px solid #d1d5db;">
        <!-- Header -->
        <div style="display: flex; justify-content: space-between; align-items: center; padding: 14px 20px; border-bottom: 1px solid #e5e7eb; background: #f9fafb;">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 18px;">🧩</span>
            <span style="font-weight: 700; font-size: 16px; color: #111827;">Add-ins & Extensions</span>
          </div>
          <button class="btn-close-plugin-manager" data-testid="plugin-manager-close" style="background: none; border: none; font-size: 18px; cursor: pointer; color: #6b7280;">✕</button>
        </div>

        <!-- Controls: Tabs, Search & Install from File -->
        <div style="padding: 12px 20px; border-bottom: 1px solid #f3f4f6; display: flex; justify-content: space-between; align-items: center; gap: 12px; background: #ffffff;">
          <div style="display: flex; gap: 6px;">
            <button class="tab-btn ${this.currentTab === "installed" ? "active" : ""}" data-testid="tab-installed" style="padding: 6px 14px; border: 1px solid ${this.currentTab === "installed" ? "#2b579a" : "#d1d5db"}; background: ${this.currentTab === "installed" ? "#2b579a" : "#ffffff"}; color: ${this.currentTab === "installed" ? "#ffffff" : "#374151"}; border-radius: 4px; font-size: 13px; font-weight: 600; cursor: pointer;">
              Installed (${installedPlugins.length})
            </button>
            <button class="tab-btn ${this.currentTab === "marketplace" ? "active" : ""}" data-testid="tab-marketplace" style="padding: 6px 14px; border: 1px solid ${this.currentTab === "marketplace" ? "#2b579a" : "#d1d5db"}; background: ${this.currentTab === "marketplace" ? "#2b579a" : "#ffffff"}; color: ${this.currentTab === "marketplace" ? "#ffffff" : "#374151"}; border-radius: 4px; font-size: 13px; font-weight: 600; cursor: pointer;">
              Marketplace
            </button>
          </div>

          <div style="display: flex; gap: 8px; align-items: center;">
            <input type="text" class="plugin-search-input" data-testid="plugin-search-input" placeholder="Search extensions..." value="${this.searchQuery}" style="padding: 6px 10px; border: 1px solid #d1d5db; border-radius: 4px; font-size: 13px; width: 180px;" />
            <button class="btn-install-file" data-testid="plugin-install-file-btn" style="padding: 6px 12px; border: 1px solid #d1d5db; border-radius: 4px; background: #f3f4f6; font-size: 12px; font-weight: 600; cursor: pointer; color: #1f2937;">
              Install from File
            </button>
            <input type="file" class="plugin-file-input" data-testid="plugin-file-input" style="display: none;" accept=".json" />
          </div>
        </div>

        <!-- Body Content -->
        <div class="plugin-list-scroll" style="flex: 1; overflow-y: auto; padding: 16px 20px; display: flex; flex-direction: column; gap: 12px; min-height: 280px; max-height: 55vh;">
          ${
            this.currentTab === "installed"
              ? this.renderInstalledList(installedPlugins)
              : this.renderMarketplaceList(marketplacePlugins)
          }
        </div>
      </div>
    `;

    this.attachEventListeners();
  }

  private renderInstalledList(plugins: PluginManifest[]): string {
    const q = this.searchQuery.toLowerCase();
    const filtered = plugins.filter(
      (p) =>
        !q ||
        p.name.toLowerCase().includes(q) ||
        (p.description && p.description.toLowerCase().includes(q)) ||
        (p.author && p.author.toLowerCase().includes(q))
    );

    if (filtered.length === 0) {
      return `
        <div style="text-align: center; color: #6b7280; padding: 32px 16px; font-size: 14px;">
          No extensions found. Browse the Marketplace to discover and install new add-ins.
        </div>
      `;
    }

    return filtered
      .map((p) => {
        const isEnabled = p.enabled !== false;
        return `
          <div class="plugin-card" data-testid="plugin-card-${p.id}" style="border: 1px solid #e5e7eb; border-radius: 6px; padding: 14px 16px; display: flex; justify-content: space-between; align-items: flex-start; background: #ffffff;">
            <div style="display: flex; gap: 12px; flex: 1;">
              <span style="font-size: 24px;">${p.icon || "🧩"}</span>
              <div style="display: flex; flex-direction: column; gap: 4px; flex: 1;">
                <div style="display: flex; align-items: baseline; gap: 8px;">
                  <span style="font-weight: 700; font-size: 14px; color: #111827;">${p.name}</span>
                  <span style="font-size: 11px; color: #6b7280;">v${p.version}</span>
                  ${p.author ? `<span style="font-size: 11px; color: #9ca3af;">by ${p.author}</span>` : ""}
                </div>
                <div style="font-size: 12px; color: #4b5563; line-height: 1.4;">${p.description || "No description provided."}</div>
                <div style="display: flex; gap: 6px; margin-top: 4px; flex-wrap: wrap;">
                  ${this.renderCapabilityBadges(p)}
                </div>
              </div>
            </div>

            <div style="display: flex; align-items: center; gap: 10px; margin-left: 12px;">
              <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 12px; font-weight: 600; color: #374151;">
                <input type="checkbox" class="plugin-toggle" data-testid="plugin-toggle-${p.id}" data-plugin-id="${p.id}" ${isEnabled ? "checked" : ""} />
                <span>${isEnabled ? "Enabled" : "Disabled"}</span>
              </label>
              <button class="btn-uninstall-plugin" data-testid="plugin-uninstall-${p.id}" data-plugin-id="${p.id}" style="padding: 4px 8px; border: 1px solid #fee2e2; border-radius: 4px; background: #fef2f2; color: #b91c1c; font-size: 11px; font-weight: 600; cursor: pointer;">
                Uninstall
              </button>
            </div>
          </div>
        `;
      })
      .join("");
  }

  private renderMarketplaceList(plugins: PluginManifest[]): string {
    const q = this.searchQuery.toLowerCase();
    const installedIds = new Set(this.host.getAllPlugins().map((p) => p.id));

    const filtered = plugins.filter(
      (p) =>
        !q ||
        p.name.toLowerCase().includes(q) ||
        (p.description && p.description.toLowerCase().includes(q)) ||
        (p.author && p.author.toLowerCase().includes(q))
    );

    return filtered
      .map((p) => {
        const isInstalled = installedIds.has(p.id);
        return `
          <div class="plugin-card" data-testid="plugin-card-${p.id}" style="border: 1px solid #e5e7eb; border-radius: 6px; padding: 14px 16px; display: flex; justify-content: space-between; align-items: flex-start; background: #ffffff;">
            <div style="display: flex; gap: 12px; flex: 1;">
              <span style="font-size: 24px;">${p.icon || "🧩"}</span>
              <div style="display: flex; flex-direction: column; gap: 4px; flex: 1;">
                <div style="display: flex; align-items: baseline; gap: 8px;">
                  <span style="font-weight: 700; font-size: 14px; color: #111827;">${p.name}</span>
                  <span style="font-size: 11px; color: #6b7280;">v${p.version}</span>
                  ${p.author ? `<span style="font-size: 11px; color: #9ca3af;">by ${p.author}</span>` : ""}
                </div>
                <div style="font-size: 12px; color: #4b5563; line-height: 1.4;">${p.description || "No description provided."}</div>
                <div style="display: flex; gap: 6px; margin-top: 4px; flex-wrap: wrap;">
                  ${this.renderCapabilityBadges(p)}
                </div>
              </div>
            </div>

            <div style="display: flex; align-items: center; margin-left: 12px;">
              ${
                isInstalled
                  ? `<span style="padding: 4px 10px; border-radius: 4px; background: #ecfdf5; color: #047857; font-size: 12px; font-weight: 600;">Installed</span>`
                  : `<button class="btn-install-plugin" data-testid="plugin-install-${p.id}" data-plugin-id="${p.id}" style="padding: 6px 14px; border: 1px solid #107c41; border-radius: 4px; background: #107c41; color: #ffffff; font-size: 12px; font-weight: 600; cursor: pointer;">Install</button>`
              }
            </div>
          </div>
        `;
      })
      .join("");
  }

  private renderCapabilityBadges(p: PluginManifest): string {
    const badges: string[] = [];
    if (p.ribbonContributions?.tabs || p.ribbonContributions?.injectedGroups) {
      badges.push("Ribbon Tab");
    }
    if (p.sidebarContributions && p.sidebarContributions.length > 0) {
      badges.push("Custom Sidebar");
    }
    if (p.udfContributions && p.udfContributions.length > 0) {
      badges.push("Formula UDF");
    }
    if (p.shortcuts && p.shortcuts.length > 0) {
      badges.push("Shortcuts");
    }
    if (badges.length === 0) {
      badges.push("General Extension");
    }

    return badges
      .map(
        (b) =>
          `<span style="background: #eff6ff; color: #1e40af; border: 1px solid #dbeafe; padding: 2px 6px; border-radius: 3px; font-size: 10px; font-weight: 600;">${b}</span>`
      )
      .join("");
  }

  private attachEventListeners(): void {
    // Close button
    const closeBtn = this.overlay.querySelector(".btn-close-plugin-manager");
    closeBtn?.addEventListener("click", () => this.close());

    // Tab buttons
    const tabInstalled = this.overlay.querySelector('[data-testid="tab-installed"]');
    tabInstalled?.addEventListener("click", () => {
      this.currentTab = "installed";
      this.render();
    });

    const tabMarketplace = this.overlay.querySelector('[data-testid="tab-marketplace"]');
    tabMarketplace?.addEventListener("click", () => {
      this.currentTab = "marketplace";
      this.render();
    });

    // Search input
    const searchInput = this.overlay.querySelector<HTMLInputElement>(
      ".plugin-search-input"
    );
    if (searchInput) {
      searchInput.addEventListener("input", (e) => {
        this.searchQuery = (e.target as HTMLInputElement).value;
        const scroll = this.overlay.querySelector(".plugin-list-scroll");
        if (scroll) {
          scroll.innerHTML =
            this.currentTab === "installed"
              ? this.renderInstalledList(this.host.getAllPlugins())
              : this.renderMarketplaceList(this.host.getMarketplaceCatalog());
          this.attachCardEventListeners();
        }
      });
    }

    // Install from file button & input
    const installFileBtn = this.overlay.querySelector(".btn-install-file");
    const fileInput = this.overlay.querySelector<HTMLInputElement>(".plugin-file-input");
    if (installFileBtn && fileInput) {
      installFileBtn.addEventListener("click", () => {
        fileInput.click();
      });
      fileInput.addEventListener("change", () => {
        if (fileInput.files && fileInput.files[0]) {
          const reader = new FileReader();
          reader.onload = (e) => {
            const content = e.target?.result as string;
            if (content) {
              try {
                this.host.installFromFile(content);
                this.render();
              } catch (err) {
                alert(`Failed to install plugin: ${(err as Error).message}`);
              }
            }
          };
          reader.readAsText(fileInput.files[0]);
        }
      });
    }

    this.attachCardEventListeners();
  }

  private attachCardEventListeners(): void {
    // Toggle enable/disable
    this.overlay.querySelectorAll<HTMLInputElement>(".plugin-toggle").forEach((chk) => {
      chk.addEventListener("change", () => {
        const id = chk.getAttribute("data-plugin-id");
        if (id) {
          if (chk.checked) {
            this.host.enablePlugin(id);
          } else {
            this.host.disablePlugin(id);
          }
          this.render();
        }
      });
    });

    // Uninstall
    this.overlay.querySelectorAll<HTMLButtonElement>(".btn-uninstall-plugin").forEach((btn) => {
      btn.addEventListener("click", () => {
        const id = btn.getAttribute("data-plugin-id");
        if (id) {
          this.host.unregisterPlugin(id);
          this.render();
        }
      });
    });

    // Marketplace install
    this.overlay.querySelectorAll<HTMLButtonElement>(".btn-install-plugin").forEach((btn) => {
      btn.addEventListener("click", () => {
        const id = btn.getAttribute("data-plugin-id");
        if (id) {
          this.host.installFromMarketplace(id);
          this.render();
        }
      });
    });
  }
}

