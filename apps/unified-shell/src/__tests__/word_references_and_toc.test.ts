import { describe, it, expect, vi } from "vitest";
import {
  WordReferenceCoordinator,
  type TocEntryItem,
  type CitationSourceItem,
} from "../state/WordReferenceCoordinator";

describe("WordReferenceCoordinator", () => {
  it("should initialize with default options", () => {
    const coordinator = new WordReferenceCoordinator();
    expect(coordinator.tocOptions.startLevel).toBe(1);
    expect(coordinator.tocOptions.endLevel).toBe(3);
    expect(coordinator.tocOptions.useHyperlinks).toBe(true);
    expect(coordinator.tocOptions.tabLeader).toBe("Dot");
    expect(coordinator.footnoteFormat).toBe("Arabic");
    expect(coordinator.endnoteFormat).toBe("RomanLower");
    expect(coordinator.activeCitationStyle).toBe("Apa");
    expect(coordinator.getFootnotes()).toHaveLength(0);
    expect(coordinator.getEndnotes()).toHaveLength(0);
    expect(coordinator.getSources()).toHaveLength(0);
  });

  it("should update TOC options and filter visible entries by level", () => {
    const coordinator = new WordReferenceCoordinator();
    const entries: TocEntryItem[] = [
      { level: 1, title: "Chapter 1", pageNumber: 1, bookmarkName: "_Toc1", paragraphIndex: 0 },
      { level: 2, title: "Section 1.1", pageNumber: 3, bookmarkName: "_Toc2", paragraphIndex: 2 },
      { level: 3, title: "Subsection 1.1.1", pageNumber: 5, bookmarkName: "_Toc3", paragraphIndex: 4 },
      { level: 4, title: "Clause 1.1.1.1", pageNumber: 6, bookmarkName: "_Toc4", paragraphIndex: 5 },
    ];

    coordinator.setTocEntries(entries);
    expect(coordinator.getTocEntries()).toHaveLength(3); // levels 1..=3 by default

    // Restrict to levels 1..=2
    coordinator.setTocOptions({ startLevel: 1, endLevel: 2 });
    const filtered = coordinator.getTocEntries();
    expect(filtered).toHaveLength(2);
    expect(filtered.map((e) => e.title)).toEqual(["Chapter 1", "Section 1.1"]);
  });

  it("should manage footnotes and endnotes with independent IDs", () => {
    const coordinator = new WordReferenceCoordinator();
    const fn1 = coordinator.addFootnote(0, "First footnote.");
    const fn2 = coordinator.addFootnote(2, "Second footnote.");
    expect(fn1.id).toBe(1);
    expect(fn2.id).toBe(2);
    expect(coordinator.getFootnotes()).toHaveLength(2);

    const en1 = coordinator.addEndnote(5, "First endnote.");
    expect(en1.id).toBe(1);
    expect(en1.isEndnote).toBe(true);
    expect(coordinator.getEndnotes()).toHaveLength(1);
  });

  it("should format footnote numbers accurately across all numbering styles", () => {
    // Arabic
    expect(WordReferenceCoordinator.formatNumber(1, "Arabic")).toBe("1");
    expect(WordReferenceCoordinator.formatNumber(42, "Arabic")).toBe("42");

    // Roman lower and upper
    expect(WordReferenceCoordinator.formatNumber(1, "RomanLower")).toBe("i");
    expect(WordReferenceCoordinator.formatNumber(4, "RomanLower")).toBe("iv");
    expect(WordReferenceCoordinator.formatNumber(9, "RomanLower")).toBe("ix");
    expect(WordReferenceCoordinator.formatNumber(14, "RomanLower")).toBe("xiv");

    expect(WordReferenceCoordinator.formatNumber(1, "RomanUpper")).toBe("I");
    expect(WordReferenceCoordinator.formatNumber(4, "RomanUpper")).toBe("IV");
    expect(WordReferenceCoordinator.formatNumber(14, "RomanUpper")).toBe("XIV");

    // Letter lower and upper
    expect(WordReferenceCoordinator.formatNumber(1, "LetterLower")).toBe("a");
    expect(WordReferenceCoordinator.formatNumber(2, "LetterLower")).toBe("b");
    expect(WordReferenceCoordinator.formatNumber(1, "LetterUpper")).toBe("A");
    expect(WordReferenceCoordinator.formatNumber(26, "LetterUpper")).toBe("Z");

    // Symbols
    expect(WordReferenceCoordinator.formatNumber(1, "Symbol")).toBe("*");
    expect(WordReferenceCoordinator.formatNumber(2, "Symbol")).toBe("†");
    expect(WordReferenceCoordinator.formatNumber(3, "Symbol")).toBe("‡");
    expect(WordReferenceCoordinator.formatNumber(4, "Symbol")).toBe("§");
    expect(WordReferenceCoordinator.formatNumber(7, "Symbol")).toBe("**");
  });

  it("should manage citation source catalog and handle updates", () => {
    const coordinator = new WordReferenceCoordinator();
    const src1: CitationSourceItem = {
      id: "Knuth1984",
      sourceType: "Article",
      authors: ["Donald E. Knuth"],
      title: "Literate Programming",
      year: 1984,
      journal: "The Computer Journal",
    };
    coordinator.addSource(src1);
    expect(coordinator.getSources()).toHaveLength(1);

    // Update same source
    const src1Updated: CitationSourceItem = {
      ...src1,
      title: "Literate Programming (Revised)",
    };
    coordinator.addSource(src1Updated);
    expect(coordinator.getSources()).toHaveLength(1);
    expect(coordinator.getSources()[0].title).toBe("Literate Programming (Revised)");
  });

  it("should format inline citations across IEEE, APA, MLA, and Chicago styles", () => {
    const coordinator = new WordReferenceCoordinator();
    const src: CitationSourceItem = {
      id: "Knuth1984",
      sourceType: "Article",
      authors: ["Donald E. Knuth"],
      title: "Literate Programming",
      year: 1984,
    };
    coordinator.addSource(src);

    // IEEE
    coordinator.setCitationStyle("Ieee");
    expect(coordinator.formatInlineCitation("Knuth1984")).toBe("[1]");
    expect(coordinator.formatInlineCitation("Knuth1984", "42")).toBe("[1, p. 42]");

    // APA
    coordinator.setCitationStyle("Apa");
    expect(coordinator.formatInlineCitation("Knuth1984")).toBe("(Knuth, 1984)");
    expect(coordinator.formatInlineCitation("Knuth1984", "42")).toBe("(Knuth, 1984, p. 42)");

    // MLA
    coordinator.setCitationStyle("Mla");
    expect(coordinator.formatInlineCitation("Knuth1984")).toBe("(Knuth)");
    expect(coordinator.formatInlineCitation("Knuth1984", "42")).toBe("(Knuth 42)");

    // Chicago
    coordinator.setCitationStyle("Chicago");
    expect(coordinator.formatInlineCitation("Knuth1984")).toBe("(Knuth 1984)");
    expect(coordinator.formatInlineCitation("Knuth1984", "42")).toBe("(Knuth 1984, 42)");
  });

  it("should handle author counts in citations (single, pair, and 3+ et al.)", () => {
    const coordinator = new WordReferenceCoordinator({ activeCitationStyle: "Apa" });
    const srcTwo: CitationSourceItem = {
      id: "Shannon1948",
      sourceType: "Article",
      authors: ["Claude Shannon", "Warren Weaver"],
      title: "Theory",
      year: 1948,
    };
    const srcThree: CitationSourceItem = {
      id: "Multi2026",
      sourceType: "Article",
      authors: ["Alice Smith", "Bob Jones", "Charlie Brown"],
      title: "Distributed Systems",
      year: 2026,
    };
    coordinator.addSources([srcTwo, srcThree]);

    expect(coordinator.formatInlineCitation("Shannon1948")).toBe("(Shannon & Weaver, 1948)");
    expect(coordinator.formatInlineCitation("Multi2026")).toBe("(Smith et al., 2026)");
  });

  it("should generate formatted bibliography lists according to active citation style", () => {
    const coordinator = new WordReferenceCoordinator();
    const src: CitationSourceItem = {
      id: "Lamport1994",
      sourceType: "Book",
      authors: ["Leslie Lamport"],
      title: "LaTeX System",
      year: 1994,
      publisher: "Addison-Wesley",
    };
    coordinator.addSource(src);

    coordinator.setCitationStyle("Ieee");
    const ieeeBib = coordinator.generateBibliography();
    expect(ieeeBib[0]).toBe('[1] Leslie Lamport, "LaTeX System", 1994.');

    coordinator.setCitationStyle("Apa");
    const apaBib = coordinator.generateBibliography();
    expect(apaBib[0]).toBe("Leslie Lamport (1994). LaTeX System.");
  });

  it("should dispatch subscriber notifications on reference state changes", () => {
    const coordinator = new WordReferenceCoordinator();
    const listener = vi.fn();
    const unsubscribe = coordinator.subscribe(listener);

    coordinator.setTocOptions({ startLevel: 2 });
    expect(listener).toHaveBeenCalledTimes(1);

    coordinator.addFootnote(0, "Note");
    expect(listener).toHaveBeenCalledTimes(2);

    coordinator.setCitationStyle("Ieee");
    expect(listener).toHaveBeenCalledTimes(3);

    unsubscribe();
    coordinator.addFootnote(1, "Another note");
    expect(listener).toHaveBeenCalledTimes(3);
  });
});

