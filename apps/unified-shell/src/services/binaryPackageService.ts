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

export interface PptxParseResult {
  slides: Array<{ slideNumber: number; title: string; text: string[] }>;
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
    slides: Array<{ title: string; shapes?: Array<{ text?: string }> }>
  ): Promise<Uint8Array> {
    const zip = new JSZip();

    zip.file(
      "[Content_Types].xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  ${slides.map((_, i) => `<Override PartName="/ppt/slides/slide${i + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>`).join("\n  ")}
</Types>`
    );

    zip.file(
      "_rels/.rels",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>`
    );

    const slideRels = slides
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

    const slideIdList = slides
      .map((_, i) => `<p:sldId id="${256 + i}" r:id="rId${i + 1}"/>`)
      .join("\n    ");

    zip.file(
      "ppt/presentation.xml",
      `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    ${slideIdList}
  </p:sldIdLst>
</p:presentation>`
    );

    slides.forEach((slide, idx) => {
      const texts = [slide.title, ...(slide.shapes?.map((s) => s.text || "") || [])].filter(Boolean);
      const spXmls = texts.map((t, sIdx) => {
        const escaped = t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
        return `<p:sp>
          <p:nvSpPr><p:cNvPr id="${sIdx + 1}" name="Shape ${sIdx + 1}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
          <p:txBody><a:bodyPr/><a:p><a:r><a:t>${escaped}</a:t></a:r></a:p></p:txBody>
        </p:sp>`;
      });

      zip.file(
        `ppt/slides/slide${idx + 1}.xml`,
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      ${spXmls.join("\n      ")}
    </p:spTree>
  </p:cSld>
</p:sld>`
      );
    });

    return await zip.generateAsync({ type: "uint8array" });
  }

  public async parsePptx(buffer: ArrayBuffer | Uint8Array): Promise<PptxParseResult> {
    const zip = await JSZip.loadAsync(buffer);
    const slides: Array<{ slideNumber: number; title: string; text: string[] }> = [];

    let slideIndex = 1;
    while (true) {
      const slideFile = zip.file(`ppt/slides/slide${slideIndex}.xml`);
      if (!slideFile) break;

      const xml = await slideFile.async("string");
      const tMatches = xml.match(/<a:t>(.*?)<\/a:t>/gs) || [];
      const texts = tMatches.map((t) =>
        t
          .replace(/<a:t>/, "")
          .replace(/<\/a:t>/, "")
          .replace(/&amp;/g, "&")
          .replace(/&lt;/g, "<")
          .replace(/&gt;/g, ">")
      );

      slides.push({
        slideNumber: slideIndex,
        title: texts[0] || `Slide ${slideIndex}`,
        text: texts,
      });

      slideIndex++;
    }

    return { slides };
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
