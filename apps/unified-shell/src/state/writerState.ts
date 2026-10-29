export interface WordRun { text: string; bold?: boolean; italic?: boolean; underline?: boolean; font?: string; size?: number; }
export interface WordParagraph { id: string; style: string; runs: WordRun[]; alignment: "left" | "center" | "right" | "justify"; }
export interface WordDocumentModel { id: string; title: string; sections: { paragraphs: WordParagraph[] }[]; }

export class WriterState {
  doc: WordDocumentModel = { id: "doc-1", title: "Document 1.docx", sections: [{ paragraphs: [
    { id: "p1", style: "Heading1", alignment: "left", runs: [{ text: "Executive Summary", bold: true, size: 24 }] },
    { id: "p2", style: "Normal", alignment: "left", runs: [{ text: "Production-grade open-source document authoring powered by Rust." }] }
  ]}] };
  cursor = { paragraphIndex: 0, runIndex: 0, offset: 0 };
  activeFormat = { bold: false, italic: false, underline: false, font: "Calibri", size: 11, alignment: "left" };

  execCommand(cmd: string, val?: any) {
    if (cmd === "toggleBold") this.activeFormat.bold = !this.activeFormat.bold;
    if (cmd === "toggleItalic") this.activeFormat.italic = !this.activeFormat.italic;
    if (cmd === "setFont") this.activeFormat.font = val;
    if (cmd === "setAlignment") this.activeFormat.alignment = val;
    console.log(`[WriterCommand] Executed: ${cmd}`, val, this.activeFormat);
  }
}
export const writerState = new WriterState();
