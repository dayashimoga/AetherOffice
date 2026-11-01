/**
 * DocumentSessionRegistry - Instance-Scoped Multi-Document Session Manager
 *
 * Implements GEC-v7.0 Article I (Single Source of Truth, Zero Global Document Singletons).
 * Manages independent, strongly-typed state instances for concurrent documents
 * across Word, Excel, PowerPoint, PDF, and Notes formats.
 */

import { DocumentType } from "../ipc";
import { WriterState, writerState } from "./writerState";
import { CalcState, calcState } from "./calcState";
import { SlidesState, slidesState } from "./slidesState";
import { PdfState, pdfState } from "./pdfState";
import { NotesState, notesState } from "./notesState";

export type SessionLifecycleState = "active" | "warm" | "cold";

export type SessionStateInstance =
  | { docType: "Word"; state: WriterState }
  | { docType: "Excel"; state: CalcState }
  | { docType: "PowerPoint"; state: SlidesState }
  | { docType: "PDF"; state: PdfState }
  | { docType: "Notes"; state: NotesState };

export interface DocumentSession {
  id: string;
  docType: DocumentType;
  title: string;
  filePath?: string;
  isDirty: boolean;
  lifecycleState: SessionLifecycleState;
  createdAtMs: number;
  lastAccessedMs: number;
  instance: SessionStateInstance;
}

export class DocumentSessionRegistry {
  private sessions: Map<string, DocumentSession> = new Map();
  private activeSessionId: string | null = null;
  private onActiveSessionChanged?: (session: DocumentSession | null) => void;
  private onSessionListChanged?: (sessions: DocumentSession[]) => void;

  constructor() {
    // Seed default baseline sessions mapped to the default exported singletons
    // for seamless backward compatibility with legacy test suites
    this.registerInitialDefaults();
  }

  private registerInitialDefaults(): void {
    // Register the standard default tabs so existing singletons are accessible by session ID
    this.createOrAdoptSession("tab-writer-1", "Word", "Untitled 1.docx", writerState);
    this.createOrAdoptSession("tab-calc-1", "Excel", "Book1.xlsx", calcState);
    this.createOrAdoptSession("tab-slides-1", "PowerPoint", "Presentation 1.pptx", slidesState);
    this.createOrAdoptSession("tab-pdf-1", "PDF", "Document 1.pdf", pdfState);
    this.createOrAdoptSession("tab-notes-1", "Notes", "Untitled 1.txt", notesState);
  }

  public reset(): void {
    this.sessions.clear();
    this.activeSessionId = null;
    this.registerInitialDefaults();
  }

  public clearAll(): void {
    this.sessions.clear();
    this.activeSessionId = null;
  }

  public setCallbacks(callbacks: {
    onActiveSessionChanged?: (session: DocumentSession | null) => void;
    onSessionListChanged?: (sessions: DocumentSession[]) => void;
  }): void {
    this.onActiveSessionChanged = callbacks.onActiveSessionChanged;
    this.onSessionListChanged = callbacks.onSessionListChanged;
  }

  /**
   * Adopts an existing state object into a managed session.
   */
  public createOrAdoptSession(
    id: string,
    docType: DocumentType,
    title: string,
    existingState?: WriterState | CalcState | SlidesState | PdfState | NotesState
  ): DocumentSession {
    if (this.sessions.has(id)) {
      return this.sessions.get(id)!;
    }

    const stateInstance = this.resolveStateInstance(docType, existingState);
    const session: DocumentSession = {
      id,
      docType,
      title,
      isDirty: false,
      lifecycleState: "warm",
      createdAtMs: Date.now(),
      lastAccessedMs: Date.now(),
      instance: stateInstance,
    };

    this.sessions.set(id, session);
    this.onSessionListChanged?.(this.listSessions());
    return session;
  }

  /**
   * Creates a fresh, isolated state instance for a new document tab.
   */
  public createSession(
    id: string,
    docType: DocumentType,
    title?: string,
    filePath?: string
  ): DocumentSession {
    const defaultTitle =
      title ||
      (docType === "Word"
        ? "Untitled.docx"
        : docType === "Excel"
        ? "Book.xlsx"
        : docType === "PowerPoint"
        ? "Presentation.pptx"
        : docType === "PDF"
        ? "Document.pdf"
        : "Notes.txt");

    const stateInstance = this.resolveStateInstance(docType);
    const session: DocumentSession = {
      id,
      docType,
      title: defaultTitle,
      filePath,
      isDirty: false,
      lifecycleState: "warm",
      createdAtMs: Date.now(),
      lastAccessedMs: Date.now(),
      instance: stateInstance,
    };

    this.sessions.set(id, session);
    this.onSessionListChanged?.(this.listSessions());
    return session;
  }

  /**
   * Returns an existing session or creates a new one.
   */
  public getOrCreateSession(
    id: string,
    docType: DocumentType,
    title?: string,
    filePath?: string
  ): DocumentSession {
    const existing = this.sessions.get(id);
    if (existing) {
      existing.lastAccessedMs = Date.now();
      if (title) existing.title = title;
      if (filePath) existing.filePath = filePath;
      return existing;
    }
    const session = this.createSession(id, docType, title, filePath);
    this.syncSingletonToSession(session);
    return session;
  }

  public getSession(id: string): DocumentSession | undefined {
    return this.sessions.get(id);
  }

  public getWriterState(id: string): WriterState | undefined {
    const s = this.sessions.get(id);
    return s?.instance.docType === "Word" ? s.instance.state : undefined;
  }

  public getCalcState(id: string): CalcState | undefined {
    const s = this.sessions.get(id);
    return s?.instance.docType === "Excel" ? s.instance.state : undefined;
  }

  public getSlidesState(id: string): SlidesState | undefined {
    const s = this.sessions.get(id);
    return s?.instance.docType === "PowerPoint" ? s.instance.state : undefined;
  }

  public getPdfState(id: string): PdfState | undefined {
    const s = this.sessions.get(id);
    return s?.instance.docType === "PDF" ? s.instance.state : undefined;
  }

  public getNotesState(id: string): NotesState | undefined {
    const s = this.sessions.get(id);
    return s?.instance.docType === "Notes" ? s.instance.state : undefined;
  }

  public getActiveSession(): DocumentSession | null {
    if (!this.activeSessionId) return null;
    return this.sessions.get(this.activeSessionId) || null;
  }

  public getActiveSessionId(): string | null {
    return this.activeSessionId;
  }

  public setActiveSession(id: string): DocumentSession | null {
    const session = this.sessions.get(id);
    if (!session) return null;

    // Transition previous active to warm
    if (this.activeSessionId && this.activeSessionId !== id) {
      const prev = this.sessions.get(this.activeSessionId);
      if (prev && prev.lifecycleState === "active") {
        prev.lifecycleState = "warm";
      }
    }

    this.activeSessionId = id;
    session.lifecycleState = "active";
    session.lastAccessedMs = Date.now();

    // Synchronize to global singleton objects so any legacy callers or test assertions
    // that check writerState/calcState/etc. observe the active session's data
    this.syncSessionToSingleton(session);

    this.onActiveSessionChanged?.(session);
    return session;
  }

  public closeSession(id: string): boolean {
    const existed = this.sessions.delete(id);
    if (this.activeSessionId === id) {
      const remaining = this.listSessions();
      this.activeSessionId = remaining.length > 0 ? remaining[remaining.length - 1].id : null;
      const newActive = this.getActiveSession();
      if (newActive) {
        newActive.lifecycleState = "active";
        this.syncSessionToSingleton(newActive);
      }
      this.onActiveSessionChanged?.(newActive);
    }
    if (existed) {
      this.onSessionListChanged?.(this.listSessions());
    }
    return existed;
  }

  public listSessions(): DocumentSession[] {
    return Array.from(this.sessions.values());
  }

  public listSessionsByDocType(docType: DocumentType): DocumentSession[] {
    return this.listSessions().filter((s) => s.docType === docType);
  }

  public setDirty(id: string, isDirty: boolean): void {
    const s = this.sessions.get(id);
    if (s) {
      s.isDirty = isDirty;
    }
  }

  public setFilePath(id: string, path: string, title?: string): void {
    const s = this.sessions.get(id);
    if (s) {
      s.filePath = path;
      if (title) s.title = title;
    }
  }

  /**
   * Copies session data into the global singleton reference for full backward compatibility.
   */
  public syncSessionToSingleton(session: DocumentSession): void {
    switch (session.instance.docType) {
      case "Word": {
        if (session.instance.state !== writerState) {
          writerState.doc = session.instance.state.doc;
          writerState.selection = session.instance.state.selection;
          writerState.editingZone = session.instance.state.editingZone;
        }
        break;
      }
      case "Excel": {
        if (session.instance.state !== calcState) {
          calcState.sheets = session.instance.state.sheets;
          calcState.activeSheetId = session.instance.state.activeSheetId;
          calcState.selectedCell = session.instance.state.selectedCell;
          calcState.selectedRanges = session.instance.state.selectedRanges;
          calcState.formulaBarText = session.instance.state.formulaBarText;
        }
        break;
      }
      case "PowerPoint": {
        if (session.instance.state !== slidesState) {
          slidesState.slides = session.instance.state.slides;
          slidesState.currentSlideIndex = session.instance.state.currentSlideIndex;
          slidesState.selectedShapeId = session.instance.state.selectedShapeId;
        }
        break;
      }
      case "PDF": {
        if (session.instance.state !== pdfState) {
          pdfState.pages = session.instance.state.pages;
          pdfState.currentPage = session.instance.state.currentPage;
        }
        break;
      }
      case "Notes": {
        if (session.instance.state !== notesState) {
          notesState.content = session.instance.state.content;
          notesState.title = session.instance.state.title;
        }
        break;
      }
    }
  }

  public syncSingletonToSession(session: DocumentSession): void {
    switch (session.instance.docType) {
      case "Word": {
        session.instance.state.doc = writerState.doc;
        session.instance.state.selection = writerState.selection;
        session.instance.state.editingZone = writerState.editingZone;
        break;
      }
      case "Excel": {
        session.instance.state.sheets = calcState.sheets;
        session.instance.state.activeSheetId = calcState.activeSheetId;
        session.instance.state.selectedCell = calcState.selectedCell;
        session.instance.state.selectedRanges = calcState.selectedRanges;
        session.instance.state.formulaBarText = calcState.formulaBarText;
        break;
      }
      case "PowerPoint": {
        session.instance.state.slides = slidesState.slides;
        session.instance.state.currentSlideIndex = slidesState.currentSlideIndex;
        session.instance.state.selectedShapeId = slidesState.selectedShapeId;
        break;
      }
      case "PDF": {
        session.instance.state.pages = pdfState.pages;
        session.instance.state.currentPage = pdfState.currentPage;
        break;
      }
      case "Notes": {
        session.instance.state.content = notesState.content;
        session.instance.state.title = notesState.title;
        break;
      }
    }
  }

  private resolveStateInstance(
    docType: DocumentType,
    existingState?: WriterState | CalcState | SlidesState | PdfState | NotesState
  ): SessionStateInstance {
    switch (docType) {
      case "Word":
        return {
          docType: "Word",
          state: existingState instanceof WriterState ? existingState : new WriterState(),
        };
      case "Excel":
        return {
          docType: "Excel",
          state: existingState instanceof CalcState ? existingState : new CalcState(),
        };
      case "PowerPoint": {
        const s = existingState instanceof SlidesState ? existingState : new SlidesState();
        if (!existingState) s.resetToBlank();
        return {
          docType: "PowerPoint",
          state: s,
        };
      }
      case "PDF": {
        const s = existingState instanceof PdfState ? existingState : new PdfState();
        if (!existingState) s.resetToBlank();
        return {
          docType: "PDF",
          state: s,
        };
      }
      case "Notes":
        return {
          docType: "Notes",
          state: existingState instanceof NotesState ? existingState : new NotesState(),
        };
    }
  }
}

export const documentSessionRegistry = new DocumentSessionRegistry();
