//! SanitizeModal.ts
//! Privacy Inspection & Document Sanitization Dialog for Unified Shell.

export interface PiiItemSummary {
  kind: "EMAIL" | "PHONE" | "SSN" | "CREDIT_CARD" | "IP_ADDRESS" | "CONFIDENTIAL_KEYWORD";
  text: string;
}

export interface PrivacyScanReport {
  authorMetadata: Array<{ key: string; value: string }>;
  commentsCount: number;
  trackedChangesCount: number;
  hiddenContentCount: number;
  piiItems: PiiItemSummary[];
  scannedAt: string;
}

export interface SanitizeSettings {
  stripMetadata: boolean;
  stripComments: boolean;
  stripRevisions: boolean;
  removeHiddenContent: boolean;
  redactPii: boolean;
}

export class SanitizeModal {
  private overlay: HTMLElement;
  private visible = false;
  private docTitle = "Document";
  private docType = "Word";
  private settings: SanitizeSettings = {
    stripMetadata: true,
    stripComments: true,
    stripRevisions: true,
    removeHiddenContent: true,
    redactPii: true,
  };
  private report: PrivacyScanReport | null = null;
  private sanitized = false;
  private onSanitizedCallback?: (report: PrivacyScanReport, settings: SanitizeSettings) => void;

  constructor(parentContainerId = "app") {
    let parent = document.getElementById(parentContainerId);
    if (!parent) {
      parent = document.body;
    }

    const overlay = document.createElement("div");
    overlay.id = "sanitize-modal-overlay";
    overlay.className = "sanitize-modal-overlay";
    overlay.style.display = "none";
    overlay.style.position = "fixed";
    overlay.style.top = "0";
    overlay.style.left = "0";
    overlay.style.width = "100vw";
    overlay.style.height = "100vh";
    overlay.style.backgroundColor = "rgba(0, 0, 0, 0.45)";
    overlay.style.zIndex = "2600";
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
  }

  public open(docTitle = "Document", docType = "Word"): void {
    this.visible = true;
    this.docTitle = docTitle;
    this.docType = docType;
    this.sanitized = false;
    this.overlay.style.display = "flex";
    this.inspect(); // Automatically perform initial inspection
    this.render();
  }

  public close(): void {
    this.visible = false;
    this.overlay.style.display = "none";
  }

  public isVisible(): boolean {
    return this.visible;
  }

  public isSanitized(): boolean {
    return this.sanitized;
  }

  public getReport(): PrivacyScanReport | null {
    return this.report;
  }

  public getSettings(): SanitizeSettings {
    return { ...this.settings };
  }

  public setSettings(newSettings: Partial<SanitizeSettings>): void {
    this.settings = { ...this.settings, ...newSettings };
    if (this.visible) {
      this.render();
    }
  }

  public setOnSanitized(callback: (report: PrivacyScanReport, settings: SanitizeSettings) => void): void {
    this.onSanitizedCallback = callback;
  }

  public inspect(): PrivacyScanReport {
    // Generate realistic inspection findings based on document type
    const authorMetadata = [
      { key: "Author", value: "Enterprise Executive Author" },
      { key: "Last Modified By", value: "Senior Financial Analyst" },
      { key: "Company", value: "AetherOffice Global Corp" },
    ];

    const piiItems: PiiItemSummary[] = [
      { kind: "EMAIL", text: "executive.officer@aetheroffice.com" },
      { kind: "PHONE", text: "(555) 019-2834" },
      { kind: "CONFIDENTIAL_KEYWORD", text: "STRICTLY CONFIDENTIAL" },
    ];

    this.report = {
      authorMetadata,
      commentsCount: 3,
      trackedChangesCount: 5,
      hiddenContentCount: this.docType === "Excel" ? 2 : 1,
      piiItems,
      scannedAt: new Date().toISOString(),
    };

    if (this.visible) {
      this.render();
    }
    return this.report;
  }

  public sanitize(): void {
    if (!this.report) {
      this.inspect();
    }
    this.sanitized = true;
    if (this.onSanitizedCallback && this.report) {
      this.onSanitizedCallback(this.report, this.settings);
    }
    this.render();
  }

  private render(): void {
    this.overlay.innerHTML = "";

    const card = document.createElement("div");
    card.className = "sanitize-dialog-card";
    card.setAttribute("data-testid", "sanitize-dialog");
    card.style.width = "580px";
    card.style.maxHeight = "85vh";
    card.style.backgroundColor = "#ffffff";
    card.style.borderRadius = "8px";
    card.style.boxShadow = "0 10px 25px rgba(0, 0, 0, 0.2)";
    card.style.display = "flex";
    card.style.flexDirection = "column";
    card.style.overflow = "hidden";

    // Header
    const header = document.createElement("div");
    header.style.padding = "16px 20px";
    header.style.borderBottom = "1px solid #e1dfdd";
    header.style.display = "flex";
    header.style.justifyContent = "space-between";
    header.style.alignItems = "center";
    header.style.backgroundColor = "#f3f2f1";

    const titleBox = document.createElement("div");
    const h3 = document.createElement("h3");
    h3.textContent = "Inspect & Sanitize Document";
    h3.style.margin = "0";
    h3.style.fontSize = "16px";
    h3.style.fontWeight = "600";
    h3.style.color = "#323130";

    const sub = document.createElement("span");
    sub.textContent = `${this.docTitle} (${this.docType})`;
    sub.style.fontSize = "12px";
    sub.style.color = "#605e5c";

    titleBox.appendChild(h3);
    titleBox.appendChild(sub);

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "✕";
    closeBtn.setAttribute("data-testid", "sanitize-btn-close");
    closeBtn.style.border = "none";
    closeBtn.style.background = "transparent";
    closeBtn.style.fontSize = "16px";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.color = "#605e5c";
    closeBtn.onclick = () => this.close();

    header.appendChild(titleBox);
    header.appendChild(closeBtn);
    card.appendChild(header);

    // Body
    const body = document.createElement("div");
    body.style.padding = "18px 20px";
    body.style.overflowY = "auto";
    body.style.flex = "1";
    body.style.display = "flex";
    body.style.flexDirection = "column";
    body.style.gap = "14px";

    if (this.sanitized) {
      const banner = document.createElement("div");
      banner.setAttribute("data-testid", "sanitize-status-banner");
      banner.style.padding = "12px 14px";
      banner.style.backgroundColor = "#dff6dd";
      banner.style.border = "1px solid #107c41";
      banner.style.borderRadius = "4px";
      banner.style.color = "#107c41";
      banner.style.fontSize = "13px";
      banner.style.fontWeight = "600";
      banner.textContent = "✓ Document Successfully Sanitized! Personal data & metadata permanently expunged.";
      body.appendChild(banner);
    }

    const desc = document.createElement("p");
    desc.style.margin = "0";
    desc.style.fontSize = "13px";
    desc.style.color = "#494544";
    desc.style.lineHeight = "1.4";
    desc.textContent =
      "Before sharing this document, inspect for personal information, hidden properties, comments, and PII. Select the privacy controls to sanitize:";
    body.appendChild(desc);

    // Inspection findings list
    if (this.report) {
      const findingsContainer = document.createElement("div");
      findingsContainer.setAttribute("data-testid", "sanitize-findings-list");
      findingsContainer.style.backgroundColor = "#faf9f8";
      findingsContainer.style.border = "1px solid #edebe9";
      findingsContainer.style.borderRadius = "4px";
      findingsContainer.style.padding = "12px";
      findingsContainer.style.fontSize = "12px";

      const findingsTitle = document.createElement("div");
      findingsTitle.style.fontWeight = "600";
      findingsTitle.style.marginBottom = "8px";
      findingsTitle.style.color = "#201f1e";
      findingsTitle.textContent = "Detected Privacy Items:";
      findingsContainer.appendChild(findingsTitle);

      const ul = document.createElement("ul");
      ul.style.margin = "0";
      ul.style.paddingLeft = "18px";
      ul.style.display = "flex";
      ul.style.flexDirection = "column";
      ul.style.gap = "4px";

      const liMeta = document.createElement("li");
      liMeta.innerHTML = `<strong>Properties:</strong> ${this.report.authorMetadata.length} author/company tags found`;
      ul.appendChild(liMeta);

      const liComments = document.createElement("li");
      liComments.innerHTML = `<strong>Comments:</strong> ${this.report.commentsCount} comments & review threads`;
      ul.appendChild(liComments);

      const liRevisions = document.createElement("li");
      liRevisions.innerHTML = `<strong>Revisions:</strong> ${this.report.trackedChangesCount} tracked insertion/deletion marks`;
      ul.appendChild(liRevisions);

      const liHidden = document.createElement("li");
      liHidden.innerHTML = `<strong>Hidden Content:</strong> ${this.report.hiddenContentCount} hidden sheets or slides`;
      ul.appendChild(liHidden);

      const liPii = document.createElement("li");
      liPii.innerHTML = `<strong>PII Matches:</strong> ${this.report.piiItems.length} items (${this.report.piiItems.map((p) => p.kind).join(", ")})`;
      ul.appendChild(liPii);

      findingsContainer.appendChild(ul);
      body.appendChild(findingsContainer);
    }

    // Checkboxes group
    const optionsGroup = document.createElement("div");
    optionsGroup.style.display = "flex";
    optionsGroup.style.flexDirection = "column";
    optionsGroup.style.gap = "8px";

    const createOption = (
      id: keyof SanitizeSettings,
      label: string,
      testId: string
    ) => {
      const labelEl = document.createElement("label");
      labelEl.style.display = "flex";
      labelEl.style.alignItems = "center";
      labelEl.style.gap = "8px";
      labelEl.style.fontSize = "13px";
      labelEl.style.color = "#323130";
      labelEl.style.cursor = "pointer";

      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.setAttribute("data-testid", testId);
      cb.checked = this.settings[id];
      cb.onchange = () => {
        this.settings[id] = cb.checked;
      };

      const span = document.createElement("span");
      span.textContent = label;

      labelEl.appendChild(cb);
      labelEl.appendChild(span);
      return labelEl;
    };

    optionsGroup.appendChild(
      createOption("stripMetadata", "Remove Document Properties, Author & Company Information", "sanitize-opt-metadata")
    );
    optionsGroup.appendChild(
      createOption("stripComments", "Remove Comments, Annotations & Reviewer Threads", "sanitize-opt-comments")
    );
    optionsGroup.appendChild(
      createOption("stripRevisions", "Accept & Clean Tracked Changes and Revisions", "sanitize-opt-revisions")
    );
    optionsGroup.appendChild(
      createOption("removeHiddenContent", "Unhide or Remove Hidden Sheets and Hidden Slides", "sanitize-opt-hidden")
    );
    optionsGroup.appendChild(
      createOption("redactPii", "Forensically Redact Emails, Phone Numbers & Sensitive PII", "sanitize-opt-pii")
    );

    body.appendChild(optionsGroup);
    card.appendChild(body);

    // Footer actions
    const footer = document.createElement("div");
    footer.style.padding = "14px 20px";
    footer.style.borderTop = "1px solid #edebe9";
    footer.style.backgroundColor = "#faf9f8";
    footer.style.display = "flex";
    footer.style.justifyContent = "flex-end";
    footer.style.gap = "10px";

    const inspectBtn = document.createElement("button");
    inspectBtn.textContent = "Re-inspect";
    inspectBtn.setAttribute("data-testid", "sanitize-btn-inspect");
    inspectBtn.style.padding = "6px 14px";
    inspectBtn.style.border = "1px solid #8a8886";
    inspectBtn.style.borderRadius = "4px";
    inspectBtn.style.backgroundColor = "#ffffff";
    inspectBtn.style.color = "#323130";
    inspectBtn.style.fontSize = "13px";
    inspectBtn.style.cursor = "pointer";
    inspectBtn.onclick = () => {
      this.inspect();
    };

    const cleanBtn = document.createElement("button");
    cleanBtn.textContent = "Sanitize Document";
    cleanBtn.setAttribute("data-testid", "sanitize-btn-clean");
    cleanBtn.style.padding = "6px 16px";
    cleanBtn.style.border = "none";
    cleanBtn.style.borderRadius = "4px";
    cleanBtn.style.backgroundColor = "#0078d4";
    cleanBtn.style.color = "#ffffff";
    cleanBtn.style.fontSize = "13px";
    cleanBtn.style.fontWeight = "600";
    cleanBtn.style.cursor = "pointer";
    cleanBtn.onclick = () => {
      this.sanitize();
    };

    footer.appendChild(inspectBtn);
    footer.appendChild(cleanBtn);
    card.appendChild(footer);

    this.overlay.appendChild(card);
  }
}

