/**
 * Real-Time Collaboration Presence Bar & Share Session Modal (GEC-v2.0 Subphase 14.2.2)
 *
 * Renders:
 * 1. Live peer avatar chips with color badges in the header bar.
 * 2. Real-time connection status pill.
 * 3. End-to-end encrypted Share / Join Session modal dialog.
 */

import { collabState, CollabStateSnapshot } from "../state/collabState";

export class CollabBar {
  private container: HTMLElement;
  private modalContainer: HTMLElement | null = null;
  private unsubscribe: (() => void) | null = null;
  private currentAppMode: string;

  constructor(containerId = "collab-presence-bar", appMode = "writer") {
    this.currentAppMode = appMode;
    let el = document.getElementById(containerId);
    if (!el) {
      el = document.createElement("div");
      el.id = containerId;
      el.className = "collab-presence-bar";
      const titleBar = document.querySelector(".title-bar");
      if (titleBar) {
        const winControls = titleBar.querySelector(".window-controls");
        if (winControls) {
          titleBar.insertBefore(el, winControls);
        } else {
          titleBar.appendChild(el);
        }
      } else {
        document.body.appendChild(el);
      }
    }
    this.container = el;

    this.unsubscribe = collabState.subscribe(() => {
      this.render();
    });

    this.render();
  }

  public setAppMode(mode: string): void {
    this.currentAppMode = mode;
  }

  public destroy(): void {
    if (this.unsubscribe) {
      this.unsubscribe();
      this.unsubscribe = null;
    }
    if (this.modalContainer && this.modalContainer.parentNode) {
      this.modalContainer.parentNode.removeChild(this.modalContainer);
      this.modalContainer = null;
    }
  }

  public render(): void {
    const snap = collabState.getSnapshot();
    this.container.innerHTML = "";
    this.container.style.display = "flex";
    this.container.style.alignItems = "center";
    this.container.style.gap = "8px";
    this.container.style.margin = "0 12px";

    // 1. Peer Avatars Stack
    const avatarGroup = document.createElement("div");
    avatarGroup.className = "collab-avatar-group";
    avatarGroup.style.display = "flex";
    avatarGroup.style.alignItems = "center";

    // Self avatar
    const selfChip = this.createAvatarChip(
      snap.myUsername,
      snap.myColor,
      `${snap.myUsername} (You)`,
      0
    );
    avatarGroup.appendChild(selfChip);

    // Remote peer avatars (up to 3, then +N badge)
    const maxVisiblePeers = 3;
    const visiblePeers = snap.peers.slice(0, maxVisiblePeers);
    visiblePeers.forEach((peer, idx) => {
      const chip = this.createAvatarChip(
        peer.username,
        peer.color_hex,
        `${peer.username} - Active on ${peer.active_document || "Document"}`,
        -(idx + 1) * 8
      );
      avatarGroup.appendChild(chip);
    });

    if (snap.peers.length > maxVisiblePeers) {
      const overflowCount = snap.peers.length - maxVisiblePeers;
      const overflowChip = document.createElement("div");
      overflowChip.className = "collab-avatar-overflow";
      overflowChip.textContent = `+${overflowCount}`;
      overflowChip.style.width = "26px";
      overflowChip.style.height = "26px";
      overflowChip.style.borderRadius = "50%";
      overflowChip.style.backgroundColor = "#e0e0e0";
      overflowChip.style.color = "#424242";
      overflowChip.style.fontSize = "11px";
      overflowChip.style.fontWeight = "bold";
      overflowChip.style.display = "flex";
      overflowChip.style.alignItems = "center";
      overflowChip.style.justifyContent = "center";
      overflowChip.style.border = "2px solid #ffffff";
      overflowChip.style.marginLeft = "-8px";
      overflowChip.title = `${overflowCount} additional peers online`;
      avatarGroup.appendChild(overflowChip);
    }

    this.container.appendChild(avatarGroup);

    // 2. Share / Live Status Button
    const shareBtn = document.createElement("button");
    shareBtn.id = "btn-collab-share";
    shareBtn.className = "btn-collab-share";
    shareBtn.style.display = "flex";
    shareBtn.style.alignItems = "center";
    shareBtn.style.gap = "6px";
    shareBtn.style.padding = "4px 10px";
    shareBtn.style.borderRadius = "16px";
    shareBtn.style.border = snap.isConnected
      ? "1px solid #4caf50"
      : "1px solid #d1d5db";
    shareBtn.style.backgroundColor = snap.isConnected ? "#e8f5e9" : "#ffffff";
    shareBtn.style.color = snap.isConnected ? "#2e7d32" : "#374151";
    shareBtn.style.fontWeight = "600";
    shareBtn.style.fontSize = "12px";
    shareBtn.style.cursor = "pointer";
    shareBtn.style.transition = "all 0.15s ease";

    const dot = document.createElement("span");
    dot.style.width = "8px";
    dot.style.height = "8px";
    dot.style.borderRadius = "50%";
    dot.style.backgroundColor = snap.isConnected ? "#4caf50" : "#9e9e9e";
    shareBtn.appendChild(dot);

    const label = document.createElement("span");
    label.textContent = snap.isConnected
      ? `Live (${snap.peers.length + 1})`
      : "Share";
    shareBtn.appendChild(label);

    shareBtn.onclick = () => {
      collabState.setModalOpen(true);
    };

    this.container.appendChild(shareBtn);

    // 3. Render Modal if Open
    this.renderModal(snap);
  }

  private createAvatarChip(
    username: string,
    colorHex: string,
    titleText: string,
    marginLeft: number
  ): HTMLElement {
    const chip = document.createElement("div");
    chip.className = "collab-avatar-chip";
    chip.style.width = "26px";
    chip.style.height = "26px";
    chip.style.borderRadius = "50%";
    chip.style.backgroundColor = colorHex || "#1E88E5";
    chip.style.color = "#ffffff";
    chip.style.fontWeight = "bold";
    chip.style.fontSize = "11px";
    chip.style.display = "flex";
    chip.style.alignItems = "center";
    chip.style.justifyContent = "center";
    chip.style.border = "2px solid #ffffff";
    chip.style.boxShadow = "0 1px 3px rgba(0,0,0,0.12)";
    chip.style.marginLeft = `${marginLeft}px`;
    chip.style.cursor = "pointer";
    chip.title = titleText;

    const initial = (username.trim()[0] || "U").toUpperCase();
    chip.textContent = initial;
    return chip;
  }

  private renderModal(snap: CollabStateSnapshot): void {
    if (!snap.isModalOpen) {
      if (this.modalContainer && this.modalContainer.parentNode) {
        this.modalContainer.parentNode.removeChild(this.modalContainer);
        this.modalContainer = null;
      }
      return;
    }

    if (!this.modalContainer) {
      this.modalContainer = document.createElement("div");
      this.modalContainer.id = "collab-modal-backdrop";
      this.modalContainer.className = "collab-modal-backdrop";
      this.modalContainer.style.position = "fixed";
      this.modalContainer.style.top = "0";
      this.modalContainer.style.left = "0";
      this.modalContainer.style.width = "100vw";
      this.modalContainer.style.height = "100vh";
      this.modalContainer.style.backgroundColor = "rgba(0, 0, 0, 0.4)";
      this.modalContainer.style.display = "flex";
      this.modalContainer.style.alignItems = "center";
      this.modalContainer.style.justifyContent = "center";
      this.modalContainer.style.zIndex = "10000";
      document.body.appendChild(this.modalContainer);
    }

    this.modalContainer.innerHTML = `
      <div id="collab-modal-dialog" class="collab-modal-dialog" style="
        background: #ffffff;
        border-radius: 8px;
        width: 440px;
        box-shadow: 0 10px 25px rgba(0,0,0,0.2);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        font-family: var(--font-family, sans-serif);
      ">
        <div style="
          padding: 16px 20px;
          border-bottom: 1px solid #e5e7eb;
          display: flex;
          align-items: center;
          justify-content: space-between;
          background: #f9fafb;
        ">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span style="font-size: 18px;">👥</span>
            <span style="font-weight: 600; font-size: 15px; color: #111827;">Real-Time Collaboration</span>
          </div>
          <button id="btn-collab-close" style="
            background: transparent;
            border: none;
            font-size: 18px;
            cursor: pointer;
            color: #6b7280;
          ">×</button>
        </div>

        <div style="padding: 20px; display: flex; flex-direction: column; gap: 14px;">
          <div style="
            padding: 8px 12px;
            border-radius: 6px;
            background: ${snap.isConnected ? "#e8f5e9" : "#f3f4f6"};
            border: 1px solid ${snap.isConnected ? "#a5d6a7" : "#e5e7eb"};
            color: ${snap.isConnected ? "#1b5e20" : "#4b5563"};
            font-size: 12px;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 8px;
          ">
            <span style="
              width: 8px;
              height: 8px;
              border-radius: 50%;
              background: ${snap.isConnected ? "#4caf50" : "#9e9e9e"};
            "></span>
            <span>${
              snap.isConnected
                ? `Connected to room: <strong>${snap.activeSession?.session_id}</strong>`
                : "Not connected to a session"
            }</span>
          </div>

          <div>
            <label style="display: block; font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 4px;">
              Session / Room ID
            </label>
            <input id="collab-session-id" type="text" placeholder="e.g. Q4-budget-review" value="${
              snap.activeSession?.session_id || "room-alpha"
            }" style="
              width: 100%;
              padding: 8px 10px;
              border: 1px solid #d1d5db;
              border-radius: 4px;
              font-size: 13px;
              box-sizing: border-box;
            " />
          </div>

          <div>
            <label style="display: block; font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 4px;">
              Your Display Name
            </label>
            <input id="collab-username" type="text" placeholder="Your name" value="${snap.myUsername}" style="
              width: 100%;
              padding: 8px 10px;
              border: 1px solid #d1d5db;
              border-radius: 4px;
              font-size: 13px;
              box-sizing: border-box;
            " />
          </div>

          <div>
            <label style="display: block; font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 4px;">
              Room Key (E2E ChaCha20-Poly1305 Passphrase)
            </label>
            <input id="collab-room-key" type="password" placeholder="Passphrase for end-to-end encryption" value="aether-secret" style="
              width: 100%;
              padding: 8px 10px;
              border: 1px solid #d1d5db;
              border-radius: 4px;
              font-size: 13px;
              box-sizing: border-box;
            " />
          </div>

          <div>
            <label style="display: block; font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 4px;">
              Transport Protocol
            </label>
            <select id="collab-transport-mode" style="
              width: 100%;
              padding: 8px 10px;
              border: 1px solid #d1d5db;
              border-radius: 4px;
              font-size: 13px;
              box-sizing: border-box;
              background: #ffffff;
            ">
              <option value="webrtc" selected>Direct WebRTC DataChannel (LAN P2P)</option>
              <option value="websocket">Fallback WebSocket Relay</option>
            </select>
          </div>

          ${
            snap.isConnected
              ? `
            <div>
              <label style="display: block; font-size: 12px; font-weight: 600; color: #374151; margin-bottom: 6px;">
                Active Peers (${snap.peers.length})
              </label>
              <div id="collab-peer-list" style="
                max-height: 100px;
                overflow-y: auto;
                border: 1px solid #e5e7eb;
                border-radius: 4px;
                padding: 6px;
                display: flex;
                flex-direction: column;
                gap: 4px;
              ">
                ${
                  snap.peers.length === 0
                    ? `<div style="font-size: 12px; color: #9ca3af; padding: 4px;">No other peers in room yet</div>`
                    : snap.peers
                        .map(
                          (p) => `
                    <div style="display: flex; align-items: center; gap: 8px; padding: 4px; border-radius: 4px; background: #f9fafb;">
                      <span style="width: 10px; height: 10px; border-radius: 50%; background: ${p.color_hex};"></span>
                      <span style="font-size: 12px; font-weight: 500; color: #111827;">${p.username}</span>
                      <span style="font-size: 11px; color: #6b7280; margin-left: auto;">${p.active_document || "Active"}</span>
                    </div>
                  `
                        )
                        .join("")
                }
              </div>
            </div>
          `
              : ""
          }
        </div>

        <div style="
          padding: 14px 20px;
          border-top: 1px solid #e5e7eb;
          display: flex;
          align-items: center;
          justify-content: flex-end;
          gap: 10px;
          background: #f9fafb;
        ">
          ${
            snap.isConnected
              ? `
            <button id="btn-collab-leave" style="
              padding: 8px 16px;
              border: 1px solid #ef5350;
              background: #ffebee;
              color: #c62828;
              font-weight: 600;
              font-size: 13px;
              border-radius: 4px;
              cursor: pointer;
            ">Disconnect</button>
          `
              : `
            <button id="btn-collab-join" style="
              padding: 8px 18px;
              border: none;
              background: #1e88e5;
              color: #ffffff;
              font-weight: 600;
              font-size: 13px;
              border-radius: 4px;
              cursor: pointer;
            ">Join Session</button>
          `
          }
        </div>
      </div>
    `;

    // Wire modal event handlers
    const closeBtn = document.getElementById("btn-collab-close");
    if (closeBtn) {
      closeBtn.onclick = () => collabState.setModalOpen(false);
    }

    const joinBtn = document.getElementById("btn-collab-join");
    if (joinBtn) {
      joinBtn.onclick = async () => {
        const sessInput = document.getElementById("collab-session-id") as HTMLInputElement;
        const userInput = document.getElementById("collab-username") as HTMLInputElement;
        const keyInput = document.getElementById("collab-room-key") as HTMLInputElement;
        const modeSelect = document.getElementById("collab-transport-mode") as HTMLSelectElement;

        const sessionId = sessInput?.value || "room-alpha";
        const username = userInput?.value || snap.myUsername;
        const roomKey = keyInput?.value || undefined;
        const mode = modeSelect?.value || "webrtc";

        await collabState.joinSession(sessionId, username, this.currentAppMode, roomKey, mode);
      };
    }

    const leaveBtn = document.getElementById("btn-collab-leave");
    if (leaveBtn) {
      leaveBtn.onclick = async () => {
        await collabState.leaveSession("User requested disconnect");
      };
    }
  }
}

