/**
 * WordRevisionCoordinator.ts
 *
 * Implements Sub-Phase 46.1.1: Revision Tracking & Redline Renderer presentation layer.
 * Complies strictly with GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - SimpleMarkup, AllMarkup, NoMarkup, Original modes.
 * - Deterministic author color palette matching Rust core.
 * - Batch and individual accept/reject with zero data loss.
 */

import type { WordDocumentModel, DocPage, WordParagraph, WordRun } from "./writerState";

export type RevisionType = "Insertion" | "Deletion" | "RunPropertiesChange" | "ParagraphPropertiesChange";
export type RevisionViewMode = "SimpleMarkup" | "AllMarkup" | "NoMarkup" | "Original";

export interface RevisionItem {
  id: string;
  author: string;
  date: string;
  revisionType: RevisionType;
  textPreview: string;
  pageIndex: number;
  paragraphIndex: number;
}

export class AuthorColorPalette {
  public static readonly PALETTE: readonly string[] = [
    "#1E88E5", // Blue
    "#D81B60", // Magenta
    "#004D40", // Deep Teal
    "#F4511E", // Orange
    "#3949AB", // Indigo
    "#8E24AA", // Purple
    "#00ACC1", // Cyan
    "#43A047", // Green
  ];

  public static getColorForAuthor(author: string): string {
    if (!author) {
      return AuthorColorPalette.PALETTE[0];
    }
    let hash = 0;
    for (let i = 0; i < author.length; i++) {
      hash = (hash + author.charCodeAt(i)) & 0xffffffff;
    }
    return AuthorColorPalette.PALETTE[Math.abs(hash) % AuthorColorPalette.PALETTE.length];
  }
}

export class WordRevisionCoordinator {
  public trackChanges = false;
  public viewMode: RevisionViewMode = "AllMarkup";
  public activeAuthor = "Author";
  public selectedRevisionId: string | null = null;

  constructor(options?: { trackChanges?: boolean; viewMode?: RevisionViewMode; activeAuthor?: string }) {
    if (options?.trackChanges !== undefined) this.trackChanges = options.trackChanges;
    if (options?.viewMode !== undefined) this.viewMode = options.viewMode;
    if (options?.activeAuthor !== undefined) this.activeAuthor = options.activeAuthor;
  }

  public toggleTrackChanges(): boolean {
    this.trackChanges = !this.trackChanges;
    return this.trackChanges;
  }

  public setViewMode(mode: RevisionViewMode): void {
    this.viewMode = mode;
  }

  public setActiveAuthor(author: string): void {
    if (author && author.trim()) {
      this.activeAuthor = author.trim();
    }
  }

  public selectRevision(id: string | null): void {
    this.selectedRevisionId = id;
  }

  /**
   * Scans the document and collects all tracked revisions across pages and paragraphs.
   */
  public listRevisions(doc: WordDocumentModel): RevisionItem[] {
    const revisions: RevisionItem[] = [];

    doc.pages.forEach((page: DocPage, pageIdx: number) => {
      page.paragraphs.forEach((p: WordParagraph, pIdx: number) => {
        p.runs.forEach((run: WordRun, rIdx: number) => {
          if (run.insertion) {
            const revId = `ins_${pageIdx}_${pIdx}_${rIdx}`;
            revisions.push({
              id: revId,
              author: this.activeAuthor,
              date: new Date().toISOString(),
              revisionType: "Insertion",
              textPreview: run.text,
              pageIndex: pageIdx,
              paragraphIndex: pIdx,
            });
          } else if (run.deletion) {
            const revId = `del_${pageIdx}_${pIdx}_${rIdx}`;
            revisions.push({
              id: revId,
              author: this.activeAuthor,
              date: new Date().toISOString(),
              revisionType: "Deletion",
              textPreview: run.text,
              pageIndex: pageIdx,
              paragraphIndex: pIdx,
            });
          }
        });
      });
    });

    return revisions;
  }

  /**
   * Accepts a single tracked revision by ID.
   */
  public acceptRevision(doc: WordDocumentModel, revisionId: string): boolean {
    let modified = false;
    doc.pages.forEach((page: DocPage, pageIdx: number) => {
      page.paragraphs.forEach((p: WordParagraph, pIdx: number) => {
        const nextRuns: WordRun[] = [];
        p.runs.forEach((run: WordRun, rIdx: number) => {
          const currentInsId = `ins_${pageIdx}_${pIdx}_${rIdx}`;
          const currentDelId = `del_${pageIdx}_${pIdx}_${rIdx}`;

          if (currentInsId === revisionId && run.insertion) {
            // Accept insertion: normalize run as standard content
            const normalized: WordRun = {
              ...run,
              insertion: false,
              underline: false,
              color: "#242424",
            };
            nextRuns.push(normalized);
            modified = true;
          } else if (currentDelId === revisionId && run.deletion) {
            // Accept deletion: completely drop run
            modified = true;
          } else {
            nextRuns.push(run);
          }
        });
        p.runs = nextRuns;
      });
    });
    return modified;
  }

  /**
   * Rejects a single tracked revision by ID.
   */
  public rejectRevision(doc: WordDocumentModel, revisionId: string): boolean {
    let modified = false;
    doc.pages.forEach((page: DocPage, pageIdx: number) => {
      page.paragraphs.forEach((p: WordParagraph, pIdx: number) => {
        const nextRuns: WordRun[] = [];
        p.runs.forEach((run: WordRun, rIdx: number) => {
          const currentInsId = `ins_${pageIdx}_${pIdx}_${rIdx}`;
          const currentDelId = `del_${pageIdx}_${pIdx}_${rIdx}`;

          if (currentInsId === revisionId && run.insertion) {
            // Reject insertion: drop run entirely
            modified = true;
          } else if (currentDelId === revisionId && run.deletion) {
            // Reject deletion: restore as regular run
            const restored: WordRun = {
              ...run,
              deletion: false,
              strikethrough: false,
              color: "#242424",
            };
            nextRuns.push(restored);
            modified = true;
          } else {
            nextRuns.push(run);
          }
        });
        p.runs = nextRuns;
      });
    });
    return modified;
  }

  /**
   * Accepts all tracked revisions across the document in a single pass.
   */
  public acceptAllRevisions(doc: WordDocumentModel): number {
    const total = this.listRevisions(doc).length;
    doc.pages.forEach((page: DocPage) => {
      page.paragraphs.forEach((p: WordParagraph) => {
        p.runs = p.runs
          .filter((r: WordRun) => !r.deletion)
          .map((r: WordRun) => {
            if (r.insertion) {
              return {
                ...r,
                insertion: false,
                underline: false,
                color: "#242424",
              };
            }
            return r;
          });
      });
    });
    return total;
  }

  /**
   * Rejects all tracked revisions across the document in a single pass.
   */
  public rejectAllRevisions(doc: WordDocumentModel): number {
    const total = this.listRevisions(doc).length;
    doc.pages.forEach((page: DocPage) => {
      page.paragraphs.forEach((p: WordParagraph) => {
        p.runs = p.runs
          .filter((r: WordRun) => !r.insertion)
          .map((r: WordRun) => {
            if (r.deletion) {
              return {
                ...r,
                deletion: false,
                strikethrough: false,
                color: "#242424",
              };
            }
            return r;
          });
      });
    });
    return total;
  }

  /**
   * Filters runs for display according to active RevisionViewMode.
   */
  public filterRunsForViewMode(runs: WordRun[], mode: RevisionViewMode = this.viewMode): WordRun[] {
    switch (mode) {
      case "AllMarkup":
        return runs.map((r) => {
          if (r.insertion) {
            return {
              ...r,
              underline: true,
              color: AuthorColorPalette.getColorForAuthor(this.activeAuthor),
            };
          }
          if (r.deletion) {
            return {
              ...r,
              strikethrough: true,
              color: AuthorColorPalette.getColorForAuthor(this.activeAuthor),
            };
          }
          return r;
        });

      case "SimpleMarkup":
        return runs
          .filter((r) => !r.deletion)
          .map((r) => {
            if (r.insertion) {
              return { ...r, underline: false };
            }
            return r;
          });

      case "NoMarkup":
        return runs
          .filter((r) => !r.deletion)
          .map((r) => {
            if (r.insertion) {
              return { ...r, underline: false, color: "#242424" };
            }
            return r;
          });

      case "Original":
        return runs
          .filter((r) => !r.insertion)
          .map((r) => {
            if (r.deletion) {
              return { ...r, strikethrough: false, color: "#242424" };
            }
            return r;
          });
    }
  }

  /**
   * Returns revision summary statistics.
   */
  public getSummary(doc: WordDocumentModel): {
    insertions: number;
    deletions: number;
    total: number;
  } {
    const revs = this.listRevisions(doc);
    let ins = 0;
    let del = 0;
    for (const r of revs) {
      if (r.revisionType === "Insertion") ins++;
      if (r.revisionType === "Deletion") del++;
    }
    return {
      insertions: ins,
      deletions: del,
      total: revs.length,
    };
  }
}
