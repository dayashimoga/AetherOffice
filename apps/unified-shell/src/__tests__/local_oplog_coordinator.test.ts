/**
 * local_oplog_coordinator.test.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.1.1)
 *
 * Vitest Unit & Integration Suite for LocalOpLogCoordinator:
 * - Default state initialization and session metadata
 * - Appending text insertion operations and materializing live document state
 * - Appending text deletion operations
 * - Appending spreadsheet cell updates
 * - Appending presentation shape mutations
 * - Milestone snapshot checkpoints creation and history reloading
 * - Non-destructive time-travel scrubbing with preview materialization
 * - Exiting time-travel scrubbing and returning to live forward state
 * - Forking new document sessions from exact historical operation IDs
 * - Offline/online connection status, unSyncedCount tracking, and peer delta reconciliation
 */

import { describe, it, expect, beforeEach } from "vitest";
import { LocalOpLogCoordinator } from "../state/LocalOpLogCoordinator";

describe("Sub-Phase 51.1.1: LocalOpLogCoordinator", () => {
  let coordinator: LocalOpLogCoordinator;
  const docId = "test-doc-collab";

  beforeEach(() => {
    coordinator = new LocalOpLogCoordinator(docId, "author-alice", 1);
  });

  it("1. initializes with default state, authorId, and online status", () => {
    const state = coordinator.getState();
    expect(state.docId).toBe(docId);
    expect(state.authorId).toBe("author-alice");
    expect(state.clientId).toBe(1);
    expect(state.history).toEqual([]);
    expect(state.checkpoints).toEqual([]);
    expect(state.syncStatus).toBe("online");
    expect(state.isScrubbing).toBe(false);
    expect(state.scrubTimestamp).toBeNull();
    expect(state.previewState).toBeNull();
    expect(state.latestState).toBeNull();
    expect(state.unSyncedCount).toBe(0);
    expect(state.errorMessage).toBeNull();
  });

  it("2. appends text insert operations and updates live state", async () => {
    const entry = await coordinator.appendTextInsert("Hello AetherOffice");
    expect(entry).toBeDefined();
    expect(entry.doc_id).toBe(docId);
    expect(entry.author_id).toBe("author-alice");
    expect("TextInsert" in entry.payload).toBe(true);

    const state = coordinator.getState();
    expect(state.history.length).toBe(1);
    expect(state.latestState).toBeDefined();
    expect(state.latestState?.text_content).toContain("Hello AetherOffice");
  });

  it("3. appends text delete operations", async () => {
    const ins = await coordinator.appendTextInsert("Antigravity");
    const del = await coordinator.appendTextDelete(ins.entry_id);

    expect(del).toBeDefined();
    expect("TextDelete" in del.payload).toBe(true);
    expect(coordinator.getState().history.length).toBe(2);
  });

  it("4. appends spreadsheet cell updates", async () => {
    const entry = await coordinator.appendCellUpdate(2, 3, { Number: 99.5 });
    expect(entry).toBeDefined();
    expect("CellUpdate" in entry.payload).toBe(true);
    if ("CellUpdate" in entry.payload) {
      expect(entry.payload.CellUpdate.row).toBe(2);
      expect(entry.payload.CellUpdate.col).toBe(3);
    }
    expect(coordinator.getState().latestState?.cells["2,3"]).toBeDefined();
  });

  it("5. appends presentation shape mutations", async () => {
    const json = JSON.stringify({ type: "rect", x: 10, y: 20 });
    const entry = await coordinator.appendShapeMutation("shape-header", json);
    expect(entry).toBeDefined();
    expect("ShapeMutation" in entry.payload).toBe(true);
    if ("ShapeMutation" in entry.payload) {
      expect(entry.payload.ShapeMutation.shape_id).toBe("shape-header");
    }
    expect(coordinator.getState().latestState?.shapes["shape-header"]).toBe(json);
  });

  it("6. creates milestone snapshot checkpoints and reloads history", async () => {
    const cp = await coordinator.createCheckpoint("v1.0-release");
    expect(cp).toBeDefined();
    expect(cp.label).toBe("v1.0-release");
    expect(cp.doc_id).toBe(docId);

    const state = coordinator.getState();
    expect(state.checkpoints.length).toBeGreaterThanOrEqual(1);
    expect(state.checkpoints.some((c) => c.label === "v1.0-release")).toBe(true);
  });

  it("7. performs time-travel scrubbing without mutating live state", async () => {
    const t0 = 1000;
    await coordinator.appendTextInsert("Initial Draft", undefined);

    const preview = await coordinator.timeTravelTo(t0);
    expect(preview).toBeDefined();

    const state = coordinator.getState();
    expect(state.isScrubbing).toBe(true);
    expect(state.scrubTimestamp).toBe(t0);
    expect(state.previewState).toBe(preview);
    expect(state.latestState).toBeDefined();
  });

  it("8. exits time-travel scrubbing mode and restores live viewport", async () => {
    await coordinator.timeTravelTo(1500);
    expect(coordinator.getState().isScrubbing).toBe(true);

    coordinator.exitTimeTravel();
    const state = coordinator.getState();
    expect(state.isScrubbing).toBe(false);
    expect(state.scrubTimestamp).toBeNull();
    expect(state.previewState).toBeNull();
  });

  it("9. forks document session from historical operation ID", async () => {
    const op = await coordinator.appendTextInsert("Base for Fork");
    const forked = await coordinator.forkVersion(op.entry_id, "doc-forked-branch");

    expect(forked).toBeDefined();
    expect(forked.doc_id).toBe("doc-forked-branch");
  });

  it("10. manages offline unSyncedCount and synchronizes with peer via deltas", async () => {
    // Go offline
    coordinator.setOnlineStatus(false);
    expect(coordinator.getState().syncStatus).toBe("offline");

    // Offline edits increment unSyncedCount
    await coordinator.appendTextInsert("Offline Note");
    expect(coordinator.getState().unSyncedCount).toBe(1);

    // Simulate peer reconnection & sync
    const remotePeerVector = { clocks: { 2: 1 } };
    const remoteEntries = [
      {
        entry_id: { lamport: 2, client_id: 2, seq: 1 },
        parent_ids: [{ lamport: 0, client_id: 0, seq: 0 }],
        timestamp_ms: 1200,
        author_id: "author-bob",
        doc_id: docId,
        payload: {
          TextInsert: {
            parent_id: { lamport: 0, client_id: 0, seq: 0 },
            content: " Peer Contribution",
            right_origin: null,
          },
        },
        hash: "remote-hash-1",
      },
    ];

    const report = await coordinator.syncWithPeer(remotePeerVector, remoteEntries);
    expect(report.applied).toBe(1);

    const finalState = coordinator.getState();
    expect(finalState.syncStatus).toBe("online");
    expect(finalState.unSyncedCount).toBe(0);
  });
});
