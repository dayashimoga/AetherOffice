import { describe, it, expect, vi } from "vitest";
import { WordMailMergeCoordinator } from "../state/WordMailMergeCoordinator";

describe("WordMailMergeCoordinator", () => {
  it("should initialize with empty data source and default LaTeX equation", () => {
    const coordinator = new WordMailMergeCoordinator();
    expect(coordinator.headers).toHaveLength(0);
    expect(coordinator.records).toHaveLength(0);
    expect(coordinator.activeRecordIndex).toBe(0);
    expect(coordinator.getActiveRecord()).toBeNull();
    expect(coordinator.getLatex()).toContain("\\frac");
    expect(coordinator.supportedFormats).toContain("docx");
    expect(coordinator.supportedFormats).toContain("odt");
    expect(coordinator.supportedFormats).toContain("rtf");
  });

  it("should load CSV data source, extracting headers and records with quoted values", () => {
    const coordinator = new WordMailMergeCoordinator();
    const csv = `FirstName,LastName,City,Amount
Alice,"Smith, Jr.",New York,"$1,200"
Bob,Jones,"San Francisco, CA",$500
`;
    coordinator.loadCsv(csv);
    expect(coordinator.headers).toEqual(["FirstName", "LastName", "City", "Amount"]);
    expect(coordinator.totalRecords()).toBe(2);

    const record0 = coordinator.getActiveRecord();
    expect(record0?.FirstName).toBe("Alice");
    expect(record0?.LastName).toBe("Smith, Jr.");
    expect(record0?.City).toBe("New York");
    expect(record0?.Amount).toBe("$1,200");

    coordinator.nextRecord();
    const record1 = coordinator.getActiveRecord();
    expect(record1?.FirstName).toBe("Bob");
    expect(record1?.City).toBe("San Francisco, CA");
  });

  it("should navigate records (next, prev, goToRecord) with strict bounds checking", () => {
    const coordinator = new WordMailMergeCoordinator();
    coordinator.loadCsv("ID,Name\n1,Alpha\n2,Beta\n3,Gamma\n");
    expect(coordinator.activeRecordIndex).toBe(0);

    expect(coordinator.prevRecord()).toBe(false);
    expect(coordinator.nextRecord()).toBe(true);
    expect(coordinator.activeRecordIndex).toBe(1);

    expect(coordinator.nextRecord()).toBe(true);
    expect(coordinator.activeRecordIndex).toBe(2);
    expect(coordinator.nextRecord()).toBe(false); // end of records

    expect(coordinator.goToRecord(1)).toBe(true);
    expect(coordinator.activeRecordIndex).toBe(1);

    expect(coordinator.goToRecord(99)).toBe(false); // out of bounds
    expect(coordinator.activeRecordIndex).toBe(1);
  });

  it("should discover merge fields in template strings with chevrons and MERGEFIELD syntax", () => {
    const template =
      "Hello «Title» «LastName», your order MERGEFIELD OrderID for «Item» is ready.";
    const fields = WordMailMergeCoordinator.findMergeFields(template);
    expect(fields).toEqual(["Item", "LastName", "OrderID", "Title"]);
  });

  it("should resolve template preview using active record", () => {
    const coordinator = new WordMailMergeCoordinator();
    coordinator.loadCsv("Name,Role,Score\nDiana,Lead Architect,98\nClark,Reporter,85\n");

    const template = "Certificate for «Name» (MERGEFIELD Role): Score «Score»/100.";
    const resolved0 = coordinator.resolvePreview(template);
    expect(resolved0).toBe("Certificate for Diana (Lead Architect): Score 98/100.");

    coordinator.nextRecord();
    const resolved1 = coordinator.resolvePreview(template);
    expect(resolved1).toBe("Certificate for Clark (Reporter): Score 85/100.");
  });

  it("should generate complete batch previews across all loaded records", () => {
    const coordinator = new WordMailMergeCoordinator();
    coordinator.loadCsv("User,Status\nUserA,Active\nUserB,Pending\nUserC,Suspended\n");

    const template = "«User»: «Status»";
    const batch = coordinator.generateBatchPreview(template);
    expect(batch).toHaveLength(3);
    expect(batch).toEqual(["UserA: Active", "UserB: Pending", "UserC: Suspended"]);
  });

  it("should manage LaTeX math equation editor string state", () => {
    const coordinator = new WordMailMergeCoordinator();
    expect(coordinator.getLatex()).toContain("\\frac");

    coordinator.setLatex("E = mc^2");
    expect(coordinator.getLatex()).toBe("E = mc^2");
  });

  it("should dispatch subscriber notifications on state mutations", () => {
    const coordinator = new WordMailMergeCoordinator();
    const listener = vi.fn();
    const unsubscribe = coordinator.subscribe(listener);

    coordinator.loadCsv("A,B\n1,2\n");
    expect(listener).toHaveBeenCalledTimes(1);

    coordinator.setLatex("\\int_0^\\infty e^{-x} dx");
    expect(listener).toHaveBeenCalledTimes(2);

    unsubscribe();
    coordinator.setLatex("x + y = z");
    expect(listener).toHaveBeenCalledTimes(2); // no notification after unsubscribe
  });
});
