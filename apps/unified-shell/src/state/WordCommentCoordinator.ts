/**
 * WordCommentCoordinator.ts
 *
 * Implements Sub-Phase 46.1.2: Anchored Threaded Comments & Resolved History presentation coordinator.
 * Strictly conforms to GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - Threaded comment cascades with parent-child relationships.
 * - Resolution state tracking and filtering ('all', 'unresolved', 'resolved').
 * - Anchored selection links and reactive subscriber notifications.
 */

export interface CommentItem {
  id: string;
  author: string;
  initials?: string;
  date?: string;
  text: string;
  parentId?: string;
  isResolved: boolean;
}

export interface CommentThread {
  root: CommentItem;
  replies: CommentItem[];
  anchorText?: string;
  paragraphIndex?: number;
  isResolved: boolean;
}

export interface CommentSummary {
  total: number;
  unresolved: number;
  resolved: number;
}

export type CommentFilterMode = "all" | "unresolved" | "resolved";

export class WordCommentCoordinator {
  private threads: CommentThread[] = [];
  public activeThreadId: string | null = null;
  public filterMode: CommentFilterMode = "all";
  public currentAuthor = "Reviewer";
  public currentInitials = "REV";
  private listeners: Set<(threads: CommentThread[]) => void> = new Set();

  constructor(options?: {
    currentAuthor?: string;
    currentInitials?: string;
    filterMode?: CommentFilterMode;
  }) {
    if (options?.currentAuthor !== undefined) this.currentAuthor = options.currentAuthor;
    if (options?.currentInitials !== undefined) this.currentInitials = options.currentInitials;
    if (options?.filterMode !== undefined) this.filterMode = options.filterMode;
  }

  public loadThreads(threads: CommentThread[]): void {
    this.threads = threads.map((t) => ({
      ...t,
      root: { ...t.root },
      replies: t.replies.map((r) => ({ ...r })),
    }));
    this.notify();
  }

  public getThreads(): CommentThread[] {
    return this.threads;
  }

  public getVisibleThreads(): CommentThread[] {
    switch (this.filterMode) {
      case "resolved":
        return this.threads.filter((t) => t.isResolved);
      case "unresolved":
        return this.threads.filter((t) => !t.isResolved);
      case "all":
      default:
        return this.threads;
    }
  }

  public getThread(id: string): CommentThread | undefined {
    return this.threads.find((t) => t.root.id === id);
  }

  public selectThread(threadId: string | null): void {
    this.activeThreadId = threadId;
  }

  public setFilterMode(mode: CommentFilterMode): void {
    this.filterMode = mode;
  }

  public setCurrentUser(author: string, initials?: string): void {
    if (author && author.trim()) {
      this.currentAuthor = author.trim();
    }
    if (initials !== undefined) {
      this.currentInitials = initials.trim();
    }
  }

  public addComment(
    anchorText: string,
    paragraphIndex: number,
    text: string,
    author?: string,
    initials?: string,
  ): CommentThread {
    const commentId = this.nextId();
    const resolvedAuthor = author || this.currentAuthor;
    const resolvedInitials = initials !== undefined ? initials : this.currentInitials;

    const root: CommentItem = {
      id: commentId,
      author: resolvedAuthor,
      initials: resolvedInitials,
      date: new Date().toISOString(),
      text,
      parentId: undefined,
      isResolved: false,
    };

    const thread: CommentThread = {
      root,
      replies: [],
      anchorText,
      paragraphIndex,
      isResolved: false,
    };

    this.threads.push(thread);
    this.activeThreadId = commentId;
    this.notify();
    return thread;
  }

  public addReply(
    parentThreadId: string,
    text: string,
    author?: string,
    initials?: string,
  ): CommentItem | null {
    const thread = this.getThread(parentThreadId);
    if (!thread) {
      return null;
    }

    const replyId = this.nextId();
    const resolvedAuthor = author || this.currentAuthor;
    const resolvedInitials = initials !== undefined ? initials : this.currentInitials;

    const reply: CommentItem = {
      id: replyId,
      author: resolvedAuthor,
      initials: resolvedInitials,
      date: new Date().toISOString(),
      text,
      parentId: parentThreadId,
      isResolved: false,
    };

    thread.replies.push(reply);
    // If thread was resolved, adding a reply reopens it
    if (thread.isResolved) {
      thread.isResolved = false;
      thread.root.isResolved = false;
    }
    this.notify();
    return reply;
  }

  public toggleResolveThread(threadId: string): boolean {
    const thread = this.getThread(threadId);
    if (!thread) return false;

    const targetState = !thread.isResolved;
    thread.isResolved = targetState;
    thread.root.isResolved = targetState;
    for (const r of thread.replies) {
      r.isResolved = targetState;
    }

    this.notify();
    return true;
  }

  public setThreadResolved(threadId: string, resolved: boolean): boolean {
    const thread = this.getThread(threadId);
    if (!thread) return false;

    thread.isResolved = resolved;
    thread.root.isResolved = resolved;
    for (const r of thread.replies) {
      r.isResolved = resolved;
    }

    this.notify();
    return true;
  }

  public deleteThread(threadId: string): boolean {
    const initialLen = this.threads.length;
    this.threads = this.threads.filter((t) => t.root.id !== threadId);
    if (this.activeThreadId === threadId) {
      this.activeThreadId = null;
    }
    const modified = this.threads.length < initialLen;
    if (modified) {
      this.notify();
    }
    return modified;
  }

  public deleteReply(threadId: string, replyId: string): boolean {
    const thread = this.getThread(threadId);
    if (!thread) return false;

    const initialLen = thread.replies.length;
    thread.replies = thread.replies.filter((r) => r.id !== replyId);
    const modified = thread.replies.length < initialLen;
    if (modified) {
      this.notify();
    }
    return modified;
  }

  public getSummary(): CommentSummary {
    let unresolved = 0;
    let resolved = 0;

    for (const thread of this.threads) {
      if (thread.isResolved) {
        resolved += 1;
      } else {
        unresolved += 1;
      }
    }

    return {
      total: this.threads.length,
      unresolved,
      resolved,
    };
  }

  public subscribe(listener: (threads: CommentThread[]) => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    const current = this.threads;
    for (const listener of this.listeners) {
      try {
        listener(current);
      } catch {
        // Safe dispatch: errors in listeners do not break state
      }
    }
  }

  private nextId(): string {
    let maxId = 0;
    for (const t of this.threads) {
      const rootNum = parseInt(t.root.id, 10);
      if (!Number.isNaN(rootNum)) {
        maxId = Math.max(maxId, rootNum);
      }
      for (const r of t.replies) {
        const replyNum = parseInt(r.id, 10);
        if (!Number.isNaN(replyNum)) {
          maxId = Math.max(maxId, replyNum);
        }
      }
    }
    return (maxId + 1).toString();
  }
}

