import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { collabState, COLLAB_PEER_COLORS } from "../state/collabState";
import { CollabBar } from "../components/CollabBar";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { WriterState } from "../state/writerState";
import { CalcState } from "../state/calcState";
import { IPC, PeerPresence } from "../ipc";

describe("Live Remote Awareness, Cursors & Selection Highlights (GEC-v2.0 Subphase 14.2.2)", () => {
  let mockCtx: any;

  beforeEach(() => {
    collabState.clear();
    IPC._resetMockState();

    mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      measureText: vi.fn((text: string) => ({ width: text.length * 8 })),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      arc: vi.fn(),
      closePath: vi.fn(),
      fill: vi.fn(),
      stroke: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      setTransform: vi.fn(),
      scale: vi.fn(),
      setLineDash: vi.fn(),
      rect: vi.fn(),
      roundRect: vi.fn(),
      clip: vi.fn(),
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="window-controls"></div>
        </header>
        <div id="sub-header-container"></div>
        <div id="sidebar-container"></div>
        <div id="viewport-container" style="width: 800px; height: 600px;"></div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
        </div>
      </div>
    `;
  });

  afterEach(() => {
    collabState.clear();
  });

  // =========================================================================
  // 1. collabState Store Lifecycle & Remote Peer Tracking
  // =========================================================================
  describe("1. collabState Store Lifecycle & Multi-Peer State", () => {
    it("initializes with disconnected state and defaults", () => {
      const snap = collabState.getSnapshot();
      expect(snap.isConnected).toBe(false);
      expect(snap.activeSession).toBeNull();
      expect(snap.peers).toHaveLength(0);
      expect(COLLAB_PEER_COLORS).toContain(snap.myColor);
    });

    it("joins collaborative session via IPC and populates session info", async () => {
      const info = await collabState.joinSession(
        "q4-financial-report",
        "Lead Auditor",
        "calc",
        "enc-secret-pass",
        "webrtc"
      );

      expect(info.session_id).toBe("q4-financial-report");
      expect(info.username).toBe("Lead Auditor");
      expect(info.status).toBe("Connected");

      const snap = collabState.getSnapshot();
      expect(snap.isConnected).toBe(true);
      expect(snap.myUsername).toBe("Lead Auditor");
      expect(snap.myClientId).toBe(101);
    });

    it("registers and updates remote peer presence (cursor & selection)", () => {
      const peer1: PeerPresence = {
        client_id: 201,
        username: "Alice",
        color_hex: "#E53935",
        active_document: "Doc1.docx",
        active_tab: "Main",
        cursor: { text_offset: 45 },
        selection: { text_start: 40, text_end: 60 },
        last_seen_ms: Date.now(),
      };

      collabState.setRemotePeerPresence(peer1);
      expect(collabState.getPeers()).toHaveLength(1);

      const retrieved = collabState.getPeer(201);
      expect(retrieved?.username).toBe("Alice");
      expect(retrieved?.cursor?.text_offset).toBe(45);
      expect(retrieved?.selection?.text_end).toBe(60);

      // Peer moves cursor
      collabState.setRemotePeerPresence({
        ...peer1,
        cursor: { text_offset: 120 },
      });
      expect(collabState.getPeer(201)?.cursor?.text_offset).toBe(120);
    });

    it("removes peer on disconnect and clears on leaveSession", async () => {
      await collabState.joinSession("room-99", "Tester", "writer");

      const peerBob: PeerPresence = {
        client_id: 301,
        username: "Bob",
        color_hex: "#1E88E5",
        active_document: "Budget.xlsx",
        active_tab: "Sheet1",
        last_seen_ms: Date.now(),
      };
      collabState.setRemotePeerPresence(peerBob);
      expect(collabState.getPeers()).toHaveLength(1);

      collabState.removePeer(301);
      expect(collabState.getPeers()).toHaveLength(0);

      // Leave entire session
      const left = await collabState.leaveSession("Done working");
      expect(left).toBe(true);
      expect(collabState.getSnapshot().isConnected).toBe(false);
    });

    it("broadcasts text and grid deltas via IPC", async () => {
      await collabState.joinSession("room-delta", "Author", "writer");

      const textOk = await collabState.broadcastTextDelta({ type: "Insert", content: "A" });
      expect(textOk).toBe(true);

      const gridOk = await collabState.broadcastGridDelta({ row: 0, col: 0, val: 42 });
      expect(gridOk).toBe(true);
    });
  });

  // =========================================================================
  // 2. CollabBar UI Header & Share / Join Modal
  // =========================================================================
  describe("2. CollabBar UI Header Presence & Share Modal", () => {
    it("renders avatar chip and Share button in header bar", () => {
      const collabBar = new CollabBar("collab-presence-bar", "writer");

      const selfChip = document.querySelector(".collab-avatar-chip");
      expect(selfChip).not.toBeNull();
      expect(selfChip?.textContent).toBeTruthy();

      const shareBtn = document.getElementById("btn-collab-share");
      expect(shareBtn).not.toBeNull();
      expect(shareBtn?.textContent).toContain("Share");

      collabBar.destroy();
    });

    it("renders remote peer avatars with color badges and overflow counter", () => {
      const collabBar = new CollabBar("collab-presence-bar", "writer");

      // Add 4 peers (> maxVisible 3)
      for (let i = 1; i <= 4; i++) {
        collabState.setRemotePeerPresence({
          client_id: 100 + i,
          username: `Peer_${i}`,
          color_hex: COLLAB_PEER_COLORS[i % COLLAB_PEER_COLORS.length],
          active_document: "Doc.docx",
          active_tab: "Tab1",
          last_seen_ms: Date.now(),
        });
      }

      const chips = document.querySelectorAll(".collab-avatar-chip");
      // 1 self + 3 visible peers = 4 chips
      expect(chips.length).toBe(4);

      const overflow = document.querySelector(".collab-avatar-overflow");
      expect(overflow).not.toBeNull();
      expect(overflow?.textContent).toBe("+1");

      collabBar.destroy();
    });

    it("toggles and interacts with Share / Join Session modal", async () => {
      const collabBar = new CollabBar("collab-presence-bar", "writer");

      // Modal closed initially
      expect(document.getElementById("collab-modal-dialog")).toBeNull();

      // Click share button opens modal
      const shareBtn = document.getElementById("btn-collab-share") as HTMLButtonElement;
      shareBtn.click();

      expect(document.getElementById("collab-modal-dialog")).not.toBeNull();
      expect(document.getElementById("collab-session-id")).not.toBeNull();
      expect(document.getElementById("collab-room-key")).not.toBeNull();
      expect(document.getElementById("collab-username")).not.toBeNull();

      // Close modal
      const closeBtn = document.getElementById("btn-collab-close") as HTMLButtonElement;
      closeBtn.click();
      expect(document.getElementById("collab-modal-dialog")).toBeNull();

      collabBar.destroy();
    });

    it("executes Join Session from modal form inputs", async () => {
      const collabBar = new CollabBar("collab-presence-bar", "writer");
      collabState.setModalOpen(true);

      const sessInput = document.getElementById("collab-session-id") as HTMLInputElement;
      sessInput.value = "boardroom-2026";

      const userInput = document.getElementById("collab-username") as HTMLInputElement;
      userInput.value = "Director Jane";

      const joinBtn = document.getElementById("btn-collab-join") as HTMLButtonElement;
      joinBtn.click();

      // Wait for async join
      await new Promise((resolve) => setTimeout(resolve, 10));

      expect(collabState.getSnapshot().isConnected).toBe(true);
      expect(collabState.getSnapshot().myUsername).toBe("Director Jane");

      collabBar.destroy();
    });
  });

  // =========================================================================
  // 3. WriterCanvas Remote Caret & Selection Highlights
  // =========================================================================
  describe("3. WriterCanvas Multi-User Remote Rendering", () => {
    it("renders remote colored caret and username badge for active peers", () => {
      const vp = document.getElementById("viewport-container")!;
      const state = new WriterState();
      state.insertText("The quick brown fox jumps over the lazy dog.");
      const canvas = new WriterCanvas(vp, state);

      // Add remote peer with caret at offset 16 (after "brown ")
      collabState.setRemotePeerPresence({
        client_id: 501,
        username: "Bob",
        color_hex: "#1E88E5",
        active_document: "Doc.docx",
        active_tab: "Main",
        cursor: { text_offset: 16 },
        last_seen_ms: Date.now(),
      });

      canvas.render();

      // Verify that canvas context rendered a remote vertical caret and badge
      expect(mockCtx.fillRect).toHaveBeenCalled();
      expect(mockCtx.fillText).toHaveBeenCalledWith("Bob", expect.any(Number), expect.any(Number));
    });

    it("renders remote selection highlight rectangle with peer tint", () => {
      const vp = document.getElementById("viewport-container")!;
      const state = new WriterState();
      state.insertText("AetherOffice Real-Time Collaborative Word Processor");
      const canvas = new WriterCanvas(vp, state);

      // Peer selects "Real-Time Collaborative" (offsets 13..38)
      collabState.setRemotePeerPresence({
        client_id: 502,
        username: "Alice",
        color_hex: "#FB8C00",
        active_document: "Doc.docx",
        active_tab: "Main",
        selection: { text_start: 13, text_end: 38 },
        last_seen_ms: Date.now(),
      });

      canvas.render();
      expect(mockCtx.fillRect).toHaveBeenCalled();
    });
  });

  // =========================================================================
  // 4. CalcGridCanvas Remote Cell Outlines & Selection Boxes
  // =========================================================================
  describe("4. CalcGridCanvas Multi-User Remote Rendering", () => {
    it("renders remote active cell outline and name badge pill", () => {
      const vp = document.getElementById("viewport-container")!;
      const state = new CalcState();
      const canvas = new CalcGridCanvas(vp, state);

      // Peer cursor on cell (Row 4, Col 2) -> C5
      collabState.setRemotePeerPresence({
        client_id: 601,
        username: "Charlie",
        color_hex: "#43A047",
        active_document: "Sheet1",
        active_tab: "Grid",
        cursor: { grid_row: 4, grid_col: 2 },
        last_seen_ms: Date.now(),
      });

      canvas.render();

      expect(mockCtx.strokeRect).toHaveBeenCalled();
      expect(mockCtx.fillText).toHaveBeenCalledWith("Charlie", expect.any(Number), expect.any(Number));
    });

    it("renders multi-cell selection range box with dashed border for remote peers", () => {
      const vp = document.getElementById("viewport-container")!;
      const state = new CalcState();
      const canvas = new CalcGridCanvas(vp, state);

      // Peer selects B2:D6
      collabState.setRemotePeerPresence({
        client_id: 602,
        username: "Dana",
        color_hex: "#8E24AA",
        active_document: "Sheet1",
        active_tab: "Grid",
        selection: {
          grid_start_row: 1,
          grid_start_col: 1,
          grid_end_row: 5,
          grid_end_col: 3,
        },
        last_seen_ms: Date.now(),
      });

      canvas.render();

      expect(mockCtx.setLineDash).toHaveBeenCalledWith([4, 4]);
      expect(mockCtx.strokeRect).toHaveBeenCalled();
      expect(mockCtx.fillRect).toHaveBeenCalled();
    });
  });
});

