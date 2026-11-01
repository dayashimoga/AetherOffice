import { describe, it, expect, beforeEach } from "vitest";
import {
  WordRevisionCoordinator,
  AuthorColorPalette,
  RevisionViewMode,
} from "../state/WordRevisionCoordinator";
import type { WordDocumentModel, WordRun } from "../state/writerState";

describe("Sub-Phase 46.1.1: Word Revision Tracking & Redline Renderer", () => {
  let coordinator: WordRevisionCoordinator;
  let testDoc: WordDocumentModel;

  beforeEach(() => {
    coordinator = new WordRevisionCoordinator({
      trackChanges: true,
      viewMode: "AllMarkup",
      activeAuthor: "Alice",
    });

    testDoc = {
      id: "doc-1",
      title: "Revision Test Document",
      pages: [
        {
          pageNumber: 1,
          headerText: "",
          bodyHtml: "",
          footerText: "",
          paragraphs: [
            {
              id: "p1",
              style: "Normal",
              alignment: "left",
              runs: [
                { text: "Base text " },
                { text: "inserted by Alice", insertion: true },
                { text: " deleted old text", deletion: true },
                { text: " concluding remark." },
              ],
            },
          ],
        },
      ],
    };
  });

  describe("AuthorColorPalette", () => {
    it("should provide deterministic hex colors for authors", () => {
      const color1 = AuthorColorPalette.getColorForAuthor("Alice");
      const color2 = AuthorColorPalette.getColorForAuthor("Alice");
      const colorBob = AuthorColorPalette.getColorForAuthor("Bob");

      expect(color1).toBe(color2);
      expect(color1).toMatch(/^#[0-9A-Fa-f]{6}$/);
      expect(colorBob).toMatch(/^#[0-9A-Fa-f]{6}$/);
    });

    it("should fallback gracefully for empty author", () => {
      const fallback = AuthorColorPalette.getColorForAuthor("");
      expect(fallback).toBe(AuthorColorPalette.PALETTE[0]);
    });
  });

  describe("Revision Tracking Toggle & View Modes", () => {
    it("should toggle track changes on and off", () => {
      expect(coordinator.trackChanges).toBe(true);
      expect(coordinator.toggleTrackChanges()).toBe(false);
      expect(coordinator.trackChanges).toBe(false);
      expect(coordinator.toggleTrackChanges()).toBe(true);
      expect(coordinator.trackChanges).toBe(true);
    });

    it("should allow setting revision view modes", () => {
      const modes: RevisionViewMode[] = ["SimpleMarkup", "AllMarkup", "NoMarkup", "Original"];
      modes.forEach((mode) => {
        coordinator.setViewMode(mode);
        expect(coordinator.viewMode).toBe(mode);
      });
    });
  });

  describe("Revision Collection & Summaries", () => {
    it("should accurately list all tracked changes in document", () => {
      const revs = coordinator.listRevisions(testDoc);
      expect(revs).toHaveLength(2);

      const ins = revs.find((r) => r.revisionType === "Insertion");
      const del = revs.find((r) => r.revisionType === "Deletion");

      expect(ins).toBeDefined();
      expect(ins?.textPreview).toBe("inserted by Alice");
      expect(del).toBeDefined();
      expect(del?.textPreview).toBe(" deleted old text");
    });

    it("should compute accurate summary counts", () => {
      const summary = coordinator.getSummary(testDoc);
      expect(summary.insertions).toBe(1);
      expect(summary.deletions).toBe(1);
      expect(summary.total).toBe(2);
    });
  });

  describe("Single Revision Actions", () => {
    it("should accept an insertion by retaining text and clearing insertion flag", () => {
      const revs = coordinator.listRevisions(testDoc);
      const insRev = revs.find((r) => r.revisionType === "Insertion")!;

      const success = coordinator.acceptRevision(testDoc, insRev.id);
      expect(success).toBe(true);

      const remainingRevs = coordinator.listRevisions(testDoc);
      expect(remainingRevs).toHaveLength(1);
      expect(remainingRevs[0].revisionType).toBe("Deletion");

      const runs = testDoc.pages[0].paragraphs[0].runs;
      expect(runs[1].text).toBe("inserted by Alice");
      expect(runs[1].insertion).toBe(false);
    });

    it("should accept a deletion by completely removing the deleted run", () => {
      const revs = coordinator.listRevisions(testDoc);
      const delRev = revs.find((r) => r.revisionType === "Deletion")!;

      const success = coordinator.acceptRevision(testDoc, delRev.id);
      expect(success).toBe(true);

      const remainingRevs = coordinator.listRevisions(testDoc);
      expect(remainingRevs).toHaveLength(1);
      expect(remainingRevs[0].revisionType).toBe("Insertion");

      const runs = testDoc.pages[0].paragraphs[0].runs;
      expect(runs).toHaveLength(3);
      expect(runs.some((r: WordRun) => r.text === " deleted old text")).toBe(false);
    });

    it("should reject an insertion by removing the inserted run", () => {
      const revs = coordinator.listRevisions(testDoc);
      const insRev = revs.find((r) => r.revisionType === "Insertion")!;

      const success = coordinator.rejectRevision(testDoc, insRev.id);
      expect(success).toBe(true);

      const remainingRevs = coordinator.listRevisions(testDoc);
      expect(remainingRevs).toHaveLength(1);
      expect(remainingRevs[0].revisionType).toBe("Deletion");

      const runs = testDoc.pages[0].paragraphs[0].runs;
      expect(runs.some((r: WordRun) => r.text === "inserted by Alice")).toBe(false);
    });

    it("should reject a deletion by restoring it as a standard run", () => {
      const revs = coordinator.listRevisions(testDoc);
      const delRev = revs.find((r) => r.revisionType === "Deletion")!;

      const success = coordinator.rejectRevision(testDoc, delRev.id);
      expect(success).toBe(true);

      const remainingRevs = coordinator.listRevisions(testDoc);
      expect(remainingRevs).toHaveLength(1);
      expect(remainingRevs[0].revisionType).toBe("Insertion");

      const runs = testDoc.pages[0].paragraphs[0].runs;
      const restored = runs.find((r: WordRun) => r.text === " deleted old text");
      expect(restored).toBeDefined();
      expect(restored?.deletion).toBe(false);
    });
  });

  describe("Batch Revision Actions", () => {
    it("should accept all revisions cleanly in a single pass", () => {
      const count = coordinator.acceptAllRevisions(testDoc);
      expect(count).toBe(2);

      const remaining = coordinator.listRevisions(testDoc);
      expect(remaining).toHaveLength(0);

      const runs = testDoc.pages[0].paragraphs[0].runs;
      expect(runs).toHaveLength(3);
      expect(runs[0].text).toBe("Base text ");
      expect(runs[1].text).toBe("inserted by Alice");
      expect(runs[1].insertion).toBe(false);
      expect(runs[2].text).toBe(" concluding remark.");
    });

    it("should reject all revisions cleanly in a single pass", () => {
      const count = coordinator.rejectAllRevisions(testDoc);
      expect(count).toBe(2);

      const remaining = coordinator.listRevisions(testDoc);
      expect(remaining).toHaveLength(0);

      const runs = testDoc.pages[0].paragraphs[0].runs;
      expect(runs).toHaveLength(3);
      expect(runs[0].text).toBe("Base text ");
      expect(runs[1].text).toBe(" deleted old text");
      expect(runs[1].deletion).toBe(false);
      expect(runs[2].text).toBe(" concluding remark.");
    });
  });

  describe("Display View Mode Redline Filtering", () => {
    it("AllMarkup: formats insertions with underline and deletions with strikethrough", () => {
      const runs = testDoc.pages[0].paragraphs[0].runs;
      const filtered = coordinator.filterRunsForViewMode(runs, "AllMarkup");

      expect(filtered).toHaveLength(4);
      expect(filtered[1].underline).toBe(true);
      expect(filtered[2].strikethrough).toBe(true);
    });

    it("SimpleMarkup: shows insertions clean, hides deletions", () => {
      const runs = testDoc.pages[0].paragraphs[0].runs;
      const filtered = coordinator.filterRunsForViewMode(runs, "SimpleMarkup");

      expect(filtered).toHaveLength(3);
      expect(filtered.some((r: WordRun) => r.text === "inserted by Alice")).toBe(true);
      expect(filtered.some((r: WordRun) => r.text === " deleted old text")).toBe(false);
    });

    it("NoMarkup: shows final document state", () => {
      const runs = testDoc.pages[0].paragraphs[0].runs;
      const filtered = coordinator.filterRunsForViewMode(runs, "NoMarkup");

      expect(filtered).toHaveLength(3);
      expect(filtered.some((r: WordRun) => r.text === "inserted by Alice")).toBe(true);
      expect(filtered.some((r: WordRun) => r.text === " deleted old text")).toBe(false);
    });

    it("Original: shows original document state", () => {
      const runs = testDoc.pages[0].paragraphs[0].runs;
      const filtered = coordinator.filterRunsForViewMode(runs, "Original");

      expect(filtered).toHaveLength(3);
      expect(filtered.some((r: WordRun) => r.text === "inserted by Alice")).toBe(false);
      expect(filtered.some((r: WordRun) => r.text === " deleted old text")).toBe(true);
    });
  });
});
