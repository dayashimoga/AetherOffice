/**
 * Real-Time Collaboration & Remote Presence State Store (GEC-v2.0 Subphase 14.2.2)
 *
 * Tracks active collaboration sessions, remote peer carets, text & cell selections,
 * presence badges, and IPC synchronization.
 */

import { IPC, CollabSessionInfo, PeerPresence, CursorCoord, SelectionSpan } from "../ipc";
export type { CursorCoord, SelectionSpan };

export const COLLAB_PEER_COLORS = [
  "#E53935", // Red
  "#1E88E5", // Blue
  "#43A047", // Green
  "#8E24AA", // Purple
  "#FB8C00", // Orange
  "#00ACC1", // Cyan
  "#D81B60", // Pink
  "#3949AB", // Indigo
];

export interface CollabStateSnapshot {
  activeSession: CollabSessionInfo | null;
  isConnected: boolean;
  isModalOpen: boolean;
  myClientId: number;
  myUsername: string;
  myColor: string;
  peers: PeerPresence[];
}

class CollabStateManager {
  private activeSession: CollabSessionInfo | null = null;
  private isConnected = false;
  private isModalOpen = false;
  private myClientId = 1;
  private myUsername = "Guest User";
  private myColor = COLLAB_PEER_COLORS[0];
  private peersMap = new Map<number, PeerPresence>();
  private listeners: Array<() => void> = [];

  constructor() {
    // Generate a default random user color
    this.myColor = COLLAB_PEER_COLORS[Math.floor(Math.random() * COLLAB_PEER_COLORS.length)];
  }

  public subscribe(listener: () => void): () => void {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  private notify(): void {
    for (const listener of this.listeners) {
      try {
        listener();
      } catch (err) {
        console.error("CollabState listener error:", err);
      }
    }
  }

  public getSnapshot(): CollabStateSnapshot {
    return {
      activeSession: this.activeSession,
      isConnected: this.isConnected,
      isModalOpen: this.isModalOpen,
      myClientId: this.myClientId,
      myUsername: this.myUsername,
      myColor: this.myColor,
      peers: Array.from(this.peersMap.values()),
    };
  }

  public getPeers(): PeerPresence[] {
    return Array.from(this.peersMap.values());
  }

  public getPeer(clientId: number): PeerPresence | undefined {
    return this.peersMap.get(clientId);
  }

  public setModalOpen(open: boolean): void {
    this.isModalOpen = open;
    this.notify();
  }

  public setMyUsername(username: string): void {
    this.myUsername = username.trim() || "Guest User";
    this.notify();
  }

  public setMyColor(color: string): void {
    this.myColor = color;
    this.notify();
  }

  public async joinSession(
    sessionId: string,
    username: string,
    appMode: string,
    roomKey?: string,
    transportMode = "webrtc"
  ): Promise<CollabSessionInfo> {
    const cleanSessionId = sessionId.trim();
    const cleanUsername = username.trim() || this.myUsername;

    this.myUsername = cleanUsername;
    const sessionInfo = await IPC.collabJoinSession(
      cleanSessionId,
      cleanUsername,
      appMode,
      roomKey,
      transportMode
    );

    this.activeSession = sessionInfo;
    this.myClientId = sessionInfo.client_id;
    this.isConnected = true;
    this.peersMap.clear();

    if (sessionInfo.peers && Array.isArray(sessionInfo.peers)) {
      for (const p of sessionInfo.peers) {
        if (p.client_id !== this.myClientId) {
          this.peersMap.set(p.client_id, p);
        }
      }
    }

    this.notify();
    return sessionInfo;
  }

  public async leaveSession(reason?: string): Promise<boolean> {
    if (!this.activeSession) return false;
    const res = await IPC.collabLeaveSession(this.activeSession.session_id, reason);
    this.activeSession = null;
    this.isConnected = false;
    this.peersMap.clear();
    this.notify();
    return res;
  }

  public async broadcastTextDelta(delta: unknown): Promise<boolean> {
    if (!this.activeSession) return false;
    const jsonStr = typeof delta === "string" ? delta : JSON.stringify(delta);
    return IPC.collabBroadcastDelta(this.activeSession.session_id, "text", jsonStr);
  }

  public async broadcastGridDelta(delta: unknown): Promise<boolean> {
    if (!this.activeSession) return false;
    const jsonStr = typeof delta === "string" ? delta : JSON.stringify(delta);
    return IPC.collabBroadcastDelta(this.activeSession.session_id, "grid", jsonStr);
  }

  public setRemotePeerPresence(peer: PeerPresence): void {
    if (peer.client_id === this.myClientId) return;
    this.peersMap.set(peer.client_id, {
      ...peer,
      last_seen_ms: Date.now(),
    });
    this.notify();
  }

  public removePeer(clientId: number): void {
    if (this.peersMap.delete(clientId)) {
      this.notify();
    }
  }

  public clear(): void {
    this.activeSession = null;
    this.isConnected = false;
    this.peersMap.clear();
    this.isModalOpen = false;
    this.myClientId = 1;
    this.listeners = [];
  }
}

export const collabState = new CollabStateManager();

