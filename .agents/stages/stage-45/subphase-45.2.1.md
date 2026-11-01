# Stage 45: High-Performance Canvas Rendering & 120fps Layout Virtualization
## Sub-Phase 45.2.1: LRU Tab Suspension & Instant Rehydration Engine

### 1. Context & Objectives
Under GEC-v7.0 Article I (Zero Global Document Singletons) and Article VI (Performance Budgets & SLAs):
- Multi-document memory budget: $< 250\text{ MB}$ RSS for 20 open documents using 3-tier Active / Warm / Cold LRU session suspension.
- Tabs transition dynamically:
  - **Active:** Tab currently focused or visible in split panes; full DOM, layout, and display list retained in memory.
  - **Warm:** Recently viewed tab; retains lightweight parsed DOM and metadata, releases GPU textures and display lists.
  - **Cold:** Inactive tab exceeding threshold; serialized to compressed snapshot, memory freed.
- Rehydration SLA: Switching to a cold tab rehydrates in $< 50\text{ ms}$ with exact scroll position and caret/selection preserved.

### 2. Architecture & Components
1. **`crates/render-core/src/session_suspension.rs`**:
   - `SessionTier`: `Active`, `Warm`, `Cold`.
   - `SuspensionSnapshot`: Serialized snapshot containing scroll position, caret position, and state payload.
   - `LruTabSuspensionEngine`:
     - `record_access(session_id, doc_type, title, payload)`: marks Active, demotes LRU tabs.
     - `suspend_to_cold(session_id, scroll, caret)`: creates snapshot and transitions to Cold.
     - `rehydrate_session(session_id)`: restores snapshot and returns RehydrationResult.
     - Configurable `max_active` and `max_warm` thresholds.
2. **Frontend Coordinator (`apps/unified-shell/src/state/LruTabSuspensionCoordinator.ts` & `documentSessionRegistry.ts`)**:
   - Integrates with `DocumentSessionRegistry` to update lifecycle states.
   - Preserves UI viewport state (scrollX, scrollY, selection) during tab switching.
   - Restores viewport state upon rehydration in $< 10\text{ ms}$.
3. **Verification & Testing**:
   - Rust test suite `crates/render-core/tests/subphase_45_2_1_test.rs`.
   - Vitest suite `apps/unified-shell/src/__tests__/lru_tab_suspension.test.ts`.
   - Gatekeeper `tools/verify-contract.ps1` exit code 0.

