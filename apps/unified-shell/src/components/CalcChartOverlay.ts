import { CalcState, SheetChart } from "../state/calcState";

const PALETTE = ["#2563eb", "#107c41", "#f59e0b", "#8b5cf6", "#ec4899", "#06b6d4"];

export class CalcChartOverlay {
  private container: HTMLElement;
  private state: CalcState;
  private draggingChartId: string | null = null;
  private dragStartX = 0;
  private dragStartY = 0;
  private initialChartX = 0;
  private initialChartY = 0;

  private resizingChartId: string | null = null;
  private resizeStartX = 0;
  private resizeStartY = 0;
  private initialChartW = 0;
  private initialChartH = 0;

  constructor(containerId: string, state: CalcState) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`Chart overlay container #${containerId} not found`);
    this.container = el;
    this.state = state;
    this.applyStyles();
    this.attachGlobalEvents();
    this.render();
  }

  private applyStyles() {
    this.container.style.position = "absolute";
    this.container.style.top = "0";
    this.container.style.left = "0";
    this.container.style.width = "100%";
    this.container.style.height = "100%";
    this.container.style.pointerEvents = "none"; // allow grid clicks through transparent areas
    this.container.style.zIndex = "50";
    this.container.style.overflow = "hidden";
  }

  public refresh() {
    this.render();
  }

  private render() {
    this.container.innerHTML = "";
    const charts = this.state.getCharts();

    charts.forEach((chart) => {
      const card = document.createElement("div");
      card.className = "calc-chart-card";
      card.dataset.chartId = chart.id;
      card.style.position = "absolute";
      card.style.left = `${chart.x}px`;
      card.style.top = `${chart.y}px`;
      card.style.width = `${chart.width}px`;
      card.style.height = `${chart.height}px`;
      card.style.backgroundColor = "#ffffff";
      card.style.border = "1px solid #d1d5db";
      card.style.borderRadius = "8px";
      card.style.boxShadow = "0 8px 24px rgba(0, 0, 0, 0.12)";
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.pointerEvents = "auto";
      card.style.overflow = "hidden";
      card.style.fontFamily = "Segoe UI, -apple-system, sans-serif";

      // 1. Header (Drag handle)
      const header = document.createElement("div");
      header.className = "calc-chart-header";
      header.style.display = "flex";
      header.style.justifyContent = "space-between";
      header.style.alignItems = "center";
      header.style.padding = "6px 12px";
      header.style.backgroundColor = "#f9fafb";
      header.style.borderBottom = "1px solid #e5e7eb";
      header.style.cursor = "move";
      header.style.userSelect = "none";

      const titleSpan = document.createElement("span");
      titleSpan.className = "calc-chart-title";
      titleSpan.style.fontWeight = "600";
      titleSpan.style.fontSize = "12px";
      titleSpan.style.color = "#1f2937";
      titleSpan.textContent = `${this.getTypeIcon(chart.type)} ${chart.title}`;

      const closeBtn = document.createElement("button");
      closeBtn.className = "calc-chart-close-btn";
      closeBtn.textContent = "✕";
      closeBtn.style.background = "none";
      closeBtn.style.border = "none";
      closeBtn.style.cursor = "pointer";
      closeBtn.style.color = "#9ca3af";
      closeBtn.style.fontSize = "13px";
      closeBtn.onclick = (e) => {
        e.stopPropagation();
        this.state.deleteChart(chart.id);
        this.render();
      };

      header.appendChild(titleSpan);
      header.appendChild(closeBtn);

      // Attach header drag
      header.onmousedown = (e) => {
        e.preventDefault();
        this.draggingChartId = chart.id;
        this.dragStartX = e.clientX;
        this.dragStartY = e.clientY;
        this.initialChartX = chart.x;
        this.initialChartY = chart.y;
      };

      card.appendChild(header);

      // 2. Chart SVG Body
      const body = document.createElement("div");
      body.className = "calc-chart-body";
      body.style.flex = "1";
      body.style.position = "relative";
      body.style.padding = "8px";
      body.style.display = "flex";
      body.style.alignItems = "center";
      body.style.justifyContent = "center";

      const svg = this.generateChartSvg(chart);
      body.appendChild(svg);
      card.appendChild(body);

      // 3. Resize Handle
      const resizeHandle = document.createElement("div");
      resizeHandle.className = "calc-chart-resize-handle";
      resizeHandle.style.position = "absolute";
      resizeHandle.style.right = "2px";
      resizeHandle.style.bottom = "2px";
      resizeHandle.style.width = "10px";
      resizeHandle.style.height = "10px";
      resizeHandle.style.cursor = "se-resize";
      resizeHandle.style.borderRight = "2px solid #9ca3af";
      resizeHandle.style.borderBottom = "2px solid #9ca3af";

      resizeHandle.onmousedown = (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.resizingChartId = chart.id;
        this.resizeStartX = e.clientX;
        this.resizeStartY = e.clientY;
        this.initialChartW = chart.width;
        this.initialChartH = chart.height;
      };

      card.appendChild(resizeHandle);
      this.container.appendChild(card);
    });
  }

  private getTypeIcon(type: string): string {
    switch (type) {
      case "column":
        return "📊";
      case "bar":
        return "▦";
      case "line":
        return "📈";
      case "pie":
        return "🥧";
      default:
        return "📊";
    }
  }

  private generateChartSvg(chart: SheetChart): SVGSVGElement {
    const data = this.state.getChartData(chart);
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    const w = Math.max(chart.width - 16, 200);
    const h = Math.max(chart.height - 40, 140);
    svg.setAttribute("width", "100%");
    svg.setAttribute("height", "100%");
    svg.setAttribute("viewBox", `0 0 ${w} ${h}`);
    svg.style.overflow = "visible";

    if (chart.type === "pie") {
      this.renderPieChartSvg(svg, data, w, h);
    } else if (chart.type === "doughnut") {
      this.renderPieChartSvg(svg, data, w, h, true);
    } else if (chart.type === "line") {
      this.renderLineChartSvg(svg, data, w, h);
    } else if (chart.type === "bar") {
      this.renderBarChartSvg(svg, data, w, h);
    } else if (chart.type === "scatter") {
      this.renderScatterChartSvg(svg, data, w, h);
    } else {
      // Default: column chart
      this.renderColumnChartSvg(svg, data, w, h);
    }

    return svg;
  }

  private renderColumnChartSvg(
    svg: SVGSVGElement,
    data: { categories: string[]; series: { name: string; values: number[] }[] },
    w: number,
    h: number
  ) {
    const padLeft = 40;
    const padBottom = 26;
    const padTop = 16;
    const padRight = 16;
    const plotW = w - padLeft - padRight;
    const plotH = h - padTop - padBottom;

    // Find max value
    let maxVal = 10;
    data.series.forEach((s) => {
      s.values.forEach((v) => {
        if (v > maxVal) maxVal = v;
      });
    });
    maxVal = Math.ceil(maxVal * 1.15);

    // 1. Gridlines & Y-Axis labels
    for (let i = 0; i <= 4; i++) {
      const y = padTop + plotH - (i / 4) * plotH;
      const val = Math.round((i / 4) * maxVal);

      const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
      line.setAttribute("x1", String(padLeft));
      line.setAttribute("y1", String(y));
      line.setAttribute("x2", String(w - padRight));
      line.setAttribute("y2", String(y));
      line.setAttribute("stroke", "#f3f4f6");
      line.setAttribute("stroke-width", "1");
      svg.appendChild(line);

      const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
      text.setAttribute("x", String(padLeft - 6));
      text.setAttribute("y", String(y + 3));
      text.setAttribute("text-anchor", "end");
      text.setAttribute("font-size", "10");
      text.setAttribute("fill", "#6b7280");
      text.textContent = String(val);
      svg.appendChild(text);
    }

    // 2. Bars
    const numCats = Math.max(data.categories.length, 1);
    const catSlotW = plotW / numCats;
    const numSeries = Math.max(data.series.length, 1);
    const barW = Math.max(6, (catSlotW * 0.7) / numSeries);

    data.categories.forEach((cat, cIdx) => {
      const groupX = padLeft + cIdx * catSlotW + catSlotW * 0.15;

      data.series.forEach((s, sIdx) => {
        const val = s.values[cIdx] || 0;
        const barH = Math.max(2, (val / maxVal) * plotH);
        const bx = groupX + sIdx * barW;
        const by = padTop + plotH - barH;

        const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        rect.setAttribute("x", String(bx));
        rect.setAttribute("y", String(by));
        rect.setAttribute("width", String(barW - 2));
        rect.setAttribute("height", String(barH));
        rect.setAttribute("fill", PALETTE[sIdx % PALETTE.length]);
        rect.setAttribute("rx", "2");

        const titleTag = document.createElementNS("http://www.w3.org/2000/svg", "title");
        titleTag.textContent = `${s.name}: ${val}`;
        rect.appendChild(titleTag);

        svg.appendChild(rect);
      });

      // Category label
      const catText = document.createElementNS("http://www.w3.org/2000/svg", "text");
      catText.setAttribute("x", String(groupX + (numSeries * barW) / 2));
      catText.setAttribute("y", String(h - 6));
      catText.setAttribute("text-anchor", "middle");
      catText.setAttribute("font-size", "10");
      catText.setAttribute("fill", "#4b5563");
      catText.textContent = cat.length > 7 ? `${cat.substring(0, 6)}…` : cat;
      svg.appendChild(catText);
    });
  }

  private renderBarChartSvg(
    svg: SVGSVGElement,
    data: { categories: string[]; series: { name: string; values: number[] }[] },
    w: number,
    h: number
  ) {
    const padLeft = 60;
    const padBottom = 20;
    const padTop = 16;
    const padRight = 24;
    const plotW = w - padLeft - padRight;
    const plotH = h - padTop - padBottom;

    let maxVal = 10;
    data.series.forEach((s) => {
      s.values.forEach((v) => {
        if (v > maxVal) maxVal = v;
      });
    });
    maxVal = Math.ceil(maxVal * 1.15);

    const numCats = Math.max(data.categories.length, 1);
    const catSlotH = plotH / numCats;
    const barH = Math.max(6, catSlotH * 0.6);

    data.categories.forEach((cat, cIdx) => {
      const by = padTop + cIdx * catSlotH + catSlotH * 0.2;
      const val = data.series[0]?.values[cIdx] || 0;
      const barLen = Math.max(2, (val / maxVal) * plotW);

      const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
      rect.setAttribute("x", String(padLeft));
      rect.setAttribute("y", String(by));
      rect.setAttribute("width", String(barLen));
      rect.setAttribute("height", String(barH));
      rect.setAttribute("fill", PALETTE[0]);
      rect.setAttribute("rx", "2");
      svg.appendChild(rect);

      // Category label
      const catText = document.createElementNS("http://www.w3.org/2000/svg", "text");
      catText.setAttribute("x", String(padLeft - 6));
      catText.setAttribute("y", String(by + barH / 2 + 3));
      catText.setAttribute("text-anchor", "end");
      catText.setAttribute("font-size", "10");
      catText.setAttribute("fill", "#4b5563");
      catText.textContent = cat;
      svg.appendChild(catText);

      // Value label
      const valText = document.createElementNS("http://www.w3.org/2000/svg", "text");
      valText.setAttribute("x", String(padLeft + barLen + 4));
      valText.setAttribute("y", String(by + barH / 2 + 3));
      valText.setAttribute("font-size", "10");
      valText.setAttribute("fill", "#6b7280");
      valText.textContent = String(val);
      svg.appendChild(valText);
    });
  }

  private renderLineChartSvg(
    svg: SVGSVGElement,
    data: { categories: string[]; series: { name: string; values: number[] }[] },
    w: number,
    h: number
  ) {
    const padLeft = 40;
    const padBottom = 26;
    const padTop = 16;
    const padRight = 16;
    const plotW = w - padLeft - padRight;
    const plotH = h - padTop - padBottom;

    let maxVal = 10;
    data.series.forEach((s) => {
      s.values.forEach((v) => {
        if (v > maxVal) maxVal = v;
      });
    });
    maxVal = Math.ceil(maxVal * 1.15);

    // Gridlines
    for (let i = 0; i <= 4; i++) {
      const y = padTop + plotH - (i / 4) * plotH;
      const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
      line.setAttribute("x1", String(padLeft));
      line.setAttribute("y1", String(y));
      line.setAttribute("x2", String(w - padRight));
      line.setAttribute("y2", String(y));
      line.setAttribute("stroke", "#f3f4f6");
      svg.appendChild(line);
    }

    const numPoints = Math.max(data.categories.length, 1);
    const stepX = numPoints > 1 ? plotW / (numPoints - 1) : plotW;

    data.series.forEach((s, sIdx) => {
      const points: string[] = [];

      s.values.forEach((val, pIdx) => {
        const px = padLeft + pIdx * stepX;
        const py = padTop + plotH - (val / maxVal) * plotH;
        points.push(`${px},${py}`);

        // Dot
        const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        circle.setAttribute("cx", String(px));
        circle.setAttribute("cy", String(py));
        circle.setAttribute("r", "3.5");
        circle.setAttribute("fill", PALETTE[sIdx % PALETTE.length]);
        circle.setAttribute("stroke", "#ffffff");
        circle.setAttribute("stroke-width", "1.5");
        svg.appendChild(circle);
      });

      const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
      polyline.setAttribute("fill", "none");
      polyline.setAttribute("stroke", PALETTE[sIdx % PALETTE.length]);
      polyline.setAttribute("stroke-width", "2");
      polyline.setAttribute("points", points.join(" "));
      svg.appendChild(polyline);
    });

    // Categories
    data.categories.forEach((cat, cIdx) => {
      const px = padLeft + cIdx * stepX;
      const catText = document.createElementNS("http://www.w3.org/2000/svg", "text");
      catText.setAttribute("x", String(px));
      catText.setAttribute("y", String(h - 6));
      catText.setAttribute("text-anchor", "middle");
      catText.setAttribute("font-size", "10");
      catText.setAttribute("fill", "#6b7280");
      catText.textContent = cat;
      svg.appendChild(catText);
    });
  }

  private renderPieChartSvg(
    svg: SVGSVGElement,
    data: { categories: string[]; series: { name: string; values: number[] }[] },
    w: number,
    h: number,
    isDoughnut = false
  ) {
    const series = data.series[0];
    if (!series || series.values.length === 0) return;

    const values = series.values;
    const total = values.reduce((a, b) => a + b, 0) || 1;

    const cx = Math.min(w * 0.38, 120);
    const cy = h / 2;
    const r = Math.min(cx - 10, cy - 10);

    let startAngle = 0;
    values.forEach((val, idx) => {
      const sliceAngle = (val / total) * 2 * Math.PI;
      const endAngle = startAngle + sliceAngle;

      const x1 = cx + r * Math.cos(startAngle);
      const y1 = cy + r * Math.sin(startAngle);
      const x2 = cx + r * Math.cos(endAngle);
      const y2 = cy + r * Math.sin(endAngle);

      const largeArc = sliceAngle > Math.PI ? 1 : 0;
      const d = `M ${cx} ${cy} L ${x1} ${y1} A ${r} ${r} 0 ${largeArc} 1 ${x2} ${y2} Z`;

      const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
      path.setAttribute("d", d);
      path.setAttribute("fill", PALETTE[idx % PALETTE.length]);
      path.setAttribute("stroke", "#ffffff");
      path.setAttribute("stroke-width", "1.5");

      const titleTag = document.createElementNS("http://www.w3.org/2000/svg", "title");
      const pct = Math.round((val / total) * 100);
      titleTag.textContent = `${data.categories[idx] || "Item"}: ${val} (${pct}%)`;
      path.appendChild(titleTag);

      svg.appendChild(path);
      startAngle = endAngle;

      // Legend entry
      const legX = cx + r + 18;
      const legY = 24 + idx * 18;
      if (legY < h - 10) {
        const legRect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        legRect.setAttribute("x", String(legX));
        legRect.setAttribute("y", String(legY - 8));
        legRect.setAttribute("width", "10");
        legRect.setAttribute("height", "10");
        legRect.setAttribute("fill", PALETTE[idx % PALETTE.length]);
        legRect.setAttribute("rx", "2");
        svg.appendChild(legRect);

        const legText = document.createElementNS("http://www.w3.org/2000/svg", "text");
        legText.setAttribute("x", String(legX + 16));
        legText.setAttribute("y", String(legY));
        legText.setAttribute("font-size", "10");
        legText.setAttribute("fill", "#374151");
        const catName = data.categories[idx] || `Item ${idx + 1}`;
        legText.textContent = `${catName} (${pct}%)`;
        svg.appendChild(legText);
      }
    });

    if (isDoughnut) {
      const hole = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      hole.setAttribute("cx", String(cx));
      hole.setAttribute("cy", String(cy));
      hole.setAttribute("r", String(r * 0.55));
      hole.setAttribute("fill", "#ffffff");
      svg.appendChild(hole);
    }
  }

  private renderScatterChartSvg(
    svg: SVGSVGElement,
    data: { categories: string[]; series: { name: string; values: number[] }[] },
    w: number,
    h: number
  ) {
    const padLeft = 40;
    const padRight = 20;
    const padTop = 20;
    const padBottom = 30;
    const plotW = w - padLeft - padRight;
    const plotH = h - padTop - padBottom;

    // Gridlines
    for (let i = 0; i <= 4; i++) {
      const y = padTop + plotH - (i / 4) * plotH;
      const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
      line.setAttribute("x1", String(padLeft));
      line.setAttribute("y1", String(y));
      line.setAttribute("x2", String(w - padRight));
      line.setAttribute("y2", String(y));
      line.setAttribute("stroke", "#f3f4f6");
      svg.appendChild(line);
    }

    const allValues = data.series.flatMap((s) => s.values);
    const maxVal = Math.max(...allValues, 100);

    data.series.forEach((s, sIdx) => {
      const color = PALETTE[sIdx % PALETTE.length];
      s.values.forEach((val, i) => {
        const cx = padLeft + (i / Math.max(1, s.values.length - 1)) * plotW;
        const cy = padTop + plotH - (val / maxVal) * plotH;
        const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        circle.setAttribute("cx", String(cx));
        circle.setAttribute("cy", String(cy));
        circle.setAttribute("r", "5");
        circle.setAttribute("fill", color);
        circle.setAttribute("stroke", "#ffffff");
        circle.setAttribute("stroke-width", "1");
        svg.appendChild(circle);
      });
    });
  }

  private attachGlobalEvents() {
    window.addEventListener("mousemove", (e: MouseEvent) => {
      if (this.draggingChartId) {
        const chart = this.state.getCharts().find((c) => c.id === this.draggingChartId);
        if (chart) {
          chart.x = Math.max(10, this.initialChartX + (e.clientX - this.dragStartX));
          chart.y = Math.max(10, this.initialChartY + (e.clientY - this.dragStartY));
          const card = this.container.querySelector<HTMLElement>(`[data-chart-id="${chart.id}"]`);
          if (card) {
            card.style.left = `${chart.x}px`;
            card.style.top = `${chart.y}px`;
          }
        }
      } else if (this.resizingChartId) {
        const chart = this.state.getCharts().find((c) => c.id === this.resizingChartId);
        if (chart) {
          chart.width = Math.max(220, this.initialChartW + (e.clientX - this.resizeStartX));
          chart.height = Math.max(160, this.initialChartH + (e.clientY - this.resizeStartY));
          const card = this.container.querySelector<HTMLElement>(`[data-chart-id="${chart.id}"]`);
          if (card) {
            card.style.width = `${chart.width}px`;
            card.style.height = `${chart.height}px`;
            const body = card.querySelector(".calc-chart-body");
            if (body) {
              body.innerHTML = "";
              body.appendChild(this.generateChartSvg(chart));
            }
          }
        }
      }
    });

    window.addEventListener("mouseup", () => {
      if (this.draggingChartId || this.resizingChartId) {
        this.draggingChartId = null;
        this.resizingChartId = null;
      }
    });
  }
}

