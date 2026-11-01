/**
 * WordReferenceCoordinator.ts
 *
 * Implements Sub-Phase 46.2.1: Dynamic Table of Contents, Footnotes/Endnotes & Bibliography presentation coordinator.
 * Strictly conforms to GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - Reactive state management for TOC options, footnote numbering, and citation catalogs.
 * - Support for APA, IEEE, MLA, and Chicago citation styles.
 * - Clean subscriber notification dispatch.
 */

export type TabLeader = "Dot" | "Hyphen" | "Underscore" | "None";

export interface TocOptions {
  startLevel: number;
  endLevel: number;
  useHyperlinks: boolean;
  showPageNumbers: boolean;
  rightAlignPageNumbers: boolean;
  tabLeader: TabLeader;
}

export interface TocEntryItem {
  level: number;
  title: string;
  pageNumber: number;
  bookmarkName: string;
  paragraphIndex: number;
}

export type FootnoteNumberingFormat =
  | "Arabic"
  | "RomanLower"
  | "RomanUpper"
  | "LetterLower"
  | "LetterUpper"
  | "Symbol";

export interface FootnoteItem {
  id: number;
  text: string;
  paragraphIndex: number;
  isEndnote: boolean;
}

export type CitationStyle = "Apa" | "Ieee" | "Mla" | "Chicago";

export type SourceType =
  | "Book"
  | "Article"
  | "InProceedings"
  | "TechReport"
  | "Web"
  | "Misc";

export interface CitationSourceItem {
  id: string;
  sourceType: SourceType;
  authors: string[];
  title: string;
  year?: number;
  journal?: string;
  volume?: string;
  issue?: string;
  pages?: string;
  publisher?: string;
  doi?: string;
  url?: string;
}

export class WordReferenceCoordinator {
  public tocOptions: TocOptions = {
    startLevel: 1,
    endLevel: 3,
    useHyperlinks: true,
    showPageNumbers: true,
    rightAlignPageNumbers: true,
    tabLeader: "Dot",
  };

  public footnoteFormat: FootnoteNumberingFormat = "Arabic";
  public endnoteFormat: FootnoteNumberingFormat = "RomanLower";
  public activeCitationStyle: CitationStyle = "Apa";

  private footnotes: FootnoteItem[] = [];
  private sources: CitationSourceItem[] = [];
  private tocEntries: TocEntryItem[] = [];
  private listeners: Set<() => void> = new Set();

  constructor(options?: {
    tocOptions?: Partial<TocOptions>;
    footnoteFormat?: FootnoteNumberingFormat;
    activeCitationStyle?: CitationStyle;
  }) {
    if (options?.tocOptions) {
      this.tocOptions = { ...this.tocOptions, ...options.tocOptions };
    }
    if (options?.footnoteFormat) {
      this.footnoteFormat = options.footnoteFormat;
    }
    if (options?.activeCitationStyle) {
      this.activeCitationStyle = options.activeCitationStyle;
    }
  }

  // --- Table of Contents Management ---

  public setTocOptions(options: Partial<TocOptions>): void {
    this.tocOptions = { ...this.tocOptions, ...options };
    this.notify();
  }

  public setTocEntries(entries: TocEntryItem[]): void {
    this.tocEntries = [...entries];
    this.notify();
  }

  public getTocEntries(): TocEntryItem[] {
    return this.tocEntries.filter(
      (e) => e.level >= this.tocOptions.startLevel && e.level <= this.tocOptions.endLevel,
    );
  }

  // --- Footnotes and Endnotes Management ---

  public addFootnote(paragraphIndex: number, text: string): FootnoteItem {
    const nextId =
      this.footnotes.filter((f) => !f.isEndnote).reduce((max, f) => Math.max(max, f.id), 0) + 1;
    const item: FootnoteItem = {
      id: nextId,
      text,
      paragraphIndex,
      isEndnote: false,
    };
    this.footnotes.push(item);
    this.notify();
    return item;
  }

  public addEndnote(paragraphIndex: number, text: string): FootnoteItem {
    const nextId =
      this.footnotes.filter((f) => f.isEndnote).reduce((max, f) => Math.max(max, f.id), 0) + 1;
    const item: FootnoteItem = {
      id: nextId,
      text,
      paragraphIndex,
      isEndnote: true,
    };
    this.footnotes.push(item);
    this.notify();
    return item;
  }

  public getFootnotes(): FootnoteItem[] {
    return this.footnotes.filter((f) => !f.isEndnote);
  }

  public getEndnotes(): FootnoteItem[] {
    return this.footnotes.filter((f) => f.isEndnote);
  }

  public formatFootnoteNumber(index: number, isEndnote = false): string {
    const format = isEndnote ? this.endnoteFormat : this.footnoteFormat;
    return WordReferenceCoordinator.formatNumber(index, format);
  }

  public static formatNumber(num: number, format: FootnoteNumberingFormat): string {
    switch (format) {
      case "Arabic":
        return num.toString();
      case "RomanLower":
        return WordReferenceCoordinator.toRoman(num, false);
      case "RomanUpper":
        return WordReferenceCoordinator.toRoman(num, true);
      case "LetterLower":
        return WordReferenceCoordinator.toLetter(num, false);
      case "LetterUpper":
        return WordReferenceCoordinator.toLetter(num, true);
      case "Symbol": {
        const symbols = ["*", "†", "‡", "§", "||", "¶"];
        if (num <= 0) return "*";
        const symIdx = (num - 1) % symbols.length;
        const repeat = Math.floor((num - 1) / symbols.length) + 1;
        return symbols[symIdx].repeat(repeat);
      }
    }
  }

  private static toRoman(num: number, upper: boolean): string {
    if (num <= 0) return upper ? "N" : "n";
    const lookup: [number, string][] = [
      [1000, "M"],
      [900, "CM"],
      [500, "D"],
      [400, "CD"],
      [100, "C"],
      [90, "XC"],
      [50, "L"],
      [40, "XL"],
      [10, "X"],
      [9, "IX"],
      [5, "V"],
      [4, "IV"],
      [1, "I"],
    ];
    let roman = "";
    let n = num;
    for (const [val, sym] of lookup) {
      while (n >= val) {
        roman += sym;
        n -= val;
      }
    }
    return upper ? roman : roman.toLowerCase();
  }

  private static toLetter(num: number, upper: boolean): string {
    if (num <= 0) return upper ? "A" : "a";
    let letters = "";
    let n = num;
    const baseCode = upper ? 65 : 97;
    while (n > 0) {
      const rem = (n - 1) % 26;
      letters = String.fromCharCode(baseCode + rem) + letters;
      n = Math.floor((n - 1) / 26);
    }
    return letters;
  }

  // --- Citations and Bibliography Management ---

  public setCitationStyle(style: CitationStyle): void {
    this.activeCitationStyle = style;
    this.notify();
  }

  public addSource(source: CitationSourceItem): void {
    const existingIdx = this.sources.findIndex((s) => s.id === source.id);
    if (existingIdx >= 0) {
      this.sources[existingIdx] = { ...source };
    } else {
      this.sources.push({ ...source });
    }
    this.notify();
  }

  public addSources(sources: CitationSourceItem[]): void {
    for (const src of sources) {
      this.addSource(src);
    }
  }

  public getSources(): CitationSourceItem[] {
    return this.sources;
  }

  public formatInlineCitation(sourceId: string, page?: string): string {
    const sourceIdx = this.sources.findIndex((s) => s.id === sourceId);
    if (sourceIdx < 0) return `[${sourceId}]`;

    const source = this.sources[sourceIdx];
    const orderIndex = sourceIdx + 1;

    switch (this.activeCitationStyle) {
      case "Ieee":
        return page ? `[${orderIndex}, p. ${page}]` : `[${orderIndex}]`;
      case "Apa": {
        const author = this.formatAuthorsShort(source.authors);
        const year = source.year ? source.year.toString() : "n.d.";
        return page ? `(${author}, ${year}, p. ${page})` : `(${author}, ${year})`;
      }
      case "Mla": {
        const author = this.formatAuthorsShort(source.authors);
        return page ? `(${author} ${page})` : `(${author})`;
      }
      case "Chicago": {
        const author = this.formatAuthorsShort(source.authors);
        const year = source.year ? source.year.toString() : "n.d.";
        return page ? `(${author} ${year}, ${page})` : `(${author} ${year})`;
      }
    }
  }

  public generateBibliography(): string[] {
    return this.sources.map((s, idx) =>
      this.formatBibliographyEntry(s, this.activeCitationStyle, idx + 1),
    );
  }

  private formatBibliographyEntry(
    source: CitationSourceItem,
    style: CitationStyle,
    index: number,
  ): string {
    const authors = source.authors.length > 0 ? source.authors.join(", ") : "Anonymous";
    const year = source.year ? source.year.toString() : "n.d.";

    switch (style) {
      case "Ieee": {
        let entry = `[${index}] ${authors}, "${source.title}"`;
        if (source.journal) entry += `, ${source.journal}`;
        if (source.year) entry += `, ${source.year}.`;
        else entry += ".";
        return entry;
      }
      case "Apa": {
        let entry = `${authors} (${year}). ${source.title}.`;
        if (source.journal) entry += ` ${source.journal}.`;
        return entry;
      }
      case "Mla": {
        let entry = `${authors}. "${source.title}."`;
        if (source.journal) entry += ` ${source.journal},`;
        if (source.year) entry += ` ${source.year}.`;
        else entry += ".";
        return entry;
      }
      case "Chicago": {
        let entry = `${authors}. ${year}. "${source.title}."`;
        if (source.journal) entry += ` ${source.journal}.`;
        return entry;
      }
    }
  }

  private formatAuthorsShort(authors: string[]): string {
    if (!authors || authors.length === 0) return "Anonymous";
    const getLastName = (name: string) => {
      const trimmed = name.trim();
      if (trimmed.includes(",")) return trimmed.split(",")[0].trim();
      const parts = trimmed.split(/\s+/);
      return parts[parts.length - 1];
    };

    const first = getLastName(authors[0]);
    if (authors.length === 1) return first;
    if (authors.length === 2) return `${first} & ${getLastName(authors[1])}`;
    return `${first} et al.`;
  }

  // --- Subscriptions ---

  public subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  private notify(): void {
    for (const listener of this.listeners) {
      try {
        listener();
      } catch {
        // Safe dispatch
      }
    }
  }
}

