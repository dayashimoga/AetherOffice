/**
 * Interactive AetherCopilot Assistant UI Sidebar Component.
 * Supports private on-device conversation threads, contextual prompt chips,
 * clickable grounded citation links, and diff application triggers.
 */

export interface CopilotCitation {
  marker: string; // e.g. "[1]"
  snippet: string;
  location: string;
}

export interface CopilotDiff {
  original: string;
  modified: string;
  targetRangeOrElement?: string;
}

export interface CopilotMessage {
  id: string;
  sender: "user" | "assistant";
  text: string;
  timestamp: number;
  citations?: CopilotCitation[];
  diff?: CopilotDiff;
}

export interface CopilotSidebarOptions {
  onSendMessage?: (prompt: string, mode: string) => Promise<void> | void;
  onReviewDiff?: (diff: CopilotDiff) => void;
  onCitationClick?: (citation: CopilotCitation) => void;
  onAcceptInlineGhost?: (text: string) => void;
}

export class CopilotSidebar {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private headerEl: HTMLElement;
  private chipsEl: HTMLElement;
  private messagesEl: HTMLElement;
  private inputEl: HTMLTextAreaElement;
  private sendBtn: HTMLButtonElement;

  private activeMode: "writer" | "calc" | "slides" | "pdf" = "writer";
  private messages: CopilotMessage[] = [];
  private visible = false;
  private options: CopilotSidebarOptions;

  constructor(
    containerOrId?: HTMLElement | string,
    options: CopilotSidebarOptions = {}
  ) {
    this.options = options;

    if (containerOrId) {
      if (typeof containerOrId === "string") {
        let el = document.getElementById(containerOrId);
        if (!el) {
          el = document.createElement("aside");
          el.id = containerOrId;
          const parent = document.getElementById("main-content") || document.body;
          parent.appendChild(el);
        }
        this.container = el;
      } else {
        this.container = containerOrId;
      }
    } else {
      let el = document.getElementById("copilot-sidebar-container");
      if (!el) {
        el = document.createElement("aside");
        el.id = "copilot-sidebar-container";
        const parent = document.getElementById("main-content") || document.body;
        parent.appendChild(el);
      }
      this.container = el;
    }

    this.container.style.display = "none";
    this.container.className = "copilot-sidebar-pane";
    this.container.setAttribute("aria-label", "AetherCopilot Assistant");

    this.rootEl = document.createElement("div");
    this.rootEl.className = "copilot-sidebar-root";
    this.rootEl.style.display = "flex";
    this.rootEl.style.flexDirection = "column";
    this.rootEl.style.height = "100%";
    this.rootEl.style.width = "360px";
    this.rootEl.style.backgroundColor = "#ffffff";
    this.rootEl.style.borderLeft = "1px solid #e2e8f0";
    this.rootEl.style.boxShadow = "-4px 0 16px rgba(0, 0, 0, 0.05)";
    this.rootEl.style.fontFamily = "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

    // 1. Header
    this.headerEl = document.createElement("div");
    this.headerEl.className = "copilot-header";
    this.headerEl.style.display = "flex";
    this.headerEl.style.alignItems = "center";
    this.headerEl.style.justifyContent = "space-between";
    this.headerEl.style.padding = "12px 16px";
    this.headerEl.style.borderBottom = "1px solid #e2e8f0";
    this.headerEl.style.backgroundColor = "#f8fafc";

    const titleBox = document.createElement("div");
    titleBox.style.display = "flex";
    titleBox.style.alignItems = "center";
    titleBox.style.gap = "8px";

    const titleIcon = document.createElement("span");
    titleIcon.textContent = "✨";
    titleIcon.style.fontSize = "18px";

    const titleText = document.createElement("span");
    titleText.textContent = "AetherCopilot";
    titleText.style.fontWeight = "700";
    titleText.style.fontSize = "14px";
    titleText.style.color = "#0f172a";

    const privacyBadge = document.createElement("span");
    privacyBadge.className = "copilot-privacy-badge";
    privacyBadge.textContent = "Local RAG";
    privacyBadge.style.fontSize = "10px";
    privacyBadge.style.fontWeight = "600";
    privacyBadge.style.padding = "2px 6px";
    privacyBadge.style.borderRadius = "9999px";
    privacyBadge.style.backgroundColor = "#dcfce7";
    privacyBadge.style.color = "#166534";

    titleBox.appendChild(titleIcon);
    titleBox.appendChild(titleText);
    titleBox.appendChild(privacyBadge);

    const closeBtn = document.createElement("button");
    closeBtn.className = "copilot-btn-close";
    closeBtn.textContent = "✕";
    closeBtn.style.border = "none";
    closeBtn.style.background = "transparent";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.color = "#64748b";
    closeBtn.style.fontSize = "14px";
    closeBtn.style.padding = "4px";
    closeBtn.title = "Close Copilot Sidebar";
    closeBtn.onclick = () => this.hide();

    this.headerEl.appendChild(titleBox);
    this.headerEl.appendChild(closeBtn);
    this.rootEl.appendChild(this.headerEl);

    // 2. Prompt Chips Bar
    this.chipsEl = document.createElement("div");
    this.chipsEl.className = "copilot-chips-container";
    this.chipsEl.style.display = "flex";
    this.chipsEl.style.flexWrap = "wrap";
    this.chipsEl.style.gap = "6px";
    this.chipsEl.style.padding = "10px 16px";
    this.chipsEl.style.backgroundColor = "#ffffff";
    this.chipsEl.style.borderBottom = "1px solid #f1f5f9";
    this.rootEl.appendChild(this.chipsEl);

    // 3. Conversation Messages Thread
    this.messagesEl = document.createElement("div");
    this.messagesEl.className = "copilot-messages-thread";
    this.messagesEl.style.flex = "1";
    this.messagesEl.style.overflowY = "auto";
    this.messagesEl.style.padding = "16px";
    this.messagesEl.style.display = "flex";
    this.messagesEl.style.flexDirection = "column";
    this.messagesEl.style.gap = "12px";
    this.rootEl.appendChild(this.messagesEl);

    // 4. Input Area
    const footerEl = document.createElement("div");
    footerEl.className = "copilot-footer";
    footerEl.style.padding = "12px 16px";
    footerEl.style.borderTop = "1px solid #e2e8f0";
    footerEl.style.backgroundColor = "#f8fafc";
    footerEl.style.display = "flex";
    footerEl.style.flexDirection = "column";
    footerEl.style.gap = "8px";

    const inputRow = document.createElement("div");
    inputRow.style.display = "flex";
    inputRow.style.alignItems = "flex-end";
    inputRow.style.gap = "8px";

    this.inputEl = document.createElement("textarea");
    this.inputEl.className = "copilot-input-textarea";
    this.inputEl.placeholder = "Ask Copilot or choose a prompt chip...";
    this.inputEl.rows = 2;
    this.inputEl.style.flex = "1";
    this.inputEl.style.resize = "none";
    this.inputEl.style.padding = "8px 10px";
    this.inputEl.style.borderRadius = "6px";
    this.inputEl.style.border = "1px solid #cbd5e1";
    this.inputEl.style.fontSize = "13px";
    this.inputEl.style.outline = "none";
    this.inputEl.style.fontFamily = "inherit";

    this.inputEl.onkeydown = (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        this.submitPrompt();
      }
    };

    this.sendBtn = document.createElement("button");
    this.sendBtn.className = "copilot-btn-send";
    this.sendBtn.textContent = "↑";
    this.sendBtn.style.padding = "8px 12px";
    this.sendBtn.style.borderRadius = "6px";
    this.sendBtn.style.border = "none";
    this.sendBtn.style.backgroundColor = "#2563eb";
    this.sendBtn.style.color = "#ffffff";
    this.sendBtn.style.fontWeight = "700";
    this.sendBtn.style.cursor = "pointer";
    this.sendBtn.style.fontSize = "16px";
    this.sendBtn.onclick = () => this.submitPrompt();

    inputRow.appendChild(this.inputEl);
    inputRow.appendChild(this.sendBtn);
    footerEl.appendChild(inputRow);
    this.rootEl.appendChild(footerEl);

    this.container.appendChild(this.rootEl);

    this.renderChips();
    this.addWelcomeMessage();
  }

  public setMode(mode: "writer" | "calc" | "slides" | "pdf"): void {
    this.activeMode = mode;
    this.renderChips();
  }

  public getMode(): string {
    return this.activeMode;
  }

  public show(): void {
    this.visible = true;
    this.container.style.display = "block";
    this.container.classList.add("active");
    setTimeout(() => this.inputEl.focus(), 50);
  }

  public hide(): void {
    this.visible = false;
    this.container.style.display = "none";
    this.container.classList.remove("active");
  }

  public toggle(): void {
    if (this.visible) {
      this.hide();
    } else {
      this.show();
    }
  }

  public isOpen(): boolean {
    return this.visible;
  }

  public renderChips(): void {
    this.chipsEl.innerHTML = "";

    const chipsByMode: Record<string, string[]> = {
      writer: [
        "Summarize Section",
        "Rewrite for Conciseness",
        "Fix Tone & Grammar",
        "Generate Outline",
      ],
      calc: [
        "Explain Formula",
        "Synthesize Formula",
        "Detect Data Anomalies",
        "Summarize Key Trends",
      ],
      slides: [
        "Generate Slide from Outline",
        "Suggest Layout",
        "Create Speaker Notes",
        "Highlight Key Takeaways",
      ],
      pdf: [
        "Summarize Document",
        "Extract Key Tables",
        "Verify Signatures",
      ],
    };

    const currentChips = chipsByMode[this.activeMode] || chipsByMode.writer;

    for (const chipText of currentChips) {
      const chip = document.createElement("button");
      chip.className = "copilot-prompt-chip";
      chip.textContent = chipText;
      chip.style.fontSize = "11px";
      chip.style.fontWeight = "500";
      chip.style.padding = "4px 8px";
      chip.style.borderRadius = "9999px";
      chip.style.border = "1px solid #cbd5e1";
      chip.style.backgroundColor = "#f1f5f9";
      chip.style.color = "#334155";
      chip.style.cursor = "pointer";
      chip.style.transition = "all 0.15s ease";

      chip.onmouseover = () => {
        chip.style.backgroundColor = "#e2e8f0";
        chip.style.borderColor = "#94a3b8";
      };
      chip.onmouseout = () => {
        chip.style.backgroundColor = "#f1f5f9";
        chip.style.borderColor = "#cbd5e1";
      };

      chip.onclick = () => {
        this.inputEl.value = chipText;
        this.submitPrompt();
      };

      this.chipsEl.appendChild(chip);
    }
  }

  private addWelcomeMessage(): void {
    this.appendMessage(
      "assistant",
      "Hello! I am your private, on-device AetherCopilot assistant. All data analysis and neural citations remain completely local with zero cloud telemetry. How may I assist your document today?"
    );
  }

  public appendMessage(
    sender: "user" | "assistant",
    text: string,
    options?: { citations?: CopilotCitation[]; diff?: CopilotDiff }
  ): CopilotMessage {
    const msg: CopilotMessage = {
      id: "msg_" + Date.now() + "_" + Math.floor(Math.random() * 1000),
      sender,
      text,
      timestamp: Date.now(),
      citations: options?.citations,
      diff: options?.diff,
    };

    this.messages.push(msg);
    this.renderMessageBubble(msg);
    return msg;
  }

  public getMessages(): CopilotMessage[] {
    return [...this.messages];
  }

  public clearMessages(): void {
    this.messages = [];
    this.messagesEl.innerHTML = "";
    this.addWelcomeMessage();
  }

  private renderMessageBubble(msg: CopilotMessage): void {
    const bubbleWrapper = document.createElement("div");
    bubbleWrapper.className = `copilot-bubble-wrapper ${msg.sender}`;
    bubbleWrapper.style.display = "flex";
    bubbleWrapper.style.flexDirection = "column";
    bubbleWrapper.style.alignItems = msg.sender === "user" ? "flex-end" : "flex-start";
    bubbleWrapper.style.width = "100%";

    const bubble = document.createElement("div");
    bubble.className = `copilot-message-bubble ${msg.sender}`;
    bubble.style.maxWidth = "85%";
    bubble.style.padding = "10px 14px";
    bubble.style.borderRadius = "12px";
    bubble.style.fontSize = "13px";
    bubble.style.lineHeight = "1.45";

    if (msg.sender === "user") {
      bubble.style.backgroundColor = "#2563eb";
      bubble.style.color = "#ffffff";
      bubble.style.borderBottomRightRadius = "2px";
      bubble.textContent = msg.text;
    } else {
      bubble.style.backgroundColor = "#f1f5f9";
      bubble.style.color = "#0f172a";
      bubble.style.borderBottomLeftRadius = "2px";
      bubble.style.border = "1px solid #e2e8f0";

      // Render text and parse citations [1], [2] into interactive pills
      bubble.innerHTML = this.formatAssistantHtml(msg.text, msg.citations);

      // Attach citation click handlers
      const citationPills = bubble.querySelectorAll(".copilot-citation-link");
      citationPills.forEach((pill) => {
        const marker = pill.getAttribute("data-marker");
        pill.addEventListener("click", () => {
          if (marker && msg.citations) {
            const found = msg.citations.find((c) => c.marker === marker);
            if (found) {
              if (this.options.onCitationClick) {
                this.options.onCitationClick(found);
              }
            }
          }
        });
      });

      // If this message provides proposed text/formula diff, render "Review Diff" button
      if (msg.diff) {
        const diffBtn = document.createElement("button");
        diffBtn.className = "copilot-btn-review-diff";
        diffBtn.textContent = "⚡ Review & Apply Diff";
        diffBtn.style.marginTop = "8px";
        diffBtn.style.display = "inline-flex";
        diffBtn.style.alignItems = "center";
        diffBtn.style.gap = "4px";
        diffBtn.style.fontSize = "11px";
        diffBtn.style.fontWeight = "600";
        diffBtn.style.padding = "5px 10px";
        diffBtn.style.borderRadius = "6px";
        diffBtn.style.border = "1px solid #3b82f6";
        diffBtn.style.backgroundColor = "#eff6ff";
        diffBtn.style.color = "#1d4ed8";
        diffBtn.style.cursor = "pointer";

        diffBtn.onclick = () => {
          if (this.options.onReviewDiff && msg.diff) {
            this.options.onReviewDiff(msg.diff);
          }
        };

        bubble.appendChild(diffBtn);
      }
    }

    bubbleWrapper.appendChild(bubble);
    this.messagesEl.appendChild(bubbleWrapper);
    this.messagesEl.scrollTop = this.messagesEl.scrollHeight;
  }

  private formatAssistantHtml(text: string, citations?: CopilotCitation[]): string {
    // Replace citation markers [1], [2] with styled clickable badges
    let html = text.replace(/\[(\d+)\]/g, (match) => {
      const cit = citations?.find((c) => c.marker === match);
      const tooltip = cit ? `title="Source: ${cit.location} - '${cit.snippet.substring(0, 60)}...'"` : "";
      return `<button class="copilot-citation-link" data-marker="${match}" ${tooltip} style="display:inline-block; margin:0 2px; padding:1px 5px; font-size:10px; font-weight:700; color:#1d4ed8; background:#dbeafe; border-radius:4px; border:none; cursor:pointer;">${match}</button>`;
    });

    // Replace linebreaks with <br>
    html = html.replace(/\n/g, "<br>");
    return html;
  }

  public submitPrompt(): void {
    const raw = this.inputEl.value.trim();
    if (!raw) return;

    this.appendMessage("user", raw);
    this.inputEl.value = "";

    // Simulated local AI assistant response if no external onSendMessage is provided
    if (this.options.onSendMessage) {
      this.options.onSendMessage(raw, this.activeMode);
    } else {
      setTimeout(() => {
        this.generateDefaultResponse(raw);
      }, 150);
    }
  }

  private generateDefaultResponse(prompt: string): void {
    const lower = prompt.toLowerCase();

    if (lower.includes("summarize") || lower.includes("summary")) {
      this.appendMessage(
        "assistant",
        "Executive Summary: In Q3, enterprise revenue increased by 24% year-over-year [1]. Strategic operating margins expanded to 31.5% driven by native cloud optimization [2].",
        {
          citations: [
            {
              marker: "[1]",
              snippet: "Enterprise subscription revenue increased by 24% year over year.",
              location: "Section 1 > Heading 2 > Para 3",
            },
            {
              marker: "[2]",
              snippet: "Operating margins improved to 31.5% driven by cloud adoption.",
              location: "Section 1 > Heading 2 > Para 4",
            },
          ],
        }
      );
    } else if (lower.includes("concise") || lower.includes("rewrite")) {
      this.appendMessage(
        "assistant",
        "Here is a 40% more concise rewrite preserving all key quantitative terms [1].",
        {
          citations: [
            {
              marker: "[1]",
              snippet: "Selected paragraph text rewrite",
              location: "Active Selection",
            },
          ],
          diff: {
            original: "In order to facilitate the completion of the project, we must utilize all available resources.",
            modified: "To complete the project, we must utilize all available resources.",
          },
        }
      );
    } else if (lower.includes("formula")) {
      this.appendMessage(
        "assistant",
        "Proposed Calc formula: `=XLOOKUP(A2, Products!A:A, Products!C:C, 0)` [1]. This matches SKU codes and returns unit prices without column index fragility.",
        {
          citations: [
            {
              marker: "[1]",
              snippet: "Product catalog table lookup reference",
              location: "Sheet: Products > Range: A2:C50",
            },
          ],
          diff: {
            original: "=VLOOKUP(A2, Products!A:C, 3, FALSE)",
            modified: "=XLOOKUP(A2, Products!A:A, Products!C:C, 0)",
            targetRangeOrElement: "D2",
          },
        }
      );
    } else {
      this.appendMessage(
        "assistant",
        `Grounded insights for '${prompt}' generated using local semantic document vectors [1].`,
        {
          citations: [
            {
              marker: "[1]",
              snippet: "Document semantic chunk",
              location: "Body > Para 1",
            },
          ],
        }
      );
    }
  }
}

