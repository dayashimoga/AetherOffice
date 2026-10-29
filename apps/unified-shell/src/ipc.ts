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
  app_mode: "hub" | "writer" | "calc" | "slides" | "pdf";
  initial_file?: string | null;
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
    return {} as T;
  }
}

export const IPC = {
  getBootConfig: (): Promise<BootConfig> =>
    invokeTauri("get_boot_config"),
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

  dispatchUiAction: (sessionId: string, action: UiActionEvent): Promise<UiDispatchResult> =>
    invokeTauri("dispatch_ui_action", { session_id: sessionId, action }),
};

