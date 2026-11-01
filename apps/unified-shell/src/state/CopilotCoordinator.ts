/**
 * CopilotCoordinator.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.2.2)
 *
 * Presentation Shell Coordinator for:
 * - Multi-Document Grounded AI Copilot
 * - Conversation threading with grounded citation tracking
 * - Provider configuration switching (Local Ollama, WebGPU, Cloud API, Offline Template)
 * - Multi-document corpus indexing and document session lifecycle
 * - Active canvas selection context routing and interactive diff proposals
 */

import {
  IPC,
  type CopilotChatMessageDto,
  type CopilotChatRequestDto,
  type CopilotChatResponseDto,
  type CopilotProviderKind,
  type GroundedCitationDto,
  type GroundedDiffDto,
  type ProviderConfigDto,
  type ProviderStatusDto,
} from "../ipc";

export {
  type CopilotChatMessageDto,
  type CopilotChatRequestDto,
  type CopilotChatResponseDto,
  type CopilotProviderKind,
  type GroundedCitationDto,
  type GroundedDiffDto,
  type ProviderConfigDto,
  type ProviderStatusDto,
};

export interface CopilotState {
  sessionId: string;
  messages: CopilotChatMessageDto[];
  providerStatus: ProviderStatusDto | null;
  indexedDocIds: string[];
  activeSelection: {
    kind?: string;
    selected_text?: string | null;
    selected_range?: string | null;
  } | null;
  targetDocIds: string[] | null;
  pendingDiff: GroundedDiffDto | null;
  isLoading: boolean;
  errorMessage: string | null;
}

export type CopilotListener = (state: Readonly<CopilotState>) => void;

export class CopilotCoordinator {
  private state: CopilotState;
  private listeners = new Set<CopilotListener>();

  constructor(sessionId = "default-session") {
    this.state = {
      sessionId,
      messages: [],
      providerStatus: null,
      indexedDocIds: [],
      activeSelection: null,
      targetDocIds: null,
      pendingDiff: null,
      isLoading: false,
      errorMessage: null,
    };
  }

  /**
   * Returns an immutable snapshot of the active state.
   */
  public getState(): Readonly<CopilotState> {
    return this.state;
  }

  /**
   * Subscribes a listener to reactive state updates.
   */
  public subscribe(listener: CopilotListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const snap = Object.freeze({ ...this.state });
    for (const listener of this.listeners) {
      try {
        listener(snap);
      } catch (err) {
        console.error("CopilotCoordinator listener error:", err);
      }
    }
  }

  /**
   * Sets the active document session ID.
   */
  public setSessionId(sessionId: string): void {
    this.state.sessionId = sessionId;
    this.notify();
  }

  /**
   * Updates the active selection context from the current editor canvas.
   */
  public setActiveSelection(
    sel: {
      kind?: string;
      selected_text?: string | null;
      selected_range?: string | null;
    } | null
  ): void {
    this.state.activeSelection = sel;
    this.notify();
  }

  /**
   * Restricts search and grounding to specific document IDs (or null for all open documents).
   */
  public setTargetDocIds(docIds: string[] | null): void {
    this.state.targetDocIds = docIds;
    this.notify();
  }

  /**
   * Fetches the current provider configuration and connection health.
   */
  public async fetchProviderConfig(): Promise<ProviderStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.copilotGetProviderConfig();
      this.state.providerStatus = status;
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Configures a new LLM provider (Ollama, WebGPU, Cloud, Offline).
   */
  public async configureProvider(config: ProviderConfigDto): Promise<ProviderStatusDto> {
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const status = await IPC.copilotConfigureProvider(config);
      this.state.providerStatus = status;
      this.state.isLoading = false;
      this.notify();
      return status;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Indexes a document into the multi-document grounding corpus.
   */
  public async indexDocument(docId: string, title: string, text: string): Promise<number> {
    this.state.errorMessage = null;

    try {
      const count = await IPC.copilotIndexDocument(docId, title, text);
      if (!this.state.indexedDocIds.includes(docId)) {
        this.state.indexedDocIds.push(docId);
      }
      this.notify();
      return count;
    } catch (err: unknown) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Removes a closed document session from the index.
   */
  public async removeDocument(docId: string): Promise<number> {
    this.state.errorMessage = null;

    try {
      const count = await IPC.copilotRemoveDocument(docId);
      this.state.indexedDocIds = this.state.indexedDocIds.filter((id) => id !== docId);
      this.notify();
      return count;
    } catch (err: unknown) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Dispatches a user query to the grounded AI Copilot engine.
   */
  public async sendMessage(prompt: string): Promise<CopilotChatResponseDto> {
    if (!prompt.trim()) {
      throw new Error("Prompt cannot be empty");
    }

    const userMsg: CopilotChatMessageDto = {
      id: `user-msg-${Date.now()}`,
      sender: "user",
      text: prompt,
      timestamp_ms: Date.now(),
      citations: [],
      diff: null,
    };
    this.state.messages.push(userMsg);
    this.state.isLoading = true;
    this.state.errorMessage = null;
    this.notify();

    try {
      const request: CopilotChatRequestDto = {
        query: prompt,
        session_id: this.state.sessionId,
        active_selection: this.state.activeSelection,
        target_doc_ids: this.state.targetDocIds,
        conversation_history: this.state.messages,
      };

      const resp = await IPC.copilotChat(request);

      const assistantMsg: CopilotChatMessageDto = {
        id: resp.message_id,
        sender: "assistant",
        text: resp.response_text,
        timestamp_ms: Date.now(),
        citations: resp.citations,
        diff: resp.suggested_diff || null,
      };

      this.state.messages.push(assistantMsg);
      if (resp.suggested_diff) {
        this.state.pendingDiff = resp.suggested_diff;
      }
      this.state.isLoading = false;
      this.notify();
      return resp;
    } catch (err: unknown) {
      this.state.isLoading = false;
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Applies the active pending diff proposal.
   */
  public applyPendingDiff(): GroundedDiffDto | null {
    const diff = this.state.pendingDiff;
    this.state.pendingDiff = null;
    this.notify();
    return diff;
  }

  /**
   * Dismisses the pending diff proposal.
   */
  public clearPendingDiff(): void {
    this.state.pendingDiff = null;
    this.notify();
  }

  /**
   * Clears the active conversation thread history.
   */
  public clearConversation(): void {
    this.state.messages = [];
    this.state.pendingDiff = null;
    this.notify();
  }

  /**
   * Clears the error banner.
   */
  public clearError(): void {
    this.state.errorMessage = null;
    this.notify();
  }
}

