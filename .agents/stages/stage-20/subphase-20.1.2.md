# Subphase 20.1.2: Document Grounded Citation & Context Retrieval Engine
- **Stage:** 20 | **Phase:** 20.1
- **Target Subsystem:** `copilot-core`
- **Primary Agent Skill:** `@ai-architect`
- **Deliverable:** Retrieval-Augmented Generation (RAG) context assembler, grounded citation verification (linking AI claims directly to exact paragraph spans or cell ranges), and prompt orchestration templates.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 20.1.2
Invoke @ai-architect under GEC-v2.0:
1. Implement context assembler collecting relevant document chunks, selection context, and document metadata.
2. Implement citation grounding engine: verify and highlight the source sentence, table cell, or slide node supporting generated insights.
3. Build specialized prompt templates:
   - Word: "Summarize Section", "Rewrite for Conciseness", "Fix Tone & Grammar".
   - Calc: "Explain Formula", "Synthesize Formula from Description", "Detect Data Anomalies".
   - Slides: "Generate Slide from Outline", "Suggest Layout", "Create Speaker Notes".
4. Add unit and integration tests verifying ground-truth citation extraction, token budget trimming, and hallucination guardrails.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

