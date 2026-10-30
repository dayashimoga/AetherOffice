# Subphase 20.1.1: Local On-Device Embedding & Semantic Chunking Engine
- **Stage:** 20 | **Phase:** 20.1
- **Target Subsystem:** `copilot-core`
- **Primary Agent Skill:** `@ai-architect`
- **Deliverable:** Zero-telemetry on-device vector embedding generator, hierarchical document chunker (paragraphs, headings, table cells, slide shapes), and local cosine similarity index.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 20.1.1
Invoke @ai-architect under GEC-v2.0:
1. Create new crate `crates/copilot-core` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Implement semantic document chunker preserving structural context (document title, headings, page/sheet numbers, row indices).
3. Implement lightweight, quantized on-device embedding tokenizer and vector generator (e.g. mini-embeddings / dense projection).
4. Implement in-memory vector index with SIMD-accelerated cosine distance and top-k nearest neighbor ranking.
5. Add unit and integration tests verifying embedding consistency, semantic retrieval precision, and incremental index updates on document edits.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

