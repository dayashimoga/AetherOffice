# Subphase 19.1.1: DrawingML Embedded Audio/Video Media Engine
- **Stage:** 19 | **Phase:** 19.1
- **Target Subsystem:** `ppt-engine` & `unified-shell`
- **Primary Agent Skill:** `@graphics-engineer`
- **Deliverable:** DrawingML media nodes (`p:pic`, `a:audioFile`, `a:videoFile`), media relationships, video poster frame caching, and synchronized media playback within the Slide Canvas.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 19.1.1
Invoke @graphics-engineer under GEC-v2.0:
1. Parse OOXML PresentationML media elements and relationships (`http://schemas.openxmlformats.org/officeDocument/2006/relationships/video`, `audio`).
2. Implement audio/video media asset extractor from OPC package parts.
3. Implement `SlideMediaOverlay.ts` rendering embedded HTML5 media elements overlaying the vector slide canvas with precise bounding box coordinates.
4. Add slide media controls (Play, Pause, Volume, Scrub, Loop, Poster Frame selection).
5. Add Insert Ribbon commands: "Insert Audio" and "Insert Video" supporting MP4, WebM, MP3, and WAV.
6. Add unit and UI integration tests verifying media element positioning, serialization, and playback trigger state.
7. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
