# Sub-Phase 51.2.2: Multi-Document Grounded AI Copilot (Local Ollama/WebGPU + Cloud API)

## 1. Context & Objectives
As defined in GEC-v7.0 and Stage 51 of the master roadmap, Sub-Phase 51.2.2 advances the AI assistance layer (`copilot-core`) into a full multi-document grounded assistant supporting local Ollama / WebGPU on-device inferencing and external cloud endpoints. The copilot seamlessly queries across concurrent open sessions (`.docx`, `.xlsx`, `.pptx`, `.pdf`, `.md`), prioritizes active selections, constructs budget-capped grounded prompt contexts, tracks verbatim citations, and proposes structured editing diffs.

Key objectives:
1. **Multi-Document Context & Retrieval Routing (`copilot-core`)**:
   - Route user queries across multiple active indexed documents simultaneously.
   - Rank and fuse semantic and lexical search matches across distinct document formats.
   - Assemble context windows under strict token budgets while guaranteeing active document selection priority.
2. **Provider Hierarchy & In-Memory Fallback**:
   - Provider abstraction: `LocalOllama`, `CloudApi`, and deterministic `OfflineTemplate`.
   - Provider configuration and switching with latency and connectivity probing.
   - Structured hallucination guard and citation grounding validation.
3. **Tauri IPC Command Endpoints**:
   - `copilot_chat`: Multi-turn conversational chat with grounded context synthesis.
   - `copilot_configure_provider`: Configures model provider, base URL, and credentials.
   - `copilot_get_provider_config`: Retrieves active provider status and model details.
   - `copilot_index_document`: Ingests/updates document session contents into the search index.
   - `copilot_remove_document`: Prunes closed document sessions from the index.
   - `copilot_grounded_suggest`: Generates contextual editing suggestions with inline diff.
4. **Presentation Shell Coordinator (`CopilotCoordinator.ts`)**:
   - Reactive state manager for multi-turn chats, provider switching, citation clicking, and diff reviews.
   - Exported from `apps/unified-shell/src/state/index.ts`.
5. **Quality & Verification**:
   - 10 unit & integration tests in `crates/copilot-core/tests/subphase_51_2_2_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/copilot_coordinator.test.ts`.
   - Master gatekeeper `tools/verify-contract.ps1` exit code 0.

