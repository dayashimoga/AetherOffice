export interface SlideShape { id: string; type: "text" | "rect" | "ellipse"; x: number; y: number; w: number; h: number; fill: string; text?: string; }
export interface SlideData { id: string; title: string; shapes: SlideShape[]; }

export class SlidesState {
  slides: SlideData[] = [
    { id: "s1", title: "Title Slide", shapes: [
      { id: "sh1", type: "text", x: 100, y: 150, w: 800, h: 80, fill: "transparent", text: "Kinetiq Presentation" },
      { id: "sh2", type: "rect", x: 100, y: 250, w: 400, h: 4, fill: "#2563eb" }
    ]},
    { id: "s2", title: "Problem Statement", shapes: [] },
    { id: "s3", title: "Solution & Demo", shapes: [] }
  ];
  currentSlideIndex = 0;
  selectedShapeId: string | null = null;

  selectSlide(idx: number) { this.currentSlideIndex = idx; this.selectedShapeId = null; }
  selectShape(id: string) { this.selectedShapeId = id; }
}
export const slidesState = new SlidesState();
