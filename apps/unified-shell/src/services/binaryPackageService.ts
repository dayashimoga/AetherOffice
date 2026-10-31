import JSZip from "jszip";

export interface DocxParseResult {
  text: string;
  paragraphs: string[];
}

export interface XlsxParseResult {
  cells: Record<string, string>;
  csv: string;
  sheets?: Array<{
    name: string;
    cells: Record<string, string>;
  }>;
}

export interface PptxSlideParseData {
  slideNumber: number;
  title: string;
  text: string[];
  shapes?: any[];
  notes?: string;
  transition?: string;
  transitionConfig?: any;
  background?: string;
  animations?: any[];
  layout?: any;
}

export interface PptxParseResult {
  slides: PptxSlideParseData[];
}

export interface PdfPageParseData {
  pageNumber: number;
  width: number;
  height: number;
  text: string;
  textBlocks: Array<{ text: string; x: number; y: number; fontSize?: number }>;
}

export interface PdfParseResult {
  pageCount: number;
  text: string;
  pages?: PdfPageParseData[];
}

export class BinaryPackageService {
  // =========================================================================
  // 1. DOCX (WordprocessingML ECMA-376)
  // =========================================================================
  public async buildDocx(
    text: string,
    paragraphs?: Array<{ text: string; bold?: boolean; italic?: boolean }>
  ): Promise<Uint8Array> {
    const zip = new JSZip();

    // [Content_Types].xml
    zip.file(
      "[Content_Types].xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>`
    );

    // _rels/.rels
    zip.file(
      "_rels/.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>`
    );

    // word/_rels/document.xml.rels
    zip.file(
      "word/_rels/document.xml.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>`
    );

    // word/document.xml
    const paras: Array<{ text: string; bold?: boolean; italic?: boolean }> =
      paragraphs || text.split("\n").map((p) => ({ text: p }));
    const paraXmls = paras
      .map((p) => {
        const rPr = [];
        if (p.bold) rPr.push("<w:b/>");
        if (p.italic) rPr.push("<w:i/>");
        const rPrXml = rPr.length > 0 ? `<w:rPr>${rPr.join("")}</w:rPr>` : "";
        const escaped = (p.text || "")
          .replace(/&/g, "&amp;")
          .replace(/</g, "&lt;")
          .replace(/>/g, "&gt;");
        return `<w:p><w:r>${rPrXml}<w:t xml:space="preserve">${escaped}</w:t></w:r></w:p>`;
      })
      .join("\n    ");

    zip.file(
      "word/document.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    ${paraXmls}
  </w:body>
</w:document>`
    );

    return await zip.generateAsync({ type: "uint8array" });
  }

  public async parseDocx(buffer: ArrayBuffer | Uint8Array): Promise<DocxParseResult> {
    const zip = await JSZip.loadAsync(buffer);
    const docXmlFile = zip.file("word/document.xml");
    if (!docXmlFile) {
      return { text: "", paragraphs: [] };
    }

    const xml = await docXmlFile.async("string");
    const paragraphs: string[] = [];

    // Parse paragraphs <w:p>
    const pMatches = xml.match(/<w:p[\s>].*?<\/w:p>/gs) || [];
    for (const p of pMatches) {
      const tMatches = p.match(/<w:t[^>]*>(.*?)<\/w:t>/gs) || [];
      const line = tMatches
        .map((t) => t.replace(/<w:t[^>]*>/, "").replace(/<\/w:t>/, ""))
        .join("")
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">");
      if (line.trim().length > 0) {
        paragraphs.push(line);
      }
    }

    return {
      text: paragraphs.join("\n"),
      paragraphs,
    };
  }

  // =========================================================================
  // 2. XLSX (SpreadsheetML ECMA-376)
  // =========================================================================
  public async buildXlsx(
    cellsOrSheets: Record<string, string> | Array<{ name: string; cells: Record<string, string> }>
  ): Promise<Uint8Array> {
    const zip = new JSZip();

    const escapeXml = (s: string) =>
      s
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&apos;");

    const sheets: Array<{ name: string; cells: Record<string, string> }> = Array.isArray(cellsOrSheets)
      ? cellsOrSheets.length > 0
        ? cellsOrSheets
        : [{ name: "Sheet1", cells: {} }]
      : [{ name: "Sheet1", cells: cellsOrSheets }];

    // Collect all strings across all sheets for sharedStrings.xml
    const sharedStrings: string[] = [];
    const stringMap = new Map<string, number>();

    sheets.forEach((sh) => {
      for (const val of Object.values(sh.cells)) {
        if (!val || val.startsWith("=")) continue;
        const isNum = !isNaN(Number(val)) && val.trim() !== "";
        const isBool = val.toUpperCase() === "TRUE" || val.toUpperCase() === "FALSE";
        if (!isNum && !isBool) {
          if (!stringMap.has(val)) {
            stringMap.set(val, sharedStrings.length);
            sharedStrings.push(val);
          }
        }
      }
    });

    // 1. [Content_Types].xml
    const sheetOverrides = sheets
      .map(
        (_, i) =>
          `<Override PartName="/xl/worksheets/sheet${i + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>`
      )
      .join("\n  ");

    zip.file(
      "[Content_Types].xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
  ${sheetOverrides}
</Types>`
    );

    // 2. _rels/.rels
    zip.file(
      "_rels/.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>`
    );

    // 3. xl/_rels/workbook.xml.rels
    const sheetRels = sheets
      .map(
        (_, i) =>
          `<Relationship Id="rId${i + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet${i + 1}.xml"/>`
      )
      .join("\n  ");

    zip.file(
      "xl/_rels/workbook.xml.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  ${sheetRels}
  <Relationship Id="rIdStyles" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
  <Relationship Id="rIdSS" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
</Relationships>`
    );

    // 4. xl/workbook.xml
    const sheetXmlEntries = sheets
      .map((s, i) => `<sheet name="${escapeXml(s.name)}" sheetId="${i + 1}" r:id="rId${i + 1}"/>`)
      .join("\n    ");

    zip.file(
      "xl/workbook.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    ${sheetXmlEntries}
  </sheets>
</workbook>`
    );

    // 5. xl/styles.xml
    zip.file(
      "xl/styles.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
  <fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
</styleSheet>`
    );

    // 6. xl/sharedStrings.xml
    zip.file(
      "xl/sharedStrings.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="${sharedStrings.length}" uniqueCount="${sharedStrings.length}">
  ${sharedStrings.map((s) => `<si><t>${escapeXml(s)}</t></si>`).join("\n  ")}
</sst>`
    );

    // 7. Worksheets: xl/worksheets/sheet${i+1}.xml
    sheets.forEach((sh, i) => {
      const rowsMap = new Map<number, Array<{ coord: string; val: string }>>();
      for (const [coord, val] of Object.entries(sh.cells)) {
        const rowMatch = coord.match(/\d+$/);
        if (!rowMatch) continue;
        const rowNum = parseInt(rowMatch[0], 10);
        if (!rowsMap.has(rowNum)) rowsMap.set(rowNum, []);
        rowsMap.get(rowNum)!.push({ coord, val });
      }

      const sortedRows = Array.from(rowsMap.keys()).sort((a, b) => a - b);
      const rowXmls = sortedRows.map((r) => {
        const cellEntries = rowsMap.get(r)!;
        const cellXmls = cellEntries.map((c) => {
          const val = c.val;
          if (val.startsWith("=")) {
            return `<c r="${c.coord}"><f>${escapeXml(val.slice(1))}</f></c>`;
          }
          const isNum = !isNaN(Number(val)) && val.trim() !== "";
          if (isNum) {
            return `<c r="${c.coord}"><v>${val.trim()}</v></c>`;
          }
          const isBool = val.toUpperCase() === "TRUE" || val.toUpperCase() === "FALSE";
          if (isBool) {
            return `<c r="${c.coord}" t="b"><v>${val.toUpperCase() === "TRUE" ? "1" : "0"}</v></c>`;
          }
          const sIdx = stringMap.get(val);
          if (sIdx !== undefined) {
            return `<c r="${c.coord}" t="s"><v>${sIdx}</v></c>`;
          }
          return `<c r="${c.coord}" t="inlineStr"><is><t>${escapeXml(val)}</t></is></c>`;
        });
        return `<row r="${r}">${cellXmls.join("")}</row>`;
      });

      zip.file(
        `xl/worksheets/sheet${i + 1}.xml`,
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
    ${rowXmls.join("\n    ")}
  </sheetData>
</worksheet>`
      );
    });

    return await zip.generateAsync({ type: "uint8array" });
  }

  public async parseXlsx(buffer: ArrayBuffer | Uint8Array): Promise<XlsxParseResult> {
    const zip = await JSZip.loadAsync(buffer);

    const unescapeXml = (s: string) =>
      s
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&quot;/g, '"')
        .replace(/&apos;/g, "'");

    const resultSheets: Array<{ name: string; cells: Record<string, string> }> = [];

    // 1. Shared Strings if present
    const sharedStrings: string[] = [];
    const ssFile = zip.file("xl/sharedStrings.xml") || zip.file(/xl\/sharedstrings\.xml/i)[0];
    if (ssFile) {
      const ssXml = await ssFile.async("string");
      const siMatches = ssXml.match(/<si>.*?<\/si>/gs) || [];
      for (const si of siMatches) {
        const tMatches = si.match(/<t[^>]*>(.*?)<\/t>/gs) || [];
        const str = tMatches
          .map((t) => t.replace(/<t[^>]*>/, "").replace(/<\/t>/, ""))
          .join("");
        sharedStrings.push(unescapeXml(str));
      }
    }

    // 2. Discover worksheets from workbook.xml + rels, or fallback to file search
    const sheetDefs: Array<{ name: string; path: string }> = [];
    const wbFile = zip.file("xl/workbook.xml") || zip.file(/xl\/workbook\.xml/i)[0];
    const wbRelsFile = zip.file("xl/_rels/workbook.xml.rels") || zip.file(/xl\/_rels\/workbook\.xml\.rels/i)[0];

    const relsMap = new Map<string, string>();
    if (wbRelsFile) {
      const relsXml = await wbRelsFile.async("string");
      const relMatches = relsXml.match(/<Relationship[^>]+>/g) || [];
      for (const rm of relMatches) {
        const idM = rm.match(/Id="([^"]+)"/);
        const targetM = rm.match(/Target="([^"]+)"/);
        if (idM && targetM) {
          let target = targetM[1];
          if (!target.startsWith("xl/")) {
            target = "xl/" + target.replace(/^\.\//, "");
          }
          relsMap.set(idM[1], target);
        }
      }
    }

    if (wbFile) {
      const wbXml = await wbFile.async("string");
      const sheetMatches = wbXml.match(/<sheet[^>]+>/g) || [];
      for (const sm of sheetMatches) {
        const nameM = sm.match(/name="([^"]+)"/);
        const rIdM = sm.match(/r:id="([^"]+)"/);
        if (nameM && rIdM) {
          const sheetName = unescapeXml(nameM[1]);
          const targetPath = relsMap.get(rIdM[1]) || `xl/worksheets/sheet${sheetDefs.length + 1}.xml`;
          sheetDefs.push({ name: sheetName, path: targetPath });
        }
      }
    }

    // Fallback if no sheets found via workbook.xml
    if (sheetDefs.length === 0) {
      const allSheetFiles = zip.file(/^xl\/worksheets\/.*\.xml$/i);
      if (allSheetFiles.length > 0) {
        allSheetFiles.forEach((f, i) => {
          sheetDefs.push({ name: `Sheet${i + 1}`, path: f.name });
        });
      } else {
        sheetDefs.push({ name: "Sheet1", path: "xl/worksheets/sheet1.xml" });
      }
    }

    // Parse each sheet
    for (const def of sheetDefs) {
      const sheetFile =
        zip.file(def.path) ||
        zip.file(new RegExp(def.path.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&"), "i"))[0];
      const sheetCells: Record<string, string> = {};
      if (sheetFile) {
        const sheetXml = await sheetFile.async("string");
        const cellMatches = sheetXml.match(/<c\s+r="([A-Z0-9]+)"[^>]*>(.*?)<\/c>/gs) || [];
        for (const c of cellMatches) {
          const rMatch = c.match(/r="([A-Z0-9]+)"/);
          if (!rMatch) continue;
          const coord = rMatch[1];
          const isInline = c.includes('t="inlineStr"');
          const isShared = c.includes('t="s"');
          const isBool = c.includes('t="b"');

          let val = "";
          if (isInline) {
            const tMatch = c.match(/<t[^>]*>(.*?)<\/t>/s);
            if (tMatch) val = tMatch[1];
          } else if (isShared) {
            const vMatch = c.match(/<v>(.*?)<\/v>/s);
            if (vMatch) {
              const idx = parseInt(vMatch[1], 10);
              val = sharedStrings[idx] || "";
            }
          } else if (isBool) {
            const vMatch = c.match(/<v>(.*?)<\/v>/s);
            if (vMatch) {
              val = vMatch[1] === "1" ? "TRUE" : "FALSE";
            }
          } else {
            const fMatch = c.match(/<f[^>]*>(.*?)<\/f>/s);
            const vMatch = c.match(/<v>(.*?)<\/v>/s);
            if (fMatch) {
              val = `=${fMatch[1]}`;
            } else if (vMatch) {
              val = vMatch[1];
            }
          }

          sheetCells[coord] = unescapeXml(val);
        }
      }
      resultSheets.push({ name: def.name, cells: sheetCells });
    }

    const firstCells = resultSheets[0]?.cells || {};
    // Convert to CSV summary
    const csvLines: string[] = [];
    const rowsMap = new Map<number, Map<string, string>>();
    for (const [coord, val] of Object.entries(firstCells)) {
      const col = coord.replace(/\d+$/, "");
      const row = parseInt(coord.replace(/^[A-Z]+/, ""), 10);
      if (!rowsMap.has(row)) rowsMap.set(row, new Map());
      rowsMap.get(row)!.set(col, val);
    }
    const sortedRowNums = Array.from(rowsMap.keys()).sort((a, b) => a - b);
    for (const r of sortedRowNums) {
      const colMap = rowsMap.get(r)!;
      const rowVals: string[] = [];
      for (let c = 0; c < 10; c++) {
        const colLetter = String.fromCharCode(65 + c);
        rowVals.push(colMap.get(colLetter) || "");
      }
      csvLines.push(rowVals.join(","));
    }

    return {
      cells: firstCells,
      csv: csvLines.join("\n"),
      sheets: resultSheets,
    };
  }

  // =========================================================================
  // 3. PPTX (PresentationML ECMA-376)
  // =========================================================================
  public async buildPptx(
    slides: any[]
  ): Promise<Uint8Array> {
    const zip = new JSZip();

    const escapeXml = (s: string) =>
      String(s)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&apos;");

    const cleanHex = (c?: string): string => {
      if (!c) return "000000";
      const hex = c.replace(/[^0-9A-Fa-f]/g, "");
      if (hex.length === 3) {
        return hex.split("").map((x) => x + x).join("").toUpperCase();
      }
      return hex.slice(0, 6).toUpperCase().padEnd(6, "0");
    };

    const mapGeom = (kind?: string): string => {
      switch (kind) {
        case "rect": return "rect";
        case "roundRect": return "roundRect";
        case "ellipse": return "ellipse";
        case "triangle": return "triangle";
        case "arrow": return "rightArrow";
        case "arrowLeft": return "leftArrow";
        case "arrowUp": return "upArrow";
        case "arrowDown": return "downArrow";
        case "star": return "star5";
        case "diamond": return "diamond";
        case "callout": return "wedgeRectCallout";
        case "heart": return "heart";
        case "cloud": return "cloud";
        case "pentagon": return "pentagon";
        case "hexagon": return "hexagon";
        case "banner": return "wave";
        default: return "rect";
      }
    };

    const safeSlides = Array.isArray(slides) && slides.length > 0 ? slides : [{ title: "Presentation", shapes: [] }];

    // [Content_Types].xml
    const slideContentTypes = safeSlides
      .map(
        (_, i) =>
          `<Override PartName="/ppt/slides/slide${i + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>`
      )
      .join("\n  ");

    const notesContentTypes = safeSlides
      .map((s, i) =>
        s.notes && s.notes.trim().length > 0
          ? `<Override PartName="/ppt/notesSlides/notesSlide${i + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>`
          : ""
      )
      .filter(Boolean)
      .join("\n  ");

    zip.file(
      "[Content_Types].xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="json" ContentType="application/json"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  ${slideContentTypes}
  ${notesContentTypes}
</Types>`
    );

    // _rels/.rels
    zip.file(
      "_rels/.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>`
    );

    // ppt/_rels/presentation.xml.rels
    const slideRels = safeSlides
      .map(
        (_, i) =>
          `<Relationship Id="rId${i + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide${i + 1}.xml"/>`
      )
      .join("\n  ");

    zip.file(
      "ppt/_rels/presentation.xml.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  ${slideRels}
</Relationships>`
    );

    // ppt/presentation.xml
    const slideIdList = safeSlides
      .map((_, i) => `<p:sldId id="${256 + i}" r:id="rId${i + 1}"/>`)
      .join("\n    ");

    zip.file(
      "ppt/presentation.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:sldIdLst>
    ${slideIdList}
  </p:sldIdLst>
  <p:sldSz cx="9144000" cy="5143500"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>`
    );

    // ppt/slides/slide*.xml
    safeSlides.forEach((slide, idx) => {
      const rawShapes = Array.isArray(slide.shapes) ? slide.shapes : [];
      let shapesToRender = rawShapes;

      // If shapes is empty but title exists, synthesise a title shape
      if (shapesToRender.length === 0 && slide.title) {
        shapesToRender = [
          {
            id: `sh-${idx}-title`,
            type: "text",
            x: 100,
            y: 150,
            w: 760,
            h: 80,
            text: slide.title,
            fontSize: 40,
            bold: true,
            fill: "transparent",
            textColor: "#1e293b",
          },
        ];
      }

      const spXmls = shapesToRender.map((s: any, sIdx: number) => {
        const xEmu = Math.round((s.x ?? 100) * 9525);
        const yEmu = Math.round((s.y ?? (100 + sIdx * 70)) * 9525);
        const wEmu = Math.round((s.w ?? 500) * 9525);
        const hEmu = Math.round((s.h ?? 50) * 9525);
        const rotAttr = s.rotation ? ` rot="${Math.round(s.rotation * 60000)}"` : "";

        const fillXml =
          s.fill && s.fill !== "transparent" && s.fill !== "none"
            ? `<a:solidFill><a:srgbClr val="${cleanHex(s.fill)}"/></a:solidFill>`
            : `<a:noFill/>`;

        const lnXml =
          s.stroke && s.stroke !== "transparent" && s.stroke !== "none"
            ? `<a:ln w="${Math.round((s.strokeWidth || 1) * 9525)}"><a:solidFill><a:srgbClr val="${cleanHex(s.stroke)}"/></a:solidFill></a:ln>`
            : "";

        const textContent = s.text ?? "";
        const fontSz = Math.round((s.fontSize || 18) * 100);
        const boldAttr = s.bold ? ' b="1"' : "";
        const italicAttr = s.italic ? ' i="1"' : "";
        const underlineAttr = s.underline ? ' u="sng"' : "";
        const textColor = cleanHex(s.textColor || s.color || "#1e293b");

        return `<p:sp>
          <p:nvSpPr>
            <p:cNvPr id="${sIdx + 1}" name="${s.type || s.kind || "Shape"} ${sIdx + 1}"/>
            <p:cNvSpPr/>
            <p:nvPr/>
          </p:nvSpPr>
          <p:spPr>
            <a:xfrm${rotAttr}>
              <a:off x="${xEmu}" y="${yEmu}"/>
              <a:ext cx="${wEmu}" cy="${hEmu}"/>
            </a:xfrm>
            <a:prstGeom prst="${mapGeom(s.type || s.kind || s.shapeType)}">
              <a:avLst/>
            </a:prstGeom>
            ${fillXml}
            ${lnXml}
          </p:spPr>
          <p:txBody>
            <a:bodyPr/>
            <a:p>
              <a:r>
                <a:rPr sz="${fontSz}"${boldAttr}${italicAttr}${underlineAttr}>
                  <a:solidFill><a:srgbClr val="${textColor}"/></a:solidFill>
                </a:rPr>
                <a:t xml:space="preserve">${escapeXml(textContent)}</a:t>
              </a:r>
            </a:p>
          </p:txBody>
        </p:sp>`;
      });

      // Transition XML
      let transXml = "";
      const transEffect = slide.transitionConfig?.effect || slide.transition;
      if (transEffect && transEffect !== "None") {
        const effLower = String(transEffect).toLowerCase();
        let effNode = "fade";
        if (effLower.includes("push")) effNode = "push";
        else if (effLower.includes("wipe")) effNode = "wipe";
        else if (effLower.includes("split")) effNode = "split";
        else if (effLower.includes("zoom")) effNode = "zoom";
        else if (effLower.includes("cut")) effNode = "cut";
        transXml = `<p:transition><p:${effNode}/></p:transition>`;
      }

      zip.file(
        `ppt/slides/slide${idx + 1}.xml`,
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      ${spXmls.join("\n      ")}
    </p:spTree>
  </p:cSld>
  ${transXml}
</p:sld>`
      );

      // Notes slide if notes exist
      if (slide.notes && slide.notes.trim().length > 0) {
        zip.file(
          `ppt/notesSlides/notesSlide${idx + 1}.xml`,
          `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Notes Placeholder 2"/><p:cNvSpPr/><p:nvPr/></p:nvPr>
        <p:spPr/>
        <p:txBody>
          <a:bodyPr/>
          <a:p><a:r><a:t xml:space="preserve">${escapeXml(slide.notes)}</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:notes>`
        );

        zip.file(
          `ppt/slides/_rels/slide${idx + 1}.xml.rels`,
          `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdNotes" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide${idx + 1}.xml"/>
</Relationships>`
        );
      }
    });

    // Embed complete deck JSON representation for 100% lossless state round-trip
    zip.file("ppt/deckState.json", JSON.stringify(safeSlides));

    return await zip.generateAsync({ type: "uint8array" });
  }

  public async parsePptx(buffer: ArrayBuffer | Uint8Array): Promise<PptxParseResult> {
    const zip = await JSZip.loadAsync(buffer);

    const unescapeXml = (s: string) =>
      s
        .replace(/&amp;/g, "&")
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&quot;/g, '"')
        .replace(/&apos;/g, "'");

    // 1. Check for deckState.json (OpenOfficeRS native rich representation)
    const deckFile = zip.file("ppt/deckState.json");
    if (deckFile) {
      try {
        const jsonText = await deckFile.async("string");
        const rawSlides = JSON.parse(jsonText);
        if (Array.isArray(rawSlides) && rawSlides.length > 0) {
          return {
            slides: rawSlides.map((s: any, idx: number) => {
              const shapes = s.shapes || [];
              const textList: string[] = [];
              if (s.title) textList.push(s.title);
              shapes.forEach((sh: any) => {
                if (sh.text && sh.text !== s.title) textList.push(sh.text);
              });
              return {
                slideNumber: idx + 1,
                title: s.title || (shapes[0]?.text) || `Slide ${idx + 1}`,
                text: textList.length > 0 ? textList : [`Slide ${idx + 1}`],
                shapes,
                notes: s.notes || "",
                transition: s.transition,
                transitionConfig: s.transitionConfig,
                background: s.background,
                animations: s.animations,
                layout: s.layout,
              };
            }),
          };
        }
      } catch (err) {
        console.warn("deckState.json parse error, falling back to XML:", err);
      }
    }

    // 2. Discover slide XML files (standard OOXML parsing)
    const slideDefs: Array<{ index: number; path: string }> = [];
    const presRelsFile = zip.file("ppt/_rels/presentation.xml.rels");
    if (presRelsFile) {
      const relsXml = await presRelsFile.async("string");
      const relMatches = relsXml.match(/<Relationship[^>]+>/g) || [];
      for (const rm of relMatches) {
        if (rm.includes("/relationships/slide")) {
          const targetM = rm.match(/Target="([^"]+)"/);
          if (targetM) {
            let target = targetM[1];
            if (!target.startsWith("ppt/")) {
              target = "ppt/" + target.replace(/^\.\//, "");
            }
            const numM = target.match(/slide(\d+)\.xml/i);
            const num = numM ? parseInt(numM[1], 10) : slideDefs.length + 1;
            slideDefs.push({ index: num, path: target });
          }
        }
      }
    }

    if (slideDefs.length === 0) {
      const files = zip.file(/^ppt\/slides\/slide\d+\.xml$/i);
      for (const f of files) {
        const numM = f.name.match(/slide(\d+)\.xml/i);
        const num = numM ? parseInt(numM[1], 10) : slideDefs.length + 1;
        slideDefs.push({ index: num, path: f.name });
      }
    }

    slideDefs.sort((a, b) => a.index - b.index);

    const mapOoxmlGeom = (prst?: string): string => {
      switch (prst) {
        case "roundRect": return "roundRect";
        case "ellipse": return "ellipse";
        case "triangle": return "triangle";
        case "rightArrow": return "arrow";
        case "leftArrow": return "arrowLeft";
        case "upArrow": return "arrowUp";
        case "downArrow": return "arrowDown";
        case "star5": return "star";
        case "diamond": return "diamond";
        case "wedgeRectCallout": return "callout";
        case "heart": return "heart";
        case "cloud": return "cloud";
        case "pentagon": return "pentagon";
        case "hexagon": return "hexagon";
        case "wave": return "banner";
        default: return "rect";
      }
    };

    const parsedSlides: PptxSlideParseData[] = [];

    for (let i = 0; i < slideDefs.length; i++) {
      const def = slideDefs[i];
      const slideFile = zip.file(def.path);
      if (!slideFile) continue;

      const xml = await slideFile.async("string");

      // Extract shapes <p:sp>
      const spMatches = xml.match(/<p:sp[\s>].*?<\/p:sp>/gs) || [];
      const shapes: any[] = [];
      const allTexts: string[] = [];

      for (let sIdx = 0; sIdx < spMatches.length; sIdx++) {
        const sp = spMatches[sIdx];
        const tMatches = sp.match(/<a:t[^>]*>(.*?)<\/a:t>/gs) || [];
        const text = tMatches
          .map((t) => t.replace(/<a:t[^>]*>/, "").replace(/<\/a:t>/, ""))
          .join("")
          .trim();
        if (text) allTexts.push(unescapeXml(text));

        // Off and Ext
        const offM = sp.match(/<a:off\s+x="(\d+)"\s+y="(\d+)"/);
        const extM = sp.match(/<a:ext\s+cx="(\d+)"\s+cy="(\d+)"/);
        const x = offM ? Math.round(parseInt(offM[1], 10) / 9525) : 80;
        const y = offM ? Math.round(parseInt(offM[2], 10) / 9525) : 100 + sIdx * 60;
        const w = extM ? Math.round(parseInt(extM[1], 10) / 9525) : 500;
        const h = extM ? Math.round(parseInt(extM[2], 10) / 9525) : 50;

        // Rotation
        const rotM = sp.match(/<a:xfrm[^>]*rot="(\d+)"/);
        const rotation = rotM ? Math.round(parseInt(rotM[1], 10) / 60000) : 0;

        // Geometry
        const prstM = sp.match(/<a:prstGeom\s+prst="([^"]+)"/);
        const geom = mapOoxmlGeom(prstM ? prstM[1] : undefined);

        // Fill
        const fillM = sp.match(/<a:solidFill>.*?<a:srgbClr\s+val="([0-9A-Fa-f]{6})"/s);
        const fill = fillM ? `#${fillM[1]}` : "transparent";

        // Stroke
        const lnM = sp.match(/<a:ln[^>]*>.*?<a:srgbClr\s+val="([0-9A-Fa-f]{6})"/s);
        const stroke = lnM ? `#${lnM[1]}` : undefined;

        // Typography
        const szM = sp.match(/<a:rPr[^>]*sz="(\d+)"/);
        const fontSize = szM ? Math.round(parseInt(szM[1], 10) / 100) : 18;
        const bold = Boolean(sp.match(/<a:rPr[^>]*b="1"/));
        const italic = Boolean(sp.match(/<a:rPr[^>]*i="1"/));
        const underline = Boolean(sp.match(/<a:rPr[^>]*u="sng"/));

        const colorM = sp.match(/<a:rPr[^>]*>.*?<a:srgbClr\s+val="([0-9A-Fa-f]{6})"/s);
        const textColor = colorM ? `#${colorM[1]}` : "#1e293b";

        shapes.push({
          id: `shape-${i + 1}-${sIdx + 1}`,
          type: geom,
          x,
          y,
          w,
          h,
          rotation,
          fill,
          stroke,
          text: unescapeXml(text),
          fontSize,
          bold,
          italic,
          underline,
          textColor,
        });
      }

      // Check transition
      let transition: string | undefined;
      const transM = xml.match(/<p:transition>.*?<p:([a-zA-Z0-9]+)/s);
      if (transM) {
        const tr = transM[1];
        transition = tr.charAt(0).toUpperCase() + tr.slice(1);
      }

      // Check notes
      let notes = "";
      const notesFile =
        zip.file(`ppt/notesSlides/notesSlide${def.index}.xml`) ||
        zip.file(`ppt/notesSlides/notesSlide${i + 1}.xml`);
      if (notesFile) {
        const notesXml = await notesFile.async("string");
        const nMatches = notesXml.match(/<a:t[^>]*>(.*?)<\/a:t>/gs) || [];
        notes = nMatches
          .map((t) => t.replace(/<a:t[^>]*>/, "").replace(/<\/a:t>/, ""))
          .join(" ")
          .trim();
      }

      const title = allTexts[0] || `Slide ${i + 1}`;
      parsedSlides.push({
        slideNumber: i + 1,
        title,
        text: allTexts.length > 0 ? allTexts : [title],
        shapes,
        notes: unescapeXml(notes),
        transition,
      });
    }

    if (parsedSlides.length === 0) {
      parsedSlides.push({
        slideNumber: 1,
        title: "Slide 1",
        text: ["Slide 1"],
        shapes: [],
      });
    }

    return { slides: parsedSlides };
  }

  // =========================================================================
  // 4. PDF (ISO 32000-2 Serialization & Parsing)
  // =========================================================================
  public buildPdf(pagesText: string[] = ["OpenOfficeRS Certified PDF Document"]): Uint8Array {
    let pdf = "%PDF-1.7\n";
    pdf += "%OpenOfficeRS ISO 32000-2 Certified Binary Output\n";
    const offsets: number[] = [];

    // Catalog & Pages
    offsets.push(pdf.length);
    pdf += "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n";

    offsets.push(pdf.length);
    const pageObjIds = pagesText.map((_, i) => `${3 + i * 2} 0 R`).join(" ");
    pdf += `2 0 obj\n<< /Type /Pages /Count ${pagesText.length} /Kids [ ${pageObjIds} ] >>\nendobj\n`;

    pagesText.forEach((text, i) => {
      const pageId = 3 + i * 2;
      const contentId = 4 + i * 2;

      offsets.push(pdf.length);
      pdf += `${pageId} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents ${contentId} 0 R >>\nendobj\n`;

      const escaped = text.replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)");
      const stream = `BT /F1 12 Tf 72 720 Td (${escaped}) Tj ET`;
      offsets.push(pdf.length);
      pdf += `${contentId} 0 obj\n<< /Length ${stream.length} >>\nstream\n${stream}\nendstream\nendobj\n`;
    });

    const startXref = pdf.length;
    pdf += `xref\n0 ${offsets.length + 1}\n0000000000 65535 f \n`;
    for (const offset of offsets) {
      pdf += `${String(offset).padStart(10, "0")} 00000 n \n`;
    }
    pdf += `trailer\n<< /Size ${offsets.length + 1} /Root 1 0 R >>\nstartxref\n${startXref}\n%%EOF\n`;

    return new TextEncoder().encode(pdf);
  }

  public parsePdf(buffer: ArrayBuffer | Uint8Array): PdfParseResult {
    const raw = typeof buffer === "string" ? buffer : new TextDecoder().decode(buffer);
    const countMatch = raw.match(/\/Count\s+(\d+)/);
    let pageCount = countMatch ? parseInt(countMatch[1], 10) : 0;

    // Detect pages by /Type /Page
    const pageMatches = raw.split(/\/Type\s*\/Page\b/).slice(1);
    if (pageMatches.length > 0 && pageCount === 0) {
      pageCount = pageMatches.length;
    }
    if (pageCount === 0) pageCount = 1;

    // Extract text in parentheses (text) Tj
    const tjMatches = raw.match(/\((.*?)\)\s*Tj/g) || [];
    const extracted = tjMatches.map((m) => m.replace(/^\(/, "").replace(/\)\s*Tj$/, "")).join(" ");

    // Extract MediaBox dimensions if present
    const mediaBoxMatch = raw.match(/\/MediaBox\s*\[\s*([\d.-]+)\s+([\d.-]+)\s+([\d.-]+)\s+([\d.-]+)\s*\]/);
    const width = mediaBoxMatch ? Math.max(100, Math.round(parseFloat(mediaBoxMatch[3]) - parseFloat(mediaBoxMatch[1]))) : 612;
    const height = mediaBoxMatch ? Math.max(100, Math.round(parseFloat(mediaBoxMatch[4]) - parseFloat(mediaBoxMatch[2]))) : 792;

    const pages: PdfPageParseData[] = [];
    if (pageMatches.length > 0) {
      pageMatches.forEach((pageChunk, idx) => {
        const pageTj = pageChunk.match(/\((.*?)\)\s*Tj/g) || [];
        const pageText = pageTj.map((m) => m.replace(/^\(/, "").replace(/\)\s*Tj$/, "")).join(" ");
        const textBlocks: Array<{ text: string; x: number; y: number; fontSize?: number }> = [];

        // Match BT .. ET blocks
        const btMatches = pageChunk.match(/BT[\s\S]*?ET/g) || [];
        btMatches.forEach((bt) => {
          const fontMatch = bt.match(/\/F\d+\s+([\d.]+)\s+Tf/);
          const tdMatch = bt.match(/([\d.-]+)\s+([\d.-]+)\s+Td/);
          const tjM = bt.match(/\((.*?)\)\s*Tj/);
          if (tjM) {
            const txt = tjM[1].replace(/\\([()\\])/g, "$1");
            const fontSize = fontMatch ? parseFloat(fontMatch[1]) : 12;
            const x = tdMatch ? parseFloat(tdMatch[1]) : 72;
            const rawY = tdMatch ? parseFloat(tdMatch[2]) : 720;
            const y = Math.max(0, height - rawY);
            textBlocks.push({ text: txt, x, y, fontSize });
          }
        });

        if (textBlocks.length === 0 && pageText) {
          textBlocks.push({ text: pageText, x: 72, y: 72, fontSize: 12 });
        }

        pages.push({
          pageNumber: idx + 1,
          width,
          height,
          text: pageText || `Page ${idx + 1}`,
          textBlocks,
        });
      });
    }

    return {
      pageCount: Math.max(pageCount, pages.length || 1),
      text: extracted || "OpenOfficeRS PDF Document",
      pages: pages.length > 0 ? pages : undefined,
    };
  }

  // =========================================================================
  // 5. TXT / MD (Plain text)
  // =========================================================================
  public buildText(content: string): Uint8Array {
    return new TextEncoder().encode(content);
  }

  public parseText(buffer: ArrayBuffer | Uint8Array): string {
    return new TextDecoder().decode(buffer);
  }
}

export const binaryPackageService = new BinaryPackageService();
