"""
generate_truth.py - Drives locally installed Microsoft Office via COM to generate
clean-room JSON input/output test vectors for OpenOfficeRS.
Requires: pip install pywin32
"""
import argparse
import json
import math
from pathlib import Path

def generate_excel_vectors(output_path: Path):
    import win32com.client as win32
    excel = win32.DispatchEx("Excel.Application")
    excel.Visible = False
    excel.DisplayAlerts = False
    wb = excel.Workbooks.Add()
    ws = wb.ActiveSheet

    test_formulas = [
        {"id": "ieee_01", "setup": {"A1": 1.333, "B1": 1.225}, "formula": "=A1-B1-0.108"},
        {"id": "ieee_02", "setup": {"A1": 1.333, "B1": 1.225}, "formula": "=(A1-B1-0.108)"},
        {"id": "fin_pmt_01", "setup": {}, "formula": "=PMT(0.08/12, 10, 10000)"},
        {"id": "stat_round_01", "setup": {}, "formula": "=ROUND(2.15, 1)"}
    ]

    vectors = []
    try:
        for case in test_formulas:
            ws.Cells.Clear()
            for cell_ref, val in case["setup"].items():
                ws.Range(cell_ref).Value = val
            ws.Range("Z1").Formula = case["formula"]
            excel.CalculateFullRebuild()
            res = ws.Range("Z1").Value
            vectors.append({
                "id": case["id"],
                "setup": case["setup"],
                "formula": case["formula"],
                "expected_value": res if not (isinstance(res, float) and math.isnan(res)) else "NaN"
            })
    finally:
        wb.Close(SaveChanges=False)
        excel.Quit()

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps({"app": "excel", "vectors": vectors}, indent=2), encoding="utf-8")
    print(f"[COM Ground Truth] Wrote {len(vectors)} vectors to {output_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="specs/evidence/excel/baseline_vectors.json")
    args = parser.parse_args()
    generate_excel_vectors(Path(args.out))
