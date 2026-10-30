/**
 * Tauri IPC Client for OpenOfficeRS Unified Shell
 */

export type DocumentType = "Word" | "Excel" | "PowerPoint" | "PDF" | "Notes";

export interface SessionSummary {
  session_id: string;
  doc_type: DocumentType;
  title: string;
  is_dirty: boolean;
  file_path?: string;
}

export interface FormattingCommand {
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  font_family?: string;
  font_size_pt?: number;
}

export interface UiDispatchResult {
  session_id: string;
  description: string;
  undo_stack_depth: number;
  redo_stack_depth: number;
  is_dirty: boolean;
}

export type UiActionEvent =
  | { type: "ToggleBold" }
  | { type: "ToggleItalic" }
  | { type: "ToggleUnderline" }
  | { type: "ToggleStrikethrough" }
  | { type: "SetFontFamily"; payload: string }
  | { type: "SetFontSize"; payload: number }
  | { type: "InsertText"; payload: { text: string } }
  | { type: "DeleteTextRange"; payload: { offset: number; length: number } }
  | { type: "SetCellFormula"; payload: { coord: string; formula: string } }
  | { type: "SetCellValue"; payload: { coord: string; value: string } }
  | { type: "AddSheet"; payload: { title: string } }
  | { type: "EditRowCardField"; payload: { row: number; col: number; field_name: string; new_value: string } }
  | { type: "NewSlide"; payload: { layout: string } }
  | { type: "SetFormField"; payload: { field_name: string; value: string } }
  | { type: "RotatePdfPage"; payload: { page: number; angle_deg: number } }
  | { type: "ClearAllInk" }
  | { type: "Cut" }
  | { type: "Copy" }
  | { type: "Paste"; payload: { content: string } }
  | { type: "Undo" }
  | { type: "Redo" }
  | { type: "MarkSaved" };

export interface BootConfig {
  app_mode: "hub" | "writer" | "calc" | "slides" | "pdf" | "notes";
  initial_file?: string | null;
}

export interface AppInstance {
  instance_id: string;
  app_mode: "hub" | "writer" | "calc" | "slides" | "pdf" | "notes";
  title: string;
  file_path?: string | null;
  process_id?: number | null;
  status: string;
  launched_at_ms: number;
}

export interface DocumentMetadata {
  session_id: string;
  title: string;
  doc_type: DocumentType;
  file_path?: string | null;
  is_dirty: boolean;
  file_size_bytes: number;
  last_modified_str: string;
}

export interface CursorCoord {
  text_offset?: number | null;
  grid_row?: number | null;
  grid_col?: number | null;
}

export interface SelectionSpan {
  text_start?: number | null;
  text_end?: number | null;
  grid_start_row?: number | null;
  grid_start_col?: number | null;
  grid_end_row?: number | null;
  grid_end_col?: number | null;
}

export interface PeerPresence {
  client_id: number;
  username: string;
  color_hex: string;
  active_document: string;
  active_tab: string;
  cursor?: CursorCoord | null;
  selection?: SelectionSpan | null;
  last_seen_ms: number;
}

export interface CollabSessionInfo {
  session_id: string;
  client_id: number;
  username: string;
  app_mode: string;
  status: string;
  transport_mode: string;
  peers: PeerPresence[];
}

// Fallback when running outside Tauri desktop container (e.g. Vite dev preview / unit tests)
const hasTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
const mockInstances: AppInstance[] = [];
let mockCollabSession: CollabSessionInfo | null = null;

async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (hasTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  } else {
    console.log(`[IPC Mock] invoke('${cmd}')`, args);
    // Mock response for browser preview
    if (cmd === "get_boot_config") {
      const params = typeof window !== "undefined" ? new URLSearchParams(window.location.search) : null;
      const mode = (params?.get("mode") as "hub" | "writer" | "calc" | "slides" | "pdf") || "writer";
      return {
        app_mode: mode,
        initial_file: params?.get("file") || null,
      } as unknown as T;
    }
    if (cmd === "create_session") {
      const type = (args?.doc_type as DocumentType) || "Word";
      return {
        session_id: `sess-${Date.now()}`,
        doc_type: type,
        title: (args?.title as string) || `New ${type}`,
        is_dirty: false,
      } as unknown as T;
    }
    if (cmd === "launch_standalone_app") {
      const mode = (args?.app_mode as "hub" | "writer" | "calc" | "slides" | "pdf") || "writer";
      const file = args?.file_path as string | undefined;
      const appName =
        mode === "writer"
          ? "AetherWriter"
          : mode === "calc"
          ? "AetherCalc"
          : mode === "slides"
          ? "AetherSlides"
          : mode === "pdf"
          ? "AetherPDF"
          : "AetherHub";
      const title = file
        ? `${appName} - ${file.split(/[/\\]/).pop()}`
        : `${appName} (Instance ${mockInstances.length + 1})`;
      const inst: AppInstance = {
        instance_id: `inst-${mode}-${Date.now()}-${mockInstances.length + 1}`,
        app_mode: mode,
        title,
        file_path: file || null,
        process_id: 10000 + mockInstances.length + 1,
        status: "running",
        launched_at_ms: Date.now(),
      };
      mockInstances.push(inst);
      return inst as unknown as T;
    }
    if (cmd === "list_app_instances") {
      return [...mockInstances] as unknown as T;
    }
    if (cmd === "close_app_instance") {
      const id = args?.instance_id as string;
      const idx = mockInstances.findIndex((i) => i.instance_id === id);
      if (idx >= 0) {
        mockInstances.splice(idx, 1);
        return true as unknown as T;
      }
      return false as unknown as T;
    }
    if (cmd === "focus_app_instance") {
      const id = args?.instance_id as string;
      return mockInstances.some((i) => i.instance_id === id) as unknown as T;
    }
    if (cmd === "open_document") {
      const path = (args?.file_path as string) || "Document.docx";
      const filename = path.split(/[/\\]/).pop() || path;
      const lower = path.toLowerCase();
      const docType: DocumentType =
        lower.endsWith(".xlsx") || lower.endsWith(".xls") || lower.endsWith(".csv")
          ? "Excel"
          : lower.endsWith(".pptx") || lower.endsWith(".ppt")
          ? "PowerPoint"
          : lower.endsWith(".pdf")
          ? "PDF"
          : "Word";
      return {
        session_id: `sess-${Date.now()}`,
        doc_type: docType,
        title: filename,
        is_dirty: false,
        file_path: path,
      } as unknown as T;
    }
    if (cmd === "save_document") {
      const path = (args?.file_path as string) || "Saved_Document.docx";
      return path as unknown as T;
    }
    if (cmd === "export_pdf") {
      const path = (args?.output_path as string) || "Exported_Document.pdf";
      return path as unknown as T;
    }
    if (cmd === "get_document_info") {
      const sessId = (args?.session_id as string) || "sess-1";
      return {
        session_id: sessId,
        title: "Active_Document.docx",
        doc_type: "Word",
        file_path: "/workspace/Active_Document.docx",
        is_dirty: false,
        file_size_bytes: 14200,
        last_modified_str: "Just now",
      } as unknown as T;
    }
    if (cmd === "collab_join_session") {
      const sessId = (args?.session_id as string) || "collab-room-1";
      const user = (args?.username as string) || "LocalUser";
      const mode = (args?.app_mode as string) || "writer";
      const tmode = (args?.transport_mode as string) || "webrtc";
      mockCollabSession = {
        session_id: sessId,
        client_id: 101,
        username: user,
        app_mode: mode,
        status: "Connected",
        transport_mode: tmode === "websocket" ? "WebSocketRelay" : "WebRtcDataChannel",
        peers: [],
      };
      return mockCollabSession as unknown as T;
    }
    if (cmd === "collab_leave_session") {
      mockCollabSession = null;
      return true as unknown as T;
    }
    if (cmd === "collab_broadcast_delta") {
      return true as unknown as T;
    }
    if (cmd === "collab_get_peers") {
      return (mockCollabSession?.peers || []) as unknown as T;
    }
    return {} as T;
  }
}

export const IPC = {
  _resetMockState: (): void => {
    mockInstances.length = 0;
    mockCollabSession = null;
  },
  getBootConfig: (): Promise<BootConfig> =>
    invokeTauri("get_boot_config"),
  launchStandaloneApp: (appMode: string, filePath?: string): Promise<AppInstance> =>
    invokeTauri("launch_standalone_app", { app_mode: appMode, file_path: filePath }),
  listAppInstances: (): Promise<AppInstance[]> =>
    invokeTauri("list_app_instances"),
  closeAppInstance: (instanceId: string): Promise<boolean> =>
    invokeTauri("close_app_instance", { instance_id: instanceId }),
  focusAppInstance: (instanceId: string): Promise<boolean> =>
    invokeTauri("focus_app_instance", { instance_id: instanceId }),
  createSession: (docType: DocumentType, title: string): Promise<SessionSummary> =>
    invokeTauri("create_session", { doc_type: docType, title }),

  openDocument: (filePath: string): Promise<SessionSummary> =>
    invokeTauri("open_document", { file_path: filePath }),

  saveDocument: (sessionId: string, filePath?: string): Promise<string> =>
    invokeTauri("save_document", { session_id: sessionId, file_path: filePath }),

  exportPdf: (sessionId: string, outputPath: string): Promise<string> =>
    invokeTauri("export_pdf", { session_id: sessionId, output_path: outputPath }),

  getDocumentInfo: (sessionId: string): Promise<DocumentMetadata> =>
    invokeTauri("get_document_info", { session_id: sessionId }),

  closeSession: (sessionId: string, force: boolean): Promise<string | null> =>
    invokeTauri("close_session", { session_id: sessionId, force }),

  executeUndo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_undo", { session_id: sessionId }),

  executeRedo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_redo", { session_id: sessionId }),

  exportSvg: (sessionId: string): Promise<string> =>
    invokeTauri("export_svg", { session_id: sessionId }),

  dispatchUiAction: (sessionId: string, action: UiActionEvent): Promise<UiDispatchResult> =>
    invokeTauri("dispatch_ui_action", { session_id: sessionId, action }),

  collabJoinSession: (
    sessionId: string,
    username: string,
    appMode: string,
    roomKey?: string,
    transportMode?: string
  ): Promise<CollabSessionInfo> =>
    invokeTauri("collab_join_session", {
      session_id: sessionId,
      username,
      app_mode: appMode,
      room_key: roomKey,
      transport_mode: transportMode,
    }),

  collabLeaveSession: (sessionId: string, reason?: string): Promise<boolean> =>
    invokeTauri("collab_leave_session", { session_id: sessionId, reason }),

  collabBroadcastDelta: (
    sessionId: string,
    deltaType: string,
    deltaJson: string
  ): Promise<boolean> =>
    invokeTauri("collab_broadcast_delta", {
      session_id: sessionId,
      delta_type: deltaType,
      delta_json: deltaJson,
    }),

  collabGetPeers: (sessionId: string): Promise<PeerPresence[]> =>
    invokeTauri("collab_get_peers", { session_id: sessionId }),
};

