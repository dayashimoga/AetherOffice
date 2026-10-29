export interface PdfFormField { id: string; name: string; type: "text" | "checkbox"; value: string | boolean; rect: [number, number, number, number]; }
export class PdfState {
  pageCount = 1;
  currentPage = 1;
  zoom = 1.0;
  formFields: PdfFormField[] = [
    { id: "f1", name: "Full_Name", type: "text", value: "John Doe", rect: [100, 200, 250, 30] },
    { id: "f2", name: "Agreed", type: "checkbox", value: true, rect: [100, 250, 20, 20] }
  ];
  activeTool: "select" | "edit_text" | "highlight" | "form_fill" | "sign" = "form_fill";

  setTool(t: typeof this.activeTool) { this.activeTool = t; }
}
export const pdfState = new PdfState();
