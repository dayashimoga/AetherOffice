/**
 * WikiLinkCoordinator.ts
 * GEC-v7.0 Stage 50 (Sub-Phase 50.1.2)
 *
 * Presentation Shell Coordinator for:
 * - Bidirectional [[WikiLinks]] & backlinks across Notes, Word documents, and PDFs
 * - Linked references inspection with surrounding context snippets
 * - Unlinked mentions discovery & one-click conversion to wikilinks
 * - Real-time autocomplete suggestions for inline `[[` link typing
 * - Workspace wikilink graph metrics
 */

import {
  IPC,
  type BacklinkReference,
  type UnlinkedMention,
  type WikiLink,
  type WikiLinkIndexStats,
  type WikiLinkSuggestion,
} from "../ipc";

export {
  type BacklinkReference,
  type UnlinkedMention,
  type WikiLink,
  type WikiLinkIndexStats,
  type WikiLinkSuggestion,
};

export interface WikiLinkState {
  activeDocId: string | null;
  activeDocTitle: string | null;
  linkedReferences: BacklinkReference[];
  forwardLinks: WikiLink[];
  unlinkedMentions: UnlinkedMention[];
  suggestions: WikiLinkSuggestion[];
  stats: WikiLinkIndexStats;
  isLoading: boolean;
  error: string | null;
}

export type WikiLinkListener = (state: Readonly<WikiLinkState>) => void;

export class WikiLinkCoordinator {
  private state: WikiLinkState = {
    activeDocId: null,
    activeDocTitle: null,
    linkedReferences: [],
    forwardLinks: [],
    unlinkedMentions: [],
    suggestions: [],
    stats: {
      total_documents: 0,
      total_wikilinks: 0,
      total_unique_targets: 0,
    },
    isLoading: false,
    error: null,
  };

  private listeners: Set<WikiLinkListener> = new Set();

  constructor() {}

  public getState(): Readonly<WikiLinkState> {
    return this.state;
  }

  public subscribe(listener: WikiLinkListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    const currentState = this.getState();
    this.listeners.forEach((listener) => {
      try {
        listener(currentState);
      } catch (err) {
        console.error("Error in WikiLinkCoordinator listener:", err);
      }
    });
  }

  /**
   * Sets the currently active document and loads its backlinks and forward links.
   */
  public async setActiveDocument(docId: string | null, title?: string | null): Promise<void> {
    this.state.activeDocId = docId;
    this.state.activeDocTitle = title ?? docId;

    if (!docId) {
      this.state.linkedReferences = [];
      this.state.forwardLinks = [];
      this.state.unlinkedMentions = [];
      this.notify();
      return;
    }

    await this.refreshActiveDocumentLinks();
  }

  /**
   * Indexes a document's plain text content into the workspace wikilink index.
   */
  public async indexDocument(docId: string, title: string, content: string): Promise<void> {
    this.state.isLoading = true;
    this.state.error = null;
    this.notify();

    try {
      await IPC.wikilinkIndexDocument(docId, title, content);
      await this.refreshStats();

      if (this.state.activeDocId) {
        await this.refreshActiveDocumentLinks();
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    } finally {
      this.state.isLoading = false;
      this.notify();
    }
  }

  /**
   * Removes a document from the workspace wikilink index.
   */
  public async removeDocument(docId: string): Promise<void> {
    this.state.isLoading = true;
    this.state.error = null;
    this.notify();

    try {
      await IPC.wikilinkRemoveDocument(docId);
      await this.refreshStats();

      if (this.state.activeDocId === docId) {
        this.state.linkedReferences = [];
        this.state.forwardLinks = [];
        this.state.unlinkedMentions = [];
      } else if (this.state.activeDocId) {
        await this.refreshActiveDocumentLinks();
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.error = message;
      this.notify();
      throw err;
    } finally {
      this.state.isLoading = false;
      this.notify();
    }
  }

  /**
   * Refreshes backlinks, forward links, and unlinked mentions for the active document.
   */
  public async refreshActiveDocumentLinks(): Promise<void> {
    if (!this.state.activeDocId) return;

    try {
      const targetQuery = this.state.activeDocTitle || this.state.activeDocId;
      const [backlinks, forwardLinks, mentions] = await Promise.all([
        IPC.wikilinkGetBacklinks(targetQuery),
        IPC.wikilinkGetForwardLinks(this.state.activeDocId),
        targetQuery ? IPC.wikilinkFindUnlinkedMentions(targetQuery) : Promise.resolve([]),
      ]);

      this.state.linkedReferences = backlinks;
      this.state.forwardLinks = forwardLinks;
      this.state.unlinkedMentions = mentions;
      this.notify();
    } catch (err) {
      console.warn("Failed to refresh active document links:", err);
    }
  }

  /**
   * Provides autocomplete suggestions when typing `[[`.
   */
  public async querySuggestions(query: string, limit = 10): Promise<WikiLinkSuggestion[]> {
    try {
      const suggestions = await IPC.wikilinkSuggestTargets(query, limit);
      this.state.suggestions = suggestions;
      this.notify();
      return suggestions;
    } catch (err) {
      console.warn("Failed to query wikilink suggestions:", err);
      return [];
    }
  }

  /**
   * Formats a target, anchor, and alias into standard `[[Target#Anchor|Alias]]` syntax.
   */
  public formatWikiLink(targetDoc: string, anchor?: string | null, alias?: string | null): string {
    let link = `[[${targetDoc}`;
    if (anchor && anchor.trim()) {
      link += `#${anchor.trim()}`;
    }
    if (alias && alias.trim()) {
      link += `|${alias.trim()}`;
    }
    link += "]]";
    return link;
  }

  /**
   * Converts an unlinked mention in source text to an explicit `[[WikiLink]]`.
   */
  public convertMentionToWikiLink(sourceContent: string, mention: UnlinkedMention): string {
    const before = sourceContent.substring(0, mention.start_offset);
    const after = sourceContent.substring(mention.end_offset);
    return `${before}[[${mention.matched_text}]]${after}`;
  }

  /**
   * Refreshes overall workspace wikilink index statistics.
   */
  public async refreshStats(): Promise<WikiLinkIndexStats> {
    try {
      const stats = await IPC.wikilinkStats();
      this.state.stats = stats;
      this.notify();
      return stats;
    } catch (err) {
      console.warn("Failed to refresh wikilink stats:", err);
      return this.state.stats;
    }
  }

  public clear(): void {
    this.state = {
      activeDocId: null,
      activeDocTitle: null,
      linkedReferences: [],
      forwardLinks: [],
      unlinkedMentions: [],
      suggestions: [],
      stats: {
        total_documents: 0,
        total_wikilinks: 0,
        total_unique_targets: 0,
      },
      isLoading: false,
      error: null,
    };
    this.notify();
  }
}

export const wikiLinkCoordinator = new WikiLinkCoordinator();

