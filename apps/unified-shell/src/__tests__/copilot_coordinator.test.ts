/**
 * copilot_coordinator.test.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.2.2)
 *
 * Vitest Unit & Integration Suite for CopilotCoordinator:
 * - Default state initialization and subscriber notification
 * - Fetching provider configuration
 * - Configuring and switching provider (LocalOllama, WebGPU, Cloud API)
 * - Multi-document corpus indexing across Word and Calc sessions
 * - Conversational chat queries with grounded citations
 * - Active canvas selection priority and diff generation
 * - Applying and clearing pending diff proposals
 * - Document target filtering
 * - Document removal and index pruning
 * - Clearing conversation threads and error handling
 */

import { describe, it, expect, beforeEach } from "vitest";
import { CopilotCoordinator } from "../state/CopilotCoordinator";

describe("Sub-Phase 51.2.2: CopilotCoordinator", () => {
  let coordinator: CopilotCoordinator;
  const sessionId = "doc-session-test";

  beforeEach(() => {
    coordinator = new CopilotCoordinator(sessionId);
  });

  it("1. initializes with empty messages, null selection, and default state", () => {
    const state = coordinator.getState();
    expect(state.sessionId).toBe(sessionId);
    expect(state.messages).toEqual([]);
    expect(state.providerStatus).toBeNull();
    expect(state.indexedDocIds).toEqual([]);
    expect(state.activeSelection).toBeNull();
    expect(state.targetDocIds).toBeNull();
    expect(state.pendingDiff).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.errorMessage).toBeNull();
  });

  it("2. fetches initial provider configuration", async () => {
    const status = await coordinator.fetchProviderConfig();
    expect(status).toBeDefined();
    expect(status.provider_kind).toBe("OfflineTemplate");
    expect(status.is_available).toBe(true);
    expect(coordinator.getState().providerStatus).toEqual(status);
  });

  it("3. configures a new provider (LocalOllama) and updates provider status", async () => {
    const newStatus = await coordinator.configureProvider({
      provider_kind: "LocalOllama",
      base_url: "http://localhost:11434",
      model_name: "llama3.2",
      api_key: null,
      timeout_ms: 12000,
      temperature: 0.1,
      max_tokens: 4096,
    });

    expect(newStatus.provider_kind).toBe("LocalOllama");
    expect(newStatus.endpoint).toBe("http://localhost:11434");
    expect(coordinator.getState().providerStatus?.provider_kind).toBe("LocalOllama");
  });

  it("4. indexes multiple documents into the grounding corpus", async () => {
    const count1 = await coordinator.indexDocument(
      "doc-whitepaper",
      "Enterprise Whitepaper",
      "AetherOffice runs entirely in local Rust with zero cloud dependencies."
    );
    expect(count1).toBeGreaterThan(0);

    const count2 = await coordinator.indexDocument(
      "doc-finances",
      "Q4 Financial Plan",
      "Revenue target is 500,000 with 15% operating margin."
    );
    expect(count2).toBeGreaterThan(0);

    const state = coordinator.getState();
    expect(state.indexedDocIds).toContain("doc-whitepaper");
    expect(state.indexedDocIds).toContain("doc-finances");
  });

  it("5. sends user chat query and receives grounded response with citations", async () => {
    const resp = await coordinator.sendMessage("What are the revenue targets?");
    expect(resp).toBeDefined();
    expect(resp.response_text).toContain("500,000");
    expect(resp.citations.length).toBeGreaterThan(0);
    expect(resp.citations[0].doc_id).toBeDefined();

    const state = coordinator.getState();
    expect(state.messages.length).toBe(2); // user + assistant
    expect(state.messages[0].sender).toBe("user");
    expect(state.messages[1].sender).toBe("assistant");
  });

  it("6. prioritizes active canvas selection and generates structured diff proposal", async () => {
    coordinator.setActiveSelection({
      kind: "Word",
      selected_text: "Our team plans to ship in Q4.",
      selected_range: "Paragraph 1",
    });

    const resp = await coordinator.sendMessage("Revise this text into an executive summary.");
    expect(resp.suggested_diff).not.toBeNull();
    expect(resp.suggested_diff?.original).toContain("Our team plans to ship");
    expect(resp.suggested_diff?.modified).toContain("Executive Summary");

    const state = coordinator.getState();
    expect(state.pendingDiff).toEqual(resp.suggested_diff);
  });

  it("7. applies pending diff proposal and updates state", async () => {
    coordinator.setActiveSelection({
      kind: "Word",
      selected_text: "Our team plans to ship in Q4.",
      selected_range: "Paragraph 1",
    });
    await coordinator.sendMessage("Revise this text into an executive summary.");
    expect(coordinator.getState().pendingDiff).not.toBeNull();

    const applied = coordinator.applyPendingDiff();
    expect(applied).not.toBeNull();
    expect(applied?.modified).toContain("Executive Summary");
    expect(coordinator.getState().pendingDiff).toBeNull();
  });

  it("8. restricts retrieval using targetDocIds filter", async () => {
    coordinator.setActiveSelection(null);
    coordinator.setTargetDocIds(["doc-whitepaper"]);

    const resp = await coordinator.sendMessage("Summarize architecture.");
    expect(resp.citations.every((c) => c.doc_id === "doc-whitepaper")).toBe(true);

    coordinator.setTargetDocIds(null); // Reset filter
  });

  it("9. removes closed document from grounding corpus", async () => {
    await coordinator.indexDocument("doc-finances", "Finances", "Some financial data");
    const removedCount = await coordinator.removeDocument("doc-finances");
    expect(removedCount).toBeGreaterThan(0);
    expect(coordinator.getState().indexedDocIds).not.toContain("doc-finances");
  });

  it("10. clears conversation thread history and resets state", async () => {
    await coordinator.sendMessage("Test message for history.");
    expect(coordinator.getState().messages.length).toBeGreaterThan(0);

    coordinator.clearConversation();
    expect(coordinator.getState().messages).toEqual([]);
    expect(coordinator.getState().pendingDiff).toBeNull();
  });
});
