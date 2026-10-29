/**
 * Tauri IPC Client for OpenOfficeRS Unified Shell
 */

export type DocumentType = "Word" | "Excel" | "PowerPoint" | "PDF";

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

// Fallback when running outside Tauri desktop container (e.g. Vite dev preview)
const hasTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (hasTauri) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  } else {
    console.log(`[IPC Mock] invoke('${cmd}')`, args);
    // Mock response for browser preview
    if (cmd === "create_session") {
      const type = (args?.doc_type as DocumentType) || "Word";
      return {
        session_id: `sess-${Date.now()}`,
        doc_type: type,
        title: (args?.title as string) || `New ${type}`,
        is_dirty: false,
      } as unknown as T;
    }
    return {} as T;
  }
}

export const IPC = {
  createSession: (docType: DocumentType, title: string): Promise<SessionSummary> =>
    invokeTauri("create_session", { doc_type: docType, title }),

  openDocument: (filePath: string): Promise<SessionSummary> =>
    invokeTauri("open_document", { file_path: filePath }),

  saveDocument: (sessionId: string, filePath?: string): Promise<void> =>
    invokeTauri("save_document", { session_id: sessionId, file_path: filePath }),

  closeSession: (sessionId: string, force: boolean): Promise<string | null> =>
    invokeTauri("close_session", { session_id: sessionId, force }),

  executeUndo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_undo", { session_id: sessionId }),

  executeRedo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_redo", { session_id: sessionId }),

  exportSvg: (sessionId: string): Promise<string> =>
    invokeTauri("export_svg", { session_id: sessionId }),
};

