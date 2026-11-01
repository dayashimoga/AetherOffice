import { describe, it, expect, vi } from "vitest";
import {
  WordCommentCoordinator,
  type CommentThread,
} from "../state/WordCommentCoordinator";

describe("WordCommentCoordinator", () => {
  it("should initialize with default settings and empty threads", () => {
    const coordinator = new WordCommentCoordinator();
    expect(coordinator.getThreads()).toHaveLength(0);
    expect(coordinator.activeThreadId).toBeNull();
    expect(coordinator.filterMode).toBe("all");
    expect(coordinator.currentAuthor).toBe("Reviewer");
    expect(coordinator.currentInitials).toBe("REV");
  });

  it("should load existing threads and query by id", () => {
    const coordinator = new WordCommentCoordinator();
    const mockThreads: CommentThread[] = [
      {
        root: {
          id: "1",
          author: "Alice",
          initials: "A",
          text: "Review this clause",
          isResolved: false,
        },
        replies: [
          {
            id: "2",
            author: "Bob",
            initials: "B",
            text: "Checked and approved",
            parentId: "1",
            isResolved: false,
          },
        ],
        anchorText: "Clause 4.1",
        paragraphIndex: 0,
        isResolved: false,
      },
    ];

    coordinator.loadThreads(mockThreads);
    expect(coordinator.getThreads()).toHaveLength(1);

    const thread = coordinator.getThread("1");
    expect(thread).toBeDefined();
    expect(thread?.root.author).toBe("Alice");
    expect(thread?.replies).toHaveLength(1);
    expect(thread?.replies[0].author).toBe("Bob");
  });

  it("should filter threads by 'all', 'unresolved', and 'resolved'", () => {
    const coordinator = new WordCommentCoordinator();
    const threads: CommentThread[] = [
      {
        root: { id: "1", author: "Alice", text: "Open issue", isResolved: false },
        replies: [],
        anchorText: "Section 1",
        paragraphIndex: 0,
        isResolved: false,
      },
      {
        root: { id: "2", author: "Bob", text: "Fixed issue", isResolved: true },
        replies: [],
        anchorText: "Section 2",
        paragraphIndex: 1,
        isResolved: true,
      },
    ];

    coordinator.loadThreads(threads);

    coordinator.setFilterMode("all");
    expect(coordinator.getVisibleThreads()).toHaveLength(2);

    coordinator.setFilterMode("unresolved");
    const unresolved = coordinator.getVisibleThreads();
    expect(unresolved).toHaveLength(1);
    expect(unresolved[0].root.id).toBe("1");

    coordinator.setFilterMode("resolved");
    const resolved = coordinator.getVisibleThreads();
    expect(resolved).toHaveLength(1);
    expect(resolved[0].root.id).toBe("2");
  });

  it("should add new anchored comments with sequential id and auto-selection", () => {
    const coordinator = new WordCommentCoordinator({
      currentAuthor: "Lead Architect",
      currentInitials: "LA",
    });

    const thread1 = coordinator.addComment("Budget table", 2, "Please update Q4 projections.");
    expect(thread1.root.id).toBe("1");
    expect(thread1.root.author).toBe("Lead Architect");
    expect(thread1.root.initials).toBe("LA");
    expect(thread1.anchorText).toBe("Budget table");
    expect(thread1.paragraphIndex).toBe(2);
    expect(coordinator.activeThreadId).toBe("1");

    const thread2 = coordinator.addComment("Timeline chart", 4, "Milestone dates shifted.");
    expect(thread2.root.id).toBe("2");
    expect(coordinator.activeThreadId).toBe("2");
    expect(coordinator.getThreads()).toHaveLength(2);
  });

  it("should add threaded replies and link to parent thread", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("Executive Summary", 0, "Needs executive sponsor approval.");

    const reply = coordinator.addReply("1", "Sponsor approved via email yesterday.", "VP Ops", "VPO");
    expect(reply).not.toBeNull();
    expect(reply?.id).toBe("2");
    expect(reply?.author).toBe("VP Ops");
    expect(reply?.initials).toBe("VPO");
    expect(reply?.parentId).toBe("1");

    const thread = coordinator.getThread("1");
    expect(thread?.replies).toHaveLength(1);
    expect(thread?.replies[0].id).toBe("2");

    // Invalid parent id
    const invalidReply = coordinator.addReply("999", "Ghost reply");
    expect(invalidReply).toBeNull();
  });

  it("should toggle resolution and update resolved flags across thread and replies", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("Intro", 0, "Typo on line 2");
    coordinator.addReply("1", "Fixed in draft 2");

    // Resolve thread
    const res = coordinator.toggleResolveThread("1");
    expect(res).toBe(true);

    const thread = coordinator.getThread("1");
    expect(thread?.isResolved).toBe(true);
    expect(thread?.root.isResolved).toBe(true);
    expect(thread?.replies[0].isResolved).toBe(true);

    // Reopen thread
    const resReopen = coordinator.toggleResolveThread("1");
    expect(resReopen).toBe(true);
    expect(thread?.isResolved).toBe(false);
    expect(thread?.root.isResolved).toBe(false);
    expect(thread?.replies[0].isResolved).toBe(false);
  });

  it("should reopen thread when a new reply is added to a resolved thread", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("Conclusion", 5, "Check references");
    coordinator.setThreadResolved("1", true);

    expect(coordinator.getThread("1")?.isResolved).toBe(true);

    // Adding reply reopens thread
    coordinator.addReply("1", "Actually reference #4 is still missing.");
    const thread = coordinator.getThread("1");
    expect(thread?.isResolved).toBe(false);
    expect(thread?.root.isResolved).toBe(false);
    expect(thread?.replies).toHaveLength(1);
  });

  it("should delete individual replies without removing root comment or thread", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("Risk Matrix", 3, "High severity risk unaddressed");
    coordinator.addReply("1", "Reply 1");
    coordinator.addReply("1", "Reply 2");

    const thread = coordinator.getThread("1");
    expect(thread?.replies).toHaveLength(2);

    const deleted = coordinator.deleteReply("1", "2");
    expect(deleted).toBe(true);
    expect(thread?.replies).toHaveLength(1);
    expect(thread?.replies[0].id).toBe("3");
    expect(coordinator.getThreads()).toHaveLength(1);

    // Delete non-existent reply
    expect(coordinator.deleteReply("1", "999")).toBe(false);
  });

  it("should delete entire thread including root and replies and clear active selection", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("Disclaimer", 8, "Legal disclaimer standard text");
    coordinator.addReply("1", "Confirmed standard");
    coordinator.selectThread("1");
    expect(coordinator.activeThreadId).toBe("1");

    const deleted = coordinator.deleteThread("1");
    expect(deleted).toBe(true);
    expect(coordinator.getThreads()).toHaveLength(0);
    expect(coordinator.activeThreadId).toBeNull();

    // Delete non-existent thread
    expect(coordinator.deleteThread("999")).toBe(false);
  });

  it("should compute accurate summary statistics for total, unresolved, and resolved", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.addComment("P1", 0, "C1"); // unresolved
    coordinator.addComment("P2", 1, "C2"); // unresolved
    coordinator.addComment("P3", 2, "C3"); // will be resolved
    coordinator.addReply("1", "R1"); // reply shouldn't increase root total

    coordinator.setThreadResolved("3", true);

    const summary = coordinator.getSummary();
    expect(summary.total).toBe(3);
    expect(summary.unresolved).toBe(2);
    expect(summary.resolved).toBe(1);
  });

  it("should notify subscribers on state modifications and unsubscribe cleanly", () => {
    const coordinator = new WordCommentCoordinator();
    const listener = vi.fn();
    const unsubscribe = coordinator.subscribe(listener);

    coordinator.addComment("Anchor", 0, "Comment text");
    expect(listener).toHaveBeenCalledTimes(1);

    coordinator.addReply("1", "Reply text");
    expect(listener).toHaveBeenCalledTimes(2);

    coordinator.toggleResolveThread("1");
    expect(listener).toHaveBeenCalledTimes(3);

    coordinator.deleteThread("1");
    expect(listener).toHaveBeenCalledTimes(4);

    unsubscribe();
    coordinator.addComment("Anchor 2", 1, "Another comment");
    // Should not be called after unsubscribe
    expect(listener).toHaveBeenCalledTimes(4);
  });

  it("should allow updating current user details", () => {
    const coordinator = new WordCommentCoordinator();
    coordinator.setCurrentUser("Dr. Evelyn Reed", "ER");
    expect(coordinator.currentAuthor).toBe("Dr. Evelyn Reed");
    expect(coordinator.currentInitials).toBe("ER");

    const thread = coordinator.addComment("Methodology", 1, "Peer review requested.");
    expect(thread.root.author).toBe("Dr. Evelyn Reed");
    expect(thread.root.initials).toBe("ER");
  });
});

