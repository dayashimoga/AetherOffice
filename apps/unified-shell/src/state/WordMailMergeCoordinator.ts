/**
 * WordMailMergeCoordinator.ts
 *
 * Implements Sub-Phase 46.2.2: Mail Merge, OMML/LaTeX Equations & .odt/.rtf Import/Export presentation coordinator.
 * Strictly conforms to GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - Reactive state management for CSV data sources, record preview navigation, and template interpolation.
 * - LaTeX math equation editor integration and format interchange management.
 * - Clean subscriber notification dispatch.
 */

export interface MailMergeRecord {
  [fieldName: string]: string;
}

export class WordMailMergeCoordinator {
  public headers: string[] = [];
  public records: MailMergeRecord[] = [];
  public activeRecordIndex = 0;
  public currentLatex = "\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}";
  public readonly supportedFormats = ["docx", "odt", "rtf", "txt", "md"] as const;

  private listeners: Set<() => void> = new Set();

  constructor(options?: {
    csvData?: string;
    initialLatex?: string;
  }) {
    if (options?.csvData) {
      this.loadCsv(options.csvData);
    }
    if (options?.initialLatex) {
      this.currentLatex = options.initialLatex;
    }
  }

  // --- CSV Data Source & Record Navigation ---

  public loadCsv(csvData: string): void {
    const lines = csvData
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    if (lines.length === 0) {
      this.headers = [];
      this.records = [];
      this.activeRecordIndex = 0;
      this.notify();
      return;
    }

    this.headers = this.parseCsvRow(lines[0]);
    this.records = [];

    for (let i = 1; i < lines.length; i++) {
      const row = this.parseCsvRow(lines[i]);
      const record: MailMergeRecord = {};
      this.headers.forEach((header, colIdx) => {
        record[header] = row[colIdx] || "";
      });
      this.records.push(record);
    }

    this.activeRecordIndex = 0;
    this.notify();
  }

  private parseCsvRow(line: string): string[] {
    const fields: string[] = [];
    let cur = "";
    let inQuotes = false;

    for (let i = 0; i < line.length; i++) {
      const c = line[i];
      if (c === '"') {
        if (inQuotes && line[i + 1] === '"') {
          cur += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (c === "," && !inQuotes) {
        fields.push(cur.trim());
        cur = "";
      } else {
        cur += c;
      }
    }
    fields.push(cur.trim());
    return fields;
  }

  public getActiveRecord(): MailMergeRecord | null {
    if (this.records.length === 0) return null;
    return this.records[this.activeRecordIndex] || null;
  }

  public nextRecord(): boolean {
    if (this.activeRecordIndex + 1 < this.records.length) {
      this.activeRecordIndex++;
      this.notify();
      return true;
    }
    return false;
  }

  public prevRecord(): boolean {
    if (this.activeRecordIndex > 0) {
      this.activeRecordIndex--;
      this.notify();
      return true;
    }
    return false;
  }

  public goToRecord(index: number): boolean {
    if (index >= 0 && index < this.records.length) {
      this.activeRecordIndex = index;
      this.notify();
      return true;
    }
    return false;
  }

  public totalRecords(): number {
    return this.records.length;
  }

  // --- Field Extraction and Preview Resolution ---

  public static findMergeFields(templateText: string): string[] {
    const fields = new Set<string>();

    // 1. Match chevrons «FieldName»
    const chevronMatches = templateText.match(/«([^»]+)»/g);
    if (chevronMatches) {
      for (const m of chevronMatches) {
        const name = m.replace(/[«»]/g, "").trim();
        if (name) fields.add(name);
      }
    }

    // 2. Match MERGEFIELD Name or MERGEFIELD "Name"
    const mergeFieldMatches = templateText.match(/MERGEFIELD\s+("([^"]+)"|(\S+))/gi);
    if (mergeFieldMatches) {
      for (const m of mergeFieldMatches) {
        const parts = m.split(/\s+/);
        if (parts.length >= 2) {
          const name = parts[1].replace(/"/g, "").trim();
          if (name) fields.add(name);
        }
      }
    }

    return Array.from(fields).sort();
  }

  public resolvePreview(templateText: string): string {
    const active = this.getActiveRecord();
    if (!active) return templateText;

    let result = templateText;
    for (const [key, val] of Object.entries(active)) {
      result = result.replaceAll(`«${key}»`, val);
      result = result.replaceAll(`MERGEFIELD "${key}"`, val);
      result = result.replaceAll(`MERGEFIELD ${key}`, val);
    }
    return result;
  }

  public generateBatchPreview(templateText: string): string[] {
    return this.records.map((rec) => {
      let result = templateText;
      for (const [key, val] of Object.entries(rec)) {
        result = result.replaceAll(`«${key}»`, val);
        result = result.replaceAll(`MERGEFIELD "${key}"`, val);
        result = result.replaceAll(`MERGEFIELD ${key}`, val);
      }
      return result;
    });
  }

  // --- LaTeX Math Equation Editor Integration ---

  public setLatex(latex: string): void {
    this.currentLatex = latex;
    this.notify();
  }

  public getLatex(): string {
    return this.currentLatex;
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

