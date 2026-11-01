/**
 * LocalOpLogCoordinator.ts
 * GEC-v7.0 Stage 51 (Sub-Phase 51.1.1)
 *
 * Presentation Shell Coordinator for:
 * - Local-First Operation Log, Version Time-Travel & Conflict-Free Reconnection
 * - Append-only operation recording across text, spreadsheet cells, and slide shapes
 * - Interactive time-travel scrubbing, historical state materialization, and preview
 * - Milestone checkpoints and document branch forking
 * - Offline/online synchronization and deterministic peer delta reconciliation
 */

import {
  IPC,
  type CheckpointMetaDto,
  type MaterializedDocumentStateDto,
  type OpIdDto,
  type OpLogEntryDto,
  type OpPayloadDto,
  type StateVectorDto,
  type SyncReportDto,
} from "../ipc";

export {
  type CheckpointMetaDto,
  type MaterializedDocumentStateDto,
  type OpIdDto,
  type OpLogEntryDto,
  type OpPayloadDto,
  type StateVectorDto,
  type SyncReportDto,
};

export type OpLogSyncStatus = "online" | "offline" | "syncing" | "desynced";

export interface LocalOpLogState {
  docId: string;
  authorId: string;
  clientId: number;
  history: OpLogEntryDto[];
  checkpoints: CheckpointMetaDto[];
  stateVector: StateVectorDto;
  syncStatus: OpLogSyncStatus;
  isScrubbing: boolean;
  scrubTimestamp: number | null;
  previewState: MaterializedDocumentStateDto | null;
  latestState: MaterializedDocumentStateDto | null;
  unSyncedCount: number;
  errorMessage: string | null;
}

export type LocalOpLogListener = (state: Readonly<LocalOpLogState>) => void;

export class LocalOpLogCoordinator {
  private state: LocalOpLogState;
  private listeners = new Set<LocalOpLogListener>();

  constructor(docId: string, authorId = "local-user", clientId = 1) {
    this.state = {
      docId,
      authorId,
      clientId,
      history: [],
      checkpoints: [],
      stateVector: { clocks: {} },
      syncStatus: "online",
      isScrubbing: false,
      scrubTimestamp: null,
      previewState: null,
      latestState: null,
      unSyncedCount: 0,
      errorMessage: null,
    };
  }

  /**
   * Initializes or reloads history and checkpoints for the active document.
   */
  public async loadHistory(): Promise<void> {
    try {
      const [history, checkpoints, sv] = await Promise.all([
        IPC.oplogGetHistory(this.state.docId),
        IPC.oplogGetCheckpoints(this.state.docId),
        IPC.oplogGetStateVector(this.state.docId),
      ]);

      this.state.history = history;
      this.state.checkpoints = checkpoints;
      this.state.stateVector = sv;

      if (history.length > 0) {
        const latestTime = history[history.length - 1].timestamp_ms;
        this.state.latestState = await IPC.oplogTimeTravel(this.state.docId, latestTime);
      } else {
        this.state.latestState = null;
      }

      this.notify();
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
    }
  }

  /**
   * Appends an arbitrary operation payload to the document's local log.
   */
  public async appendOperation(payload: OpPayloadDto, timestampMs?: number): Promise<OpLogEntryDto> {
    const time = timestampMs ?? Date.now();
    try {
      const entry = await IPC.oplogAppendEntry(
        this.state.docId,
        this.state.authorId,
        payload,
        time,
        this.state.clientId
      );

      this.state.history.push(entry);
      if (this.state.syncStatus === "offline") {
        this.state.unSyncedCount++;
      }

      // Materialize new latest state if not actively scrubbing
      if (!this.state.isScrubbing) {
        this.state.latestState = await IPC.oplogTimeTravel(this.state.docId, time);
      }

      this.notify();
      return entry;
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Helper: Appends a text insertion operation.
   */
  public async appendTextInsert(content: string, parentId?: OpIdDto): Promise<OpLogEntryDto> {
    const pId = parentId ?? { lamport: 0, client_id: 0, seq: 0 };
    return this.appendOperation({
      TextInsert: {
        parent_id: pId,
        content,
        right_origin: null,
      },
    });
  }

  /**
   * Helper: Appends a text deletion operation.
   */
  public async appendTextDelete(targetId: OpIdDto): Promise<OpLogEntryDto> {
    return this.appendOperation({
      TextDelete: {
        target_id: targetId,
      },
    });
  }

  /**
   * Helper: Appends a spreadsheet cell update operation.
   */
  public async appendCellUpdate(row: number, col: number, value: unknown): Promise<OpLogEntryDto> {
    return this.appendOperation({
      CellUpdate: {
        row,
        col,
        value,
      },
    });
  }

  /**
   * Helper: Appends a slide shape mutation operation.
   */
  public async appendShapeMutation(shapeId: string, propertiesJson: string): Promise<OpLogEntryDto> {
    return this.appendOperation({
      ShapeMutation: {
        shape_id: shapeId,
        properties_json: propertiesJson,
      },
    });
  }

  /**
   * Creates a named milestone checkpoint.
   */
  public async createCheckpoint(label: string): Promise<CheckpointMetaDto> {
    const time = Date.now();
    try {
      const cp = await IPC.oplogCreateCheckpoint(
        this.state.docId,
        this.state.authorId,
        label,
        this.state.clientId,
        time
      );

      this.state.checkpoints.push(cp);
      await this.loadHistory();
      return cp;
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Scrubs timeline to a specific historical timestamp without mutating live state.
   */
  public async timeTravelTo(timestampMs: number): Promise<MaterializedDocumentStateDto> {
    try {
      const preview = await IPC.oplogTimeTravel(this.state.docId, timestampMs);
      this.state.isScrubbing = true;
      this.state.scrubTimestamp = timestampMs;
      this.state.previewState = preview;
      this.notify();
      return preview;
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Exits time-travel scrub mode and returns viewport to live forward state.
   */
  public exitTimeTravel(): void {
    this.state.isScrubbing = false;
    this.state.scrubTimestamp = null;
    this.state.previewState = null;
    this.notify();
  }

  /**
   * Forks a new document session starting from an exact historical operation ID.
   */
  public async forkVersion(
    targetOpId: OpIdDto,
    newDocId: string
  ): Promise<MaterializedDocumentStateDto> {
    const time = Date.now();
    try {
      const forkedState = await IPC.oplogForkVersion(
        this.state.docId,
        targetOpId,
        newDocId,
        this.state.authorId,
        this.state.clientId,
        time
      );
      return forkedState;
    } catch (err) {
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Sets offline/online connection status.
   */
  public setOnlineStatus(online: boolean): void {
    this.state.syncStatus = online ? (this.state.unSyncedCount > 0 ? "syncing" : "online") : "offline";
    this.notify();
  }

  /**
   * Synchronizes with a remote peer by exchanging deltas and applying remote operations.
   */
  public async syncWithPeer(
    remoteVector: StateVectorDto,
    incomingRemoteEntries: OpLogEntryDto[]
  ): Promise<SyncReportDto> {
    this.state.syncStatus = "syncing";
    this.notify();

    try {
      // 1. Calculate outgoing entries needed by remote peer
      await IPC.oplogComputeSyncDelta(this.state.docId, remoteVector);

      // 2. Apply incoming remote entries locally
      const report = await IPC.oplogApplyRemoteDelta(this.state.docId, incomingRemoteEntries);

      this.state.unSyncedCount = 0;
      this.state.syncStatus = "online";
      this.state.stateVector = report.state_vector;

      await this.loadHistory();
      return report;
    } catch (err) {
      this.state.syncStatus = "desynced";
      this.state.errorMessage = err instanceof Error ? err.message : String(err);
      this.notify();
      throw err;
    }
  }

  /**
   * Subscribes to coordinator state mutations.
   */
  public subscribe(listener: LocalOpLogListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Returns current immutable state.
   */
  public getState(): Readonly<LocalOpLogState> {
    return { ...this.state };
  }

  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      try {
        listener(snapshot);
      } catch (err) {
        console.error("LocalOpLogCoordinator subscriber error:", err);
      }
    }
  }
}

