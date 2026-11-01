/**
 * Tauri IPC Client for OpenOfficeRS Unified Shell
 * GEC-v7.0 Stage 43 (Sub-Phase 43.1.1)
 *
 * Full Rust-to-UI Binary IPC Bridge & Command Protocol:
 * - Multi-document sessions & document serialization
 * - Undo/Redo & Retained Display List compilation
 * - High-speed zero-copy Binary Buffer Bridge (File IO, PDF, Crypto, Collab, Macro, Copilot, Data Connectors)
 */

import { WasmBridge, type WasmCoreEngine } from "./wasm_bridge";
export { WasmBridge, type WasmCoreEngine };

export type DocumentType = "Word" | "Excel" | "PowerPoint" | "PDF" | "Notes";

export interface SessionSummary {
  session_id: string;
  doc_type: DocumentType;
  title: string;
  is_dirty: boolean;
  file_path?: string;
}

export interface FormattingCommand {
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  font_family?: string;
  font_size_pt?: number;
}

export interface UiDispatchResult {
  session_id: string;
  description: string;
  undo_stack_depth: number;
  redo_stack_depth: number;
  is_dirty: boolean;
}

export type UiActionEvent =
  | { type: "ToggleBold" }
  | { type: "ToggleItalic" }
  | { type: "ToggleUnderline" }
  | { type: "ToggleStrikethrough" }
  | { type: "SetFontFamily"; payload: string }
  | { type: "SetFontSize"; payload: number }
  | { type: "InsertText"; payload: { text: string } }
  | { type: "DeleteTextRange"; payload: { offset: number; length: number } }
  | { type: "SetCellFormula"; payload: { coord: string; formula: string } }
  | { type: "SetCellValue"; payload: { coord: string; value: string } }
  | { type: "AddSheet"; payload: { title: string } }
  | { type: "EditRowCardField"; payload: { row: number; col: number; field_name: string; new_value: string } }
  | { type: "NewSlide"; payload: { layout: string } }
  | { type: "SetFormField"; payload: { field_name: string; value: string } }
  | { type: "RotatePdfPage"; payload: { page: number; angle_deg: number } }
  | { type: "ClearAllInk" }
  | { type: "Cut" }
  | { type: "Copy" }
  | { type: "Paste"; payload: { content: string } }
  | { type: "Undo" }
  | { type: "Redo" }
  | { type: "MarkSaved" };

export interface BootConfig {
  app_mode: "hub" | "writer" | "calc" | "slides" | "pdf" | "notes";
  initial_file?: string | null;
}

export interface AppInstance {
  instance_id: string;
  app_mode: "hub" | "writer" | "calc" | "slides" | "pdf" | "notes";
  title: string;
  file_path?: string | null;
  process_id?: number | null;
  status: string;
  launched_at_ms: number;
}

export interface DocumentMetadata {
  session_id: string;
  title: string;
  doc_type: DocumentType;
  file_path?: string | null;
  is_dirty: boolean;
  file_size_bytes: number;
  last_modified_str: string;
}

export interface CursorCoord {
  text_offset?: number | null;
  grid_row?: number | null;
  grid_col?: number | null;
}

export interface SelectionSpan {
  text_start?: number | null;
  text_end?: number | null;
  grid_start_row?: number | null;
  grid_start_col?: number | null;
  grid_end_row?: number | null;
  grid_end_col?: number | null;
}

export interface PeerPresence {
  client_id: number;
  username: string;
  color_hex: string;
  active_document: string;
  active_tab: string;
  cursor?: CursorCoord | null;
  selection?: SelectionSpan | null;
  last_seen_ms: number;
}

export interface CollabSessionInfo {
  session_id: string;
  client_id: number;
  username: string;
  app_mode: string;
  status: string;
  transport_mode: string;
  peers: PeerPresence[];
}

export interface CopilotQueryResult {
  session_id: string;
  prompt: string;
  response: string;
  confidence: number;
}

export interface DataConnectorResult {
  row_count: number;
  column_count: number;
  columns: string[];
}

export interface DataConnectorImportOptions {
  start_row?: number;
  start_col?: number;
  write_headers?: boolean;
  max_rows?: number;
}

export interface DataConnectorSchemaField {
  name: string;
  data_type: string;
  nullable: boolean;
}

export interface DataConnectorSchema {
  fields: DataConnectorSchemaField[];
}

export type DataConnectorValue =
  | null
  | boolean
  | number
  | string
  | { Boolean: boolean }
  | { Int64: number }
  | { Float64: number }
  | { Date: number }
  | { DateTime: number }
  | { Text: string }
  | { Binary: number[] };

export interface DataConnectorImportResult {
  rows_ingested: number;
  columns_ingested: number;
  cells_written: number;
  duration_ms: number;
  start_row: number;
  start_col: number;
  end_row: number;
  end_col: number;
  schema: DataConnectorSchema;
  sample_rows: DataConnectorValue[][];
}

export interface DataConnectorExportPayload {
  format: "csv" | "json" | "parquet" | "arrow" | "sqlite" | "ods";
  table_name?: string;
  schema: DataConnectorSchema;
  rows: DataConnectorValue[][];
}

export interface MacroExecutionResult {
  success: boolean;
  return_value: string;
  gas_consumed: number;
  error_message?: string;
}

export type IngestionStage =
  | "ReadingCentralDirectory"
  | "ResolvingContentTypes"
  | "ParsingRelationships"
  | "ExtractingParts"
  | "BuildingDom"
  | "Complete"
  | "Cancelled"
  | { Failed: string };

export interface IngestionProgress {
  stage: IngestionStage;
  bytes_processed: number;
  total_bytes: number;
  percentage: number;
  current_part?: string | null;
}

export interface HybridSearchResult {
  chunk: {
    id: string;
    doc_id: string;
    kind: string;
    raw_text: string;
    structural_context: string;
    composite_text: string;
    token_count_estimate: number;
  };
  semantic_score: number;
  lexical_score: number;
  score: number;
}

export interface IndexSummary {
  total_documents: number;
  total_chunks: number;
  chunks_by_kind: Record<string, number>;
  total_estimated_tokens: number;
}

export interface SearchOptions {
  case_sensitive: boolean;
  whole_word: boolean;
  use_regex: boolean;
}

export type MatchLocation =
  | { type: "Word"; details: { line: number; char_offset: number } }
  | { type: "Excel"; details: { sheet: string; coord: string } }
  | { type: "PowerPoint"; details: { slide_index: number } }
  | { type: "Pdf"; details: { page_number: number } }
  | { type: "Generic"; details: { offset: number } };

export interface FederatedSearchMatch {
  match_id: string;
  session_id: string;
  doc_type: DocumentType;
  document_title: string;
  location: MatchLocation;
  snippet: string;
  matched_text: string;
  start_offset: number;
  end_offset: number;
}

export interface FederatedReplaceSummary {
  total_replacements: number;
  documents_affected: number;
  replacements_by_session: Record<string, number>;
}

export type OmniboxCategory = "Command" | "File" | "Symbol" | "Search";

export interface OmniboxItem {
  id: string;
  category: OmniboxCategory;
  title: string;
  description: string;
  shortcut?: string | null;
  icon: string;
  score: number;
  payload: string;
}

export interface PlaceholderDto {
  ph_type: string;
  index?: number | null;
  shape_id: number;
  name: string;
  x: number;
  y: number;
  cx: number;
  cy: number;
  prompt_text: string;
}

export interface SlideLayoutDto {
  part_name: string;
  master_part_name: string;
  layout_type: string;
  name: string;
  show_master_shapes: boolean;
  placeholders: PlaceholderDto[];
}

export interface SlideMasterDto {
  part_name: string;
  id: number;
  name: string;
  theme_name: string;
  background_color?: string | null;
  placeholders: PlaceholderDto[];
  layouts: SlideLayoutDto[];
}

export interface MasterLayoutTreeDto {
  masters: SlideMasterDto[];
  slide_size: {
    cx: number;
    cy: number;
  };
}

export interface LayerItemDto {
  id: number;
  name: string;
  kind: string;
  z_index: number;
  is_hidden: boolean;
  is_locked: boolean;
  x: number;
  y: number;
  cx: number;
  cy: number;
  children: LayerItemDto[];
}

export interface SlideLayerTreeDto {
  slide_index: number;
  items: LayerItemDto[];
}

export interface SmartArtItemDto {
  title: string;
  description?: string | null;
  level: number;
  color_hex?: string | null;
}

export interface SmartArtBuildRequest {
  layout_type: string;
  items: SmartArtItemDto[];
  x: number;
  y: number;
  cx: number;
  cy: number;
  palette?: string[] | null;
}

export type GuideOrientation = "Horizontal" | "Vertical";
export type GuideKind = "Edge" | "Center" | "Spacing" | "Margin";

export interface SmartGuideLine {
  orientation: GuideOrientation;
  kind: GuideKind;
  coordinate: number;
  start: number;
  end: number;
}

export interface RectBounds {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface SnapResult {
  snapped_x: number;
  snapped_y: number;
  dx: number;
  dy: number;
  guides: SmartGuideLine[];
}

export interface MorphedTransformDto {
  x: number;
  y: number;
  width: number;
  height: number;
  rotation_deg: number;
}

export interface MorphedTextElementDto {
  text: string;
  x: number;
  y: number;
  font_size_pt: number;
  opacity: number;
}

export interface MorphedShapeStateDto {
  shape_id: number;
  name: string;
  transform: MorphedTransformDto;
  opacity: number;
  is_matched: boolean;
  text_elements: MorphedTextElementDto[];
}

export interface MorphedSlideStateDto {
  t: number;
  shapes: MorphedShapeStateDto[];
}

export interface AnimationTrackItemDto {
  shape_id: number;
  shape_name: string;
  effect_type: string;
  click_step: number;
  start_time_ms: number;
  duration_ms: number;
}

export interface SlideAnimationTracksDto {
  slide_index: number;
  total_clicks: number;
  total_duration_ms: number;
  tracks: AnimationTrackItemDto[];
}

export interface AnimatedShapeStateDto {
  shape_id: number;
  visible: boolean;
  opacity: number;
  translation_x: number;
  translation_y: number;
  rotation_deg: number;
  scale_x: number;
  scale_y: number;
  fill_color_rgb?: [number, number, number] | null;
}

export interface AnimatedSceneStateDto {
  time_ms: number;
  click_index: number;
  shapes: AnimatedShapeStateDto[];
}

export interface AnimationClickResultDto {
  click_index: number;
  total_clicks: number;
  time_ms: number;
  scene: AnimatedSceneStateDto;
}

export interface RenderedVectorPathDto {
  d: string;
  fill_color: string | null;
  stroke_color: string | null;
  stroke_width: number;
  fill_rule: string;
  opacity: number;
  line_cap: string;
  line_join: string;
  dash_array: string | null;
}

export interface RenderedTextElementDto {
  text: string;
  x: number;
  y: number;
  width: number;
  height: number;
  font_name: string;
  font_size: number;
  color_hex: string;
  transform: [number, number, number, number, number, number] | null;
}

export interface RenderedImageElementDto {
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  mime_type: string;
  data_base64: string;
  raw_bytes_len: number;
}

export interface RenderedPageCanvasDto {
  page_index: number;
  width_pt: number;
  height_pt: number;
  scale: number;
  vector_paths: RenderedVectorPathDto[];
  text_elements: RenderedTextElementDto[];
  image_elements: RenderedImageElementDto[];
  svg_markup: string;
}

export type BatesPosition =
  | "TopLeft"
  | "TopCenter"
  | "TopRight"
  | "BottomLeft"
  | "BottomCenter"
  | "BottomRight";

export interface BatesNumberingConfig {
  prefix: string;
  suffix: string;
  start_number: number;
  num_digits?: number;
  digits?: number;
  position: BatesPosition;
  font_size?: number;
  font_size_pt?: number;
  color_rgb?: [number, number, number];
  margin_pt?: number;
  margin_x_pt?: number;
  margin_y_pt?: number;
}

export interface OcrBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface OcrWord {
  text: string;
  bbox: OcrBoundingBox;
  confidence: number;
}

export interface OcrPageData {
  page_index: number;
  words: OcrWord[];
}

export type DiffKind = "Inserted" | "Deleted" | "Modified" | "Moved";

export interface DiffBoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DiffItemDto {
  id: string;
  kind: DiffKind;
  page_index_a?: number | null;
  page_index_b?: number | null;
  text_a?: string | null;
  text_b?: string | null;
  bbox_a?: DiffBoundingBox | null;
  bbox_b?: DiffBoundingBox | null;
  description: string;
}

export interface PageDiffDto {
  page_index: number;
  has_changes: boolean;
  media_box_changed: boolean;
  dimension_a?: [number, number] | null;
  dimension_b?: [number, number] | null;
  items: DiffItemDto[];
}

export interface PdfDiffSummaryDto {
  total_diffs: number;
  insertions: number;
  deletions: number;
  modifications: number;
  page_count_a: number;
  page_count_b: number;
  pages_with_diffs: number;
}

export interface PdfDiffResultDto {
  summary: PdfDiffSummaryDto;
  pages: PageDiffDto[];
}

export interface PdfDiffOptions {
  ignore_case?: boolean;
  ignore_whitespace?: boolean;
  detect_moves?: boolean;
}

export interface PdfSigningOptions {
  signer_name: string;
  reason?: string | null;
  location?: string | null;
  contact_info?: string | null;
  add_ltv_dss?: boolean;
  signing_time?: number | null;
}

export interface PdfSignatureValidationDto {
  field_name: string;
  is_valid: boolean;
  signer_common_name: string;
  signer_organization: string;
  has_ltv_dss: boolean;
  has_timestamp: boolean;
  signing_time?: number | null;
  byte_range: [number, number, number, number];
  document_hash: string;
  error_message?: string | null;
}

export interface PdfRedactionTarget {
  page_index: number;
  bbox?: DiffBoundingBox | null;
  text_pattern?: string | null;
  fill_color_rgb?: [number, number, number] | null;
  overlay_text?: string | null;
}

export interface PdfRedactionReportDto {
  total_redactions_applied: number;
  pages_redacted: number[];
  purged_strings: string[];
  forensically_sanitized: boolean;
}

export interface PdfaExportOptions {
  conformance?: string;
  title?: string | null;
  author?: string | null;
  output_condition?: string | null;
}

export type EmbedAnchor =
  | { type: "CalcRange"; data: { sheet_name: string; range_a1: string } }
  | { type: "CalcCell"; data: { sheet_name: string; coordinate: string } }
  | { type: "CalcChart"; data: { sheet_name: string; chart_id: string } }
  | { type: "WordParagraph"; data: { paragraph_id: string } }
  | { type: "WordSection"; data: { section_id: string } }
  | { type: "SlideShape"; data: { slide_index: number; shape_id: number } }
  | { type: "PdfRegion"; data: { page_index: number; bbox: [number, number, number, number] } }
  | { type: "GenericAnchor"; data: { key: string } };

export type EmbedTarget =
  | { type: "WordInline"; data: { container_id: string } }
  | { type: "SlideShapeContainer"; data: { slide_index: number; shape_id: number } }
  | { type: "NotesBlock"; data: { block_id: string } }
  | { type: "GenericTarget"; data: { id: string } };

export type DependencyKind =
  | "LiveRangeEmbed"
  | "LiveChartEmbed"
  | "LiveTextEmbed"
  | "CalculatedValueRef";

export interface DependencyEdge {
  id: string;
  source_doc_id: string;
  source_anchor: EmbedAnchor;
  target_doc_id: string;
  target_container: EmbedTarget;
  kind: DependencyKind;
  created_at: number;
  version: number;
}

export interface LiveEmbedSnapshot {
  embed_id: string;
  source_doc_id: string;
  version: number;
  rendered_html?: string | null;
  rendered_svg?: string | null;
  tabular_data?: string[][] | null;
  scalar_value?: string | null;
  last_updated_timestamp: number;
  is_stale: boolean;
}

export interface CrossDocSummaryDto {
  total_nodes: number;
  total_edges: number;
  total_snapshots: number;
  stale_snapshots: number;
}

export interface WikiLink {
  target_doc: string;
  target_anchor?: string | null;
  display_text?: string | null;
  start_offset: number;
  end_offset: number;
  raw_text: string;
}

export interface WikiLinkSuggestion {
  doc_id: string;
  title: string;
  anchor?: string | null;
  preview?: string | null;
  score: number;
}

export interface BacklinkReference {
  source_doc_id: string;
  source_doc_title: string;
  target_anchor?: string | null;
  display_text?: string | null;
  snippet: string;
  start_offset: number;
  end_offset: number;
}

export interface UnlinkedMention {
  source_doc_id: string;
  source_doc_title: string;
  matched_text: string;
  snippet: string;
  start_offset: number;
  end_offset: number;
}

export interface WikiLinkIndexStats {
  total_documents: number;
  total_wikilinks: number;
  total_unique_targets: number;
}

export type ConversionFormat =
  | "Docx"
  | "Xlsx"
  | "Pptx"
  | "Pdf"
  | "Markdown"
  | "Csv"
  | "Html"
  | "PlainText"
  | "Json";

export interface ConversionOptions {
  csv_delimiter?: string | null;
  include_headers?: boolean;
  pretty_print?: boolean;
  page_orientation?: string | null;
  author?: string | null;
  title?: string | null;
}

export interface ConversionTask {
  id: string;
  source_format: ConversionFormat;
  target_format: ConversionFormat;
  input_bytes: number[];
  options: ConversionOptions;
}

export interface ConversionResult {
  task_id: string;
  success: boolean;
  source_format: ConversionFormat;
  target_format: ConversionFormat;
  output_bytes: number[];
  error_message?: string | null;
  duration_ms: number;
}

export interface BatchConversionReport {
  total_tasks: number;
  succeeded: number;
  failed: number;
  results: ConversionResult[];
  duration_total_ms: number;
}

export interface ConversionEstimateDto {
  estimated_duration_ms: number;
  task_count: number;
  total_bytes: number;
}

export interface OpIdDto {
  lamport: number;
  client_id: number;
  seq: number;
}

export type OpPayloadDto =
  | { TextInsert: { parent_id: OpIdDto; content: string; right_origin?: OpIdDto | null } }
  | { TextDelete: { target_id: OpIdDto } }
  | { TextFormat: { start: number; len: number; attribute: unknown } }
  | { CellUpdate: { row: number; col: number; value: unknown } }
  | { RowShift: { after_row: number; count: number } }
  | { ShapeMutation: { shape_id: string; properties_json: string } }
  | { SnapshotCheckpoint: { checkpoint_id: string; label: string; snapshot_data: string } };

export interface OpLogEntryDto {
  entry_id: OpIdDto;
  parent_ids: OpIdDto[];
  timestamp_ms: number;
  author_id: string;
  doc_id: string;
  payload: OpPayloadDto;
  hash: string;
}

export interface CheckpointMetaDto {
  checkpoint_id: string;
  doc_id: string;
  label: string;
  op_id: OpIdDto;
  timestamp_ms: number;
  author_id: string;
  snapshot_preview: string;
}

export interface MaterializedDocumentStateDto {
  doc_id: string;
  timestamp_ms: number;
  text_content: string;
  cells: Record<string, unknown>;
  shapes: Record<string, string>;
  active_checkpoints: string[];
  total_ops_replayed: number;
  last_op_id?: OpIdDto | null;
}

export interface StateVectorDto {
  clocks: Record<number, number>;
}

export interface SyncReportDto {
  applied: number;
  skipped: number;
  conflicts_resolved: number;
  state_vector: StateVectorDto;
}

export interface PluginCellCoordDto {
  sheet: string;
  row: number;
  col: number;
}

export interface PluginSelectionInfoDto {
  doc_id: string;
  doc_type: DocumentType;
  start_offset: number;
  end_offset: number;
  selected_text: string;
}

export type PluginActionDto =
  | { QuerySelection: { doc_id: string } }
  | { QueryCellValue: { coord: PluginCellCoordDto } }
  | { QueryParagraph: { doc_id: string; para_index: number } }
  | { QuerySlideShapes: { doc_id: string; slide_index: number } }
  | { InsertText: { doc_id: string; text: string } }
  | { SetCellValue: { coord: PluginCellCoordDto; value: string } }
  | { SetCellFormula: { coord: PluginCellCoordDto; formula: string } }
  | { ExecuteRibbonCommand: { action: string; payload: string } };

export type ActionResultDto =
  | { Selection: PluginSelectionInfoDto }
  | { CellValue: string }
  | { ParagraphText: string }
  | { SlideShapes: string[] }
  | "MutationSuccess"
  | { RibbonSuccess: boolean };

export interface PluginStatusDto {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  state: string;
  capabilities: string[];
  fuel_limit: number;
  fuel_remaining: number;
  dispatched_events: number;
  exported_hooks: string[];
}

export interface PluginUdfMetaDto {
  name: string;
  description: string;
  min_args: number;
  max_args: number;
}

export interface PluginUdfCallResultDto {
  function_name: string;
  success: boolean;
  result_value?: string | null;
  error_message?: string | null;
}

export type CopilotProviderKind = "LocalOllama" | "WebGpuLlm" | "CloudApi" | "OfflineTemplate";

export interface ProviderConfigDto {
  provider_kind: CopilotProviderKind;
  base_url: string;
  model_name: string;
  api_key?: string | null;
  timeout_ms: number;
  temperature: number;
  max_tokens: number;
}

export interface ProviderStatusDto {
  provider_kind: string;
  model_name: string;
  endpoint: string;
  is_available: boolean;
  latency_ms: number;
}

export interface GroundedCitationDto {
  marker: string;
  doc_id: string;
  chunk_id: string;
  snippet: string;
  location: string;
}

export interface GroundedDiffDto {
  doc_id: string;
  original: string;
  modified: string;
  target_element?: string | null;
}

export interface CopilotChatMessageDto {
  id: string;
  sender: "user" | "assistant" | "system";
  text: string;
  timestamp_ms: number;
  citations: GroundedCitationDto[];
  diff?: GroundedDiffDto | null;
}

export interface CopilotChatRequestDto {
  query: string;
  session_id: string;
  active_selection?: {
    kind?: string;
    selected_text?: string | null;
    selected_range?: string | null;
  } | null;
  target_doc_ids?: string[] | null;
  conversation_history: CopilotChatMessageDto[];
}

export interface CopilotChatResponseDto {
  message_id: string;
  response_text: string;
  citations: GroundedCitationDto[];
  suggested_diff?: GroundedDiffDto | null;
  confidence_score: number;
  tokens_used: number;
  model_used: string;
}

export type ThemeMode = "light" | "dark" | "high-contrast-dark" | "high-contrast-light";
export type RibbonLayoutMode = "classic" | "compact" | "collapsed";

export interface ThemeTokensDto {
  mode: string;
  ribbon_mode: string;
  bg_primary: string;
  bg_surface: string;
  bg_shell: string;
  bg_canvas_backdrop: string;
  bg_panel: string;
  bg_toolbar: string;
  border_color: string;
  border_subtle: string;
  text_primary: string;
  text_secondary: string;
  text_muted: string;
  primary_accent: string;
  focus_ring: string;
  ribbon_height: number;
  title_bar_height: number;
  tab_bar_height: number;
  status_bar_height: number;
  button_height: number;
  icon_size: number;
  contrast_ratio: number;
}

export interface RibbonLayoutConfigDto {
  mode: RibbonLayoutMode;
  collapsed: boolean;
  height_px: number;
  is_single_row: boolean;
  show_group_headers: boolean;
}

export type AriaRole =
  | "document"
  | "heading"
  | "paragraph"
  | "grid"
  | "row"
  | "gridcell"
  | "shape"
  | "button"
  | "textbox"
  | "list"
  | "listitem"
  | "img"
  | "region";

export interface AriaBoundingBoxDto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface AriaNodeDto {
  id: string;
  role: AriaRole;
  label: string;
  value?: string | null;
  level?: number | null;
  selected: boolean;
  checked?: boolean | null;
  disabled: boolean;
  expanded?: boolean | null;
  live?: string | null;
  bounds?: AriaBoundingBoxDto | null;
  children: AriaNodeDto[];
}

export interface AriaTreeDto {
  root: AriaNodeDto;
  total_nodes: number;
}

export interface ImeCompositionStateDto {
  is_composing: boolean;
  preedit_string: string;
  cursor_position: number;
  selection_start: number;
  selection_end: number;
  caret_x: number;
  caret_y: number;
  caret_height: number;
  candidate_box: AriaBoundingBoxDto;
}

export type BidiDirection = "ltr" | "rtl" | "auto";

export interface BidiRunDto {
  text: string;
  direction: BidiDirection;
  start_idx: number;
  length: number;
}

export interface BidiAnalysisResultDto {
  base_direction: BidiDirection;
  runs: BidiRunDto[];
}

export type NavFocusPane = "title-bar" | "ribbon" | "side-pane" | "canvas" | "status-bar";

export interface TouchPointDto {
  id: number;
  x: number;
  y: number;
  radius_x: number;
  radius_y: number;
  pressure: number;
  is_stylus: boolean;
  timestamp_ms: number;
}

export type SwipeDirection = "Left" | "Right" | "Up" | "Down";

export type TouchGestureDto =
  | { type: "none" }
  | { type: "pinch-zoom"; payload: { scale: number; focal_x: number; focal_y: number } }
  | { type: "pan-scroll"; payload: { delta_x: number; delta_y: number } }
  | { type: "double-tap"; payload: { x: number; y: number } }
  | { type: "long-press"; payload: { x: number; y: number } }
  | { type: "two-finger-swipe"; payload: { direction: SwipeDirection } };

export type SplitGridMode =
  | "single"
  | "two-way-horizontal"
  | "two-way-vertical"
  | "four-way-grid";

export interface AdaptiveSplitGridDto {
  mode: SplitGridMode;
  ratio_x: number;
  ratio_y: number;
  snap_threshold: number;
}

export interface Rect2DDto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type DlpSeverity = "low" | "medium" | "high" | "critical";

export type DlpAction =
  | "allow"
  | "warn"
  | "redact-and-allow"
  | "block"
  | "quarantine";

export type DlpPatternDto =
  | { kind: "pii"; value: string }
  | { kind: "keyword"; value: string }
  | { kind: "regex-pattern"; value: string };

export interface DlpRuleDto {
  id: string;
  name: string;
  description: string;
  patterns: DlpPatternDto[];
  threshold: number;
  severity: DlpSeverity;
  action: DlpAction;
}

export interface DlpPolicyDto {
  id: string;
  name: string;
  version: number;
  rules: DlpRuleDto[];
  default_action: DlpAction;
}

export interface DlpMatchDto {
  rule_id: string;
  pattern_label: string;
  matched_text: string;
  start: number;
  end: number;
  severity: DlpSeverity;
  replacement: string;
}

export interface DlpEvaluationResultDto {
  policy_id: string;
  policy_version: number;
  action: DlpAction;
  highest_severity: DlpSeverity | null;
  matches: DlpMatchDto[];
  triggered_rule_ids: string[];
  is_violation: boolean;
  summary: string;
}

export interface SanitizationCertificateDto {
  certificate_id: string;
  doc_id: string;
  original_digest: string;
  sanitized_digest: string;
  policy_id: string;
  sanitized_at_ms: number;
  redactions_count: Array<[string, number]>;
  signature: string;
}

export type AuditEventTypeDto =
  | { type: "document-opened" }
  | { type: "document-saved" }
  | { type: "dlp-violation-detected" }
  | { type: "document-sanitized" }
  | { type: "export-executed" }
  | { type: "signature-applied" }
  | { type: "access-revoked" }
  | { type: "custom"; payload: string };

export interface AuditLogEntryDto {
  sequence_number: number;
  timestamp_ms: number;
  event_type: AuditEventTypeDto | string;
  user_id: string;
  doc_id: string;
  details: string;
  prev_hash: string;
  entry_hash: string;
}

export type EditionTierDto = "community" | "professional" | "enterprise";

export type FeatureEntitlementDto =
  | "basic-editing"
  | "advanced-typography"
  | "standard-formulas"
  | "full-document-io"
  | "local-ai-copilot"
  | "macro-debugger"
  | "batch-conversion"
  | "enterprise-dlp"
  | "sanitization-certificates"
  | "tamper-evident-audit-logs"
  | "realtime-collab-webrtc"
  | "custom-wasm-plugins";

export interface LicenseTokenDto {
  license_id: string;
  licensee_name: string;
  organization?: string | null;
  tier: EditionTierDto;
  issued_at_ms: number;
  expires_at_ms?: number | null;
  max_seats: number;
  custom_features: string[];
  signature: string;
}

export type EntitlementStatusDto =
  | { status: "granted" }
  | { status: "grace-period"; days_remaining: number; message: string }
  | { status: "denied"; required_tier: EditionTierDto; reason: string };

export interface SlaBudgetThresholdsDto {
  max_cold_start_ms: number;
  max_idle_memory_mb: number;
  max_active_session_memory_mb: number;
  max_keystroke_latency_ms: number;
  max_frame_render_latency_ms: number;
}

export interface SlaMetricResultDto {
  metric_name: string;
  measured_value: number;
  budget_threshold: number;
  unit: string;
  passed: boolean;
  p50_ms?: number | null;
  p99_ms?: number | null;
}

export interface SlaBenchmarkReportDto {
  timestamp_ms: number;
  all_passed: boolean;
  metrics: SlaMetricResultDto[];
  summary: string;
}

export type FuzzMutationTypeDto =
  | "truncated-zip"
  | "corrupted-xml-tag"
  | "malformed-utf8-sequence"
  | "cyclic-entity-expansion"
  | "deep-nesting";

export interface DifferentialAuditResultDto {
  is_identical: boolean;
  total_base_commands: number;
  total_candidate_commands: number;
  divergences: string[];
}

export type TargetOsDto = "Windows" | "Linux" | "MacOS" | "Android" | "Ios";
export type TargetArchDto = "X86_64" | "Aarch64" | "Armv7" | "Universal";
export type PackagingBundleDto =
  | "Msi"
  | "Nsis"
  | "AppImage"
  | "Deb"
  | "Rpm"
  | "Dmg"
  | "AppBundle"
  | "Apk"
  | "Aab"
  | "Ipa"
  | "StandaloneExe"
  | "StandaloneElf";

export interface InstallerArtifactDto {
  id: string;
  platform: TargetOsDto;
  arch: TargetArchDto;
  bundle_format: PackagingBundleDto;
  file_name: string;
  sha256_checksum: string;
  file_size_bytes: number;
  min_os_version: string;
  is_signed: boolean;
  cert_fingerprint: string | null;
  download_url: string;
}

export interface InstallerMatrixManifestDto {
  app_name: string;
  version: string;
  release_date_iso: string;
  artifacts: InstallerArtifactDto[];
}

export type SbomComponentTypeDto = "library" | "application" | "framework";

export interface SbomComponentDto {
  name: string;
  version: string;
  purl: string;
  license_spdx: string;
  description: string;
  sha256_hash: string;
  component_type: SbomComponentTypeDto;
  dependencies: string[];
}

export interface LicenseAuditReportDto {
  total_components: number;
  compliant_count: number;
  non_compliant_count: number;
  unlicensed_count: number;
  licenses_found: string[];
  has_gpl_contamination: boolean;
  is_clean_room_compliant: boolean;
  violation_details: string[];
}

// Fallback when running outside Tauri desktop container (e.g. Vite dev preview / unit tests)
function isTauriAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
const mockInstances: AppInstance[] = [];
let mockCollabSession: CollabSessionInfo | null = null;
const mockEdges = new Map<string, DependencyEdge>();
const mockSnapshots = new Map<string, LiveEmbedSnapshot>();
const mockIndexedDocs = new Map<string, { title: string; content: string; links: WikiLink[] }>();
const mockOpLogs = new Map<string, OpLogEntryDto[]>();
const mockCheckpoints = new Map<string, CheckpointMetaDto[]>();
const mockStateVectors = new Map<string, Record<number, number>>();
const mockPlugins = new Map<string, PluginStatusDto>();
const mockUdfs = new Map<string, PluginUdfMetaDto>();
let mockProviderConfig: ProviderConfigDto = {
  provider_kind: "OfflineTemplate",
  base_url: "http://localhost:11434",
  model_name: "llama3.2",
  api_key: null,
  timeout_ms: 10000,
  temperature: 0.2,
  max_tokens: 2048,
};
const mockCopilotDocs = new Map<string, { title: string; text: string }>();
let mockCopilotMessageCounter = 0;
let mockCurrentTheme: ThemeMode = "light";
let mockRibbonLayout: RibbonLayoutConfigDto = {
  mode: "classic",
  collapsed: false,
  height_px: 96,
  is_single_row: false,
  show_group_headers: true,
};
let mockAriaTree: AriaTreeDto = {
  root: {
    id: "root-doc",
    role: "document",
    label: "Document Surface",
    selected: false,
    disabled: false,
    children: [],
  },
  total_nodes: 1,
};
let mockImeState: ImeCompositionStateDto = {
  is_composing: false,
  preedit_string: "",
  cursor_position: 0,
  selection_start: 0,
  selection_end: 0,
  caret_x: 0,
  caret_y: 0,
  caret_height: 18,
  candidate_box: { x: 0, y: 18, width: 220, height: 130 },
};
let mockNavPane: NavFocusPane = "canvas";
const mockActiveTouches = new Map<number, TouchPointDto>();
let mockSplitGrid: AdaptiveSplitGridDto = {
  mode: "single",
  ratio_x: 0.5,
  ratio_y: 0.5,
  snap_threshold: 0.04,
};
let mockDlpPolicy: DlpPolicyDto = {
  id: "ent-dlp-std",
  name: "Standard Enterprise Governance",
  version: 1,
  default_action: "allow",
  rules: [
    {
      id: "rule-pci-dss",
      name: "Payment Card Data Protection",
      description: "Prohibits unencrypted transmission of credit card numbers",
      patterns: [{ kind: "pii", value: "credit-card" }],
      threshold: 1,
      severity: "critical",
      action: "block",
    },
    {
      id: "rule-ssn-redact",
      name: "Social Security Number Sanitization",
      description: "Mandates redaction of SSNs prior to export",
      patterns: [{ kind: "pii", value: "ssn" }],
      threshold: 1,
      severity: "high",
      action: "redact-and-allow",
    },
  ],
};
const mockAuditEntries: AuditLogEntryDto[] = [];
let mockEditionTier: EditionTierDto = "community";
let mockActiveLicense: LicenseTokenDto | null = null;

function mockEvaluateThemeTokens(mode: ThemeMode, ribbonMode: RibbonLayoutMode): ThemeTokensDto {
  let bg_primary = "#f8fafc";
  let bg_surface = "#ffffff";
  let bg_shell = "#f1f5f9";
  let bg_canvas_backdrop = "#e2e8f0";
  let bg_panel = "#ffffff";
  let bg_toolbar = "#f8fafc";
  let border_color = "#cbd5e1";
  let border_subtle = "#e2e8f0";
  let text_primary = "#0f172a";
  let text_secondary = "#475569";
  let text_muted = "#64748b";
  let primary_accent = "#0f6cbd";
  let focus_ring = "#0f6cbd";
  let contrast_ratio = 16.5;

  if (mode === "dark") {
    bg_primary = "#0f172a";
    bg_surface = "#1e293b";
    bg_shell = "#0b1120";
    bg_canvas_backdrop = "#020617";
    bg_panel = "#1e293b";
    bg_toolbar = "#1e293b";
    border_color = "#334155";
    border_subtle = "#1e293b";
    text_primary = "#f8fafc";
    text_secondary = "#94a3b8";
    text_muted = "#64748b";
    primary_accent = "#38bdf8";
    focus_ring = "#38bdf8";
    contrast_ratio = 12.3;
  } else if (mode === "high-contrast-dark") {
    bg_primary = "#000000";
    bg_surface = "#000000";
    bg_shell = "#000000";
    bg_canvas_backdrop = "#000000";
    bg_panel = "#000000";
    bg_toolbar = "#000000";
    border_color = "#ffffff";
    border_subtle = "#ffffff";
    text_primary = "#ffffff";
    text_secondary = "#ffff00";
    text_muted = "#00ffff";
    primary_accent = "#00ffff";
    focus_ring = "#00ffff";
    contrast_ratio = 21.0;
  } else if (mode === "high-contrast-light") {
    bg_primary = "#ffffff";
    bg_surface = "#ffffff";
    bg_shell = "#ffffff";
    bg_canvas_backdrop = "#ffffff";
    bg_panel = "#ffffff";
    bg_toolbar = "#ffffff";
    border_color = "#000000";
    border_subtle = "#000000";
    text_primary = "#000000";
    text_secondary = "#000080";
    text_muted = "#333333";
    primary_accent = "#0000ee";
    focus_ring = "#0000ee";
    contrast_ratio = 21.0;
  }

  let ribbon_height = 96;
  let button_height = 32;
  let icon_size = 20;

  if (ribbonMode === "compact") {
    ribbon_height = 44;
    button_height = 26;
    icon_size = 16;
  } else if (ribbonMode === "collapsed") {
    ribbon_height = 32;
    button_height = 22;
    icon_size = 14;
  }

  return {
    mode,
    ribbon_mode: ribbonMode,
    bg_primary,
    bg_surface,
    bg_shell,
    bg_canvas_backdrop,
    bg_panel,
    bg_toolbar,
    border_color,
    border_subtle,
    text_primary,
    text_secondary,
    text_muted,
    primary_accent,
    focus_ring,
    ribbon_height,
    title_bar_height: 38,
    tab_bar_height: 34,
    status_bar_height: 26,
    button_height,
    icon_size,
    contrast_ratio,
  };
}

async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauriAvailable()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  } else {
    console.log(`[IPC Mock] invoke('${cmd}')`, args);
    // Mock response for browser preview
    if (cmd === "get_boot_config") {
      const params = typeof window !== "undefined" ? new URLSearchParams(window.location.search) : null;
      const mode = (params?.get("mode") as "hub" | "writer" | "calc" | "slides" | "pdf") || "writer";
      return {
        app_mode: mode,
        initial_file: params?.get("file") || null,
      } as unknown as T;
    }
    if (cmd === "create_session") {
      const type = (args?.doc_type as DocumentType) || "Word";
      return {
        session_id: `sess-${Date.now()}`,
        doc_type: type,
        title: (args?.title as string) || `New ${type}`,
        is_dirty: false,
      } as unknown as T;
    }
    if (cmd === "launch_standalone_app") {
      const mode = (args?.app_mode as "hub" | "writer" | "calc" | "slides" | "pdf") || "writer";
      const file = args?.file_path as string | undefined;
      const appName =
        mode === "writer"
          ? "AetherWriter"
          : mode === "calc"
          ? "AetherCalc"
          : mode === "slides"
          ? "AetherSlides"
          : mode === "pdf"
          ? "AetherPDF"
          : "AetherHub";
      const title = file
        ? `${appName} - ${file.split(/[/\\]/).pop()}`
        : `${appName} (Instance ${mockInstances.length + 1})`;
      const inst: AppInstance = {
        instance_id: `inst-${mode}-${Date.now()}-${mockInstances.length + 1}`,
        app_mode: mode,
        title,
        file_path: file || null,
        process_id: 10000 + mockInstances.length + 1,
        status: "running",
        launched_at_ms: Date.now(),
      };
      mockInstances.push(inst);
      return inst as unknown as T;
    }
    if (cmd === "list_app_instances") {
      return [...mockInstances] as unknown as T;
    }
    if (cmd === "close_app_instance") {
      const id = args?.instance_id as string;
      const idx = mockInstances.findIndex((i) => i.instance_id === id);
      if (idx >= 0) {
        mockInstances.splice(idx, 1);
        return true as unknown as T;
      }
      return false as unknown as T;
    }
    if (cmd === "focus_app_instance") {
      const id = args?.instance_id as string;
      return mockInstances.some((i) => i.instance_id === id) as unknown as T;
    }
    if (cmd === "spawn_app_window") {
      const type = (args?.app_type as string) || "word";
      const winId = `win-${type}-${Date.now()}`;
      return winId as unknown as T;
    }
    if (cmd === "open_document" || cmd === "open_document_async") {
      const path = (args?.file_path as string) || "Document.docx";
      const filename = path.split(/[/\\]/).pop() || path;
      const lower = path.toLowerCase();
      const docType: DocumentType =
        lower.endsWith(".xlsx") || lower.endsWith(".xls") || lower.endsWith(".csv")
          ? "Excel"
          : lower.endsWith(".pptx") || lower.endsWith(".ppt")
          ? "PowerPoint"
          : lower.endsWith(".pdf")
          ? "PDF"
          : lower.endsWith(".txt") || lower.endsWith(".md") || lower.endsWith(".log") || lower.endsWith(".json")
          ? "Notes"
          : "Word";
      return {
        session_id: (args?.session_id as string) || `sess-${Date.now()}`,
        doc_type: docType,
        title: filename,
        is_dirty: false,
        file_path: path,
      } as unknown as T;
    }
    if (cmd === "cancel_ingestion") {
      return true as unknown as T;
    }
    if (cmd === "save_document") {
      const path = (args?.file_path as string) || "Saved_Document.docx";
      return path as unknown as T;
    }
    if (cmd === "save_file_bytes" || cmd === "save_file_binary") {
      const fileName = (args?.file_name as string) || "document.bin";
      const customPath = args?.custom_path as string | undefined;
      return (customPath || `/mock/saved/${fileName}`) as unknown as T;
    }
    if (cmd === "read_file_bytes") {
      return [] as unknown as T;
    }
    if (cmd === "read_file_binary") {
      return new Uint8Array([0x50, 0x4b, 0x03, 0x04]) as unknown as T;
    }
    if (cmd === "render_pdf_page_binary") {
      return new TextEncoder().encode("PDF_PAGE_MOCK_V1\n") as unknown as T;
    }
    if (cmd === "crypto_hash_binary") {
      return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" as unknown as T;
    }
    if (cmd === "collab_sync_binary") {
      return new TextEncoder().encode("AECR\x00\x01") as unknown as T;
    }
    if (cmd === "macro_execute_code") {
      return {
        success: true,
        return_value: "MockMacroOutput",
        gas_consumed: 120,
      } as unknown as T;
    }
    if (cmd === "copilot_query") {
      const sessId = (args?.session_id as string) || "sess-mock";
      const prompt = (args?.prompt as string) || "";
      return {
        session_id: sessId,
        prompt,
        response: `[Aether AI] Verified document synthesis for: ${prompt}`,
        confidence: 0.98,
      } as unknown as T;
    }
    if (cmd === "data_connector_parse_csv") {
      return {
        row_count: 5,
        column_count: 3,
        columns: ["Col1", "Col2", "Col3"],
      } as unknown as T;
    }
    if (cmd === "data_connector_import") {
      const format = (args?.format as string) || "csv";
      const byteList = (args?.bytes as number[]) || [];
      const u8 = new Uint8Array(byteList);
      const text = new TextDecoder().decode(u8);
      const optsJson = args?.options_json as string | undefined;
      const opts = optsJson ? JSON.parse(optsJson) : {};
      const startRow = opts.start_row ?? 0;
      const startCol = opts.start_col ?? 0;
      const writeHeaders = opts.write_headers ?? true;

      if (format === "json") {
        try {
          const parsed = JSON.parse(text);
          if (Array.isArray(parsed) && parsed.length > 0) {
            const keys = Object.keys(parsed[0]);
            const fields = keys.map((k) => ({
              name: k,
              data_type: typeof parsed[0][k] === "number" ? "Float64" : "Utf8",
              nullable: true,
            }));
            const sample_rows = parsed.map((item) =>
              keys.map((k) => {
                const v = item[k];
                if (v === null || v === undefined) return null;
                if (typeof v === "number") return { Float64: v };
                if (typeof v === "boolean") return { Boolean: v };
                return { Text: String(v) };
              })
            );
            return {
              rows_ingested: sample_rows.length,
              columns_ingested: keys.length,
              cells_written: sample_rows.length * keys.length,
              duration_ms: 0.5,
              start_row: startRow,
              start_col: startCol,
              end_row: startRow + (writeHeaders ? 1 : 0) + sample_rows.length,
              end_col: startCol + keys.length,
              schema: { fields },
              sample_rows,
            } as unknown as T;
          }
        } catch {
          // Fall through
        }
      }

      const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
      if (lines.length > 0) {
        if (!writeHeaders) {
          const firstLine = lines[0];
          const delimiter = firstLine.includes("\t") ? "\t" : firstLine.includes(";") ? ";" : ",";
          const firstCols = firstLine.split(delimiter);
          const fields = firstCols.map((_, i) => ({ name: `Col_${i + 1}`, data_type: "Utf8", nullable: true }));
          const sample_rows = lines.map((line) =>
            line.split(delimiter).map((tok) => {
              const clean = tok.replace(/^"|"$/g, "").trim();
              const num = Number(clean);
              if (clean !== "" && !isNaN(num)) {
                return Number.isInteger(num) ? { Int64: num } : { Float64: num };
              }
              return { Text: clean };
            })
          );
          return {
            rows_ingested: sample_rows.length,
            columns_ingested: fields.length,
            cells_written: sample_rows.length * fields.length,
            duration_ms: 0.5,
            start_row: startRow,
            start_col: startCol,
            end_row: startRow + sample_rows.length,
            end_col: startCol + fields.length,
            schema: { fields },
            sample_rows,
          } as unknown as T;
        }

        const firstLine = lines[0];
        const delimiter = firstLine.includes("\t") ? "\t" : firstLine.includes(";") ? ";" : ",";
        const headers = firstLine.split(delimiter).map((h) => h.replace(/^"|"$/g, "").trim());
        const fields = headers.map((h) => ({ name: h, data_type: "Utf8", nullable: true }));
        const sample_rows = lines.slice(1).map((line) =>
          line.split(delimiter).map((tok) => {
            const clean = tok.replace(/^"|"$/g, "").trim();
            const num = Number(clean);
            if (clean !== "" && !isNaN(num)) {
              return Number.isInteger(num) ? { Int64: num } : { Float64: num };
            }
            return { Text: clean };
          })
        );
        return {
          rows_ingested: sample_rows.length,
          columns_ingested: headers.length,
          cells_written: sample_rows.length * headers.length,
          duration_ms: 0.5,
          start_row: startRow,
          start_col: startCol,
          end_row: startRow + (writeHeaders ? 1 : 0) + sample_rows.length,
          end_col: startCol + headers.length,
          schema: { fields },
          sample_rows,
        } as unknown as T;
      }

      return {
        rows_ingested: 0,
        columns_ingested: 0,
        cells_written: 0,
        duration_ms: 0.0,
        start_row: startRow,
        start_col: startCol,
        end_row: startRow,
        end_col: startCol,
        schema: { fields: [] },
        sample_rows: [],
      } as unknown as T;
    }
    if (cmd === "data_connector_export") {
      return new Uint8Array([0x50, 0x4b, 0x03, 0x04]) as unknown as T;
    }
    if (cmd === "ppt_get_slide_master_tree") {
      const mockMaster: SlideMasterDto = {
        part_name: "ppt/slideMasters/slideMaster1.xml",
        id: 1,
        name: "Office Theme",
        theme_name: "Office Theme",
        placeholders: [
          {
            ph_type: "title",
            shape_id: 1,
            name: "Title Placeholder",
            x: 838200,
            y: 457200,
            cx: 10515600,
            cy: 1143000,
            prompt_text: "Click to edit Master title style",
          },
          {
            ph_type: "body",
            shape_id: 2,
            name: "Text Placeholder",
            x: 838200,
            y: 1828800,
            cx: 10515600,
            cy: 4572000,
            prompt_text: "Click to edit Master text styles",
          },
        ],
        layouts: [
          {
            part_name: "ppt/slideLayouts/slideLayout1.xml",
            master_part_name: "ppt/slideMasters/slideMaster1.xml",
            layout_type: "title",
            name: "Title Slide",
            show_master_shapes: true,
            placeholders: [
              {
                ph_type: "title",
                shape_id: 1,
                name: "Title",
                x: 838200,
                y: 1828800,
                cx: 10515600,
                cy: 1828800,
                prompt_text: "Click to edit Master title style",
              },
              {
                ph_type: "subTitle",
                shape_id: 2,
                name: "Subtitle",
                x: 838200,
                y: 3886200,
                cx: 10515600,
                cy: 1524000,
                prompt_text: "Click to edit Master subtitle style",
              },
            ],
          },
          {
            part_name: "ppt/slideLayouts/slideLayout2.xml",
            master_part_name: "ppt/slideMasters/slideMaster1.xml",
            layout_type: "titleAndContent",
            name: "Title and Content",
            show_master_shapes: true,
            placeholders: [
              {
                ph_type: "title",
                shape_id: 1,
                name: "Title",
                x: 838200,
                y: 457200,
                cx: 10515600,
                cy: 1143000,
                prompt_text: "Click to edit Master title style",
              },
              {
                ph_type: "body",
                shape_id: 2,
                name: "Content",
                x: 838200,
                y: 1828800,
                cx: 10515600,
                cy: 4572000,
                prompt_text: "Click to edit Master text styles",
              },
            ],
          },
        ],
      };
      return {
        masters: [mockMaster],
        slide_size: { cx: 12192000, cy: 6858000 },
      } as unknown as T;
    }
    if (cmd === "ppt_apply_slide_master_layout") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_create_slide_layout") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_update_placeholder_transform") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_build_smartart") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_group_shapes") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_ungroup_shapes") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_reorder_z_order") {
      const orig = (args?.package_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_get_slide_layer_tree") {
      const mockTree: SlideLayerTreeDto = {
        slide_index: (args?.slide_idx as number) || 0,
        items: [
          {
            id: 1,
            name: "Title 1",
            kind: "shape",
            z_index: 0,
            is_hidden: false,
            is_locked: false,
            x: 838200,
            y: 457200,
            cx: 10515600,
            cy: 1143000,
            children: [],
          },
          {
            id: 2,
            name: "Content 2",
            kind: "shape",
            z_index: 1,
            is_hidden: false,
            is_locked: false,
            x: 838200,
            y: 1828800,
            cx: 10515600,
            cy: 4572000,
            children: [],
          },
        ],
      };
      return mockTree as unknown as T;
    }
    if (cmd === "render_smart_guides_snap") {
      const movingStr = (args?.moving_bounds_json as string) || "{}";
      const moving: RectBounds = JSON.parse(movingStr);
      const neighborsStr = (args?.neighbors_json as string) || "[]";
      const neighbors: RectBounds[] = JSON.parse(neighborsStr);
      let snapped_x = moving.x ?? 0;
      let snapped_y = moving.y ?? 0;
      let dx = 0;
      let dy = 0;
      const guides: SmartGuideLine[] = [];
      const threshold = (args?.threshold as number) || 5;

      for (const n of neighbors) {
        if (Math.abs(moving.x - n.x) <= threshold) {
          dx = n.x - moving.x;
          snapped_x = n.x;
          guides.push({
            orientation: "Vertical",
            kind: "Edge",
            coordinate: n.x,
            start: Math.min(moving.y, n.y),
            end: Math.max((moving.y ?? 0) + (moving.h ?? 0), (n.y ?? 0) + (n.h ?? 0)),
          });
        }
      }

      return {
        snapped_x,
        snapped_y,
        dx,
        dy,
        guides,
      } as unknown as T;
    }
    if (cmd === "ppt_evaluate_morph_transition") {
      const progress = (args?.progress as number) ?? 0.0;
      const morphRes: MorphedSlideStateDto = {
        t: progress,
        shapes: [
          {
            shape_id: 1,
            name: "Title 1",
            transform: {
              x: 100 + 200 * progress,
              y: 100 + 50 * progress,
              width: 400 + 100 * progress,
              height: 100,
              rotation_deg: 15 * progress,
            },
            opacity: 1.0,
            is_matched: true,
            text_elements: [
              {
                text: "Overview",
                x: 120 + 200 * progress,
                y: 130 + 50 * progress,
                font_size_pt: 24,
                opacity: 1.0,
              },
            ],
          },
        ],
      };
      return morphRes as unknown as T;
    }
    if (cmd === "ppt_get_slide_animation_tracks") {
      const slideIdx = (args?.slide_idx as number) ?? 0;
      const tracksRes: SlideAnimationTracksDto = {
        slide_index: slideIdx,
        total_clicks: 2,
        total_duration_ms: 2500,
        tracks: [
          {
            shape_id: 1,
            shape_name: "Title 1",
            effect_type: "Fade",
            click_step: 0,
            start_time_ms: 0,
            duration_ms: 1000,
          },
          {
            shape_id: 2,
            shape_name: "Subtitle 2",
            effect_type: "Motion",
            click_step: 1,
            start_time_ms: 500,
            duration_ms: 1000,
          },
        ],
      };
      return tracksRes as unknown as T;
    }
    if (cmd === "ppt_scrub_slide_animation") {
      const timeMs = (args?.time_ms as number) ?? 0.0;
      const clickIdx = (args?.click_index as number) ?? 0;
      const sceneRes: AnimatedSceneStateDto = {
        time_ms: timeMs,
        click_index: clickIdx,
        shapes: [
          {
            shape_id: 1,
            visible: true,
            opacity: Math.min(1.0, timeMs / 1000.0),
            translation_x: 0,
            translation_y: 0,
            rotation_deg: 0,
            scale_x: 1,
            scale_y: 1,
            fill_color_rgb: [30, 144, 255],
          },
        ],
      };
      return sceneRes as unknown as T;
    }
    if (cmd === "ppt_advance_animation_click") {
      const currentClick = (args?.current_click as number) ?? 0;
      const forward = (args?.forward as boolean) ?? true;
      const nextClick = forward ? currentClick + 1 : Math.max(0, currentClick - 1);
      const clickRes: AnimationClickResultDto = {
        click_index: nextClick,
        total_clicks: 2,
        time_ms: 0,
        scene: {
          time_ms: 0,
          click_index: nextClick,
          shapes: [
            {
              shape_id: 1,
              visible: true,
              opacity: 1.0,
              translation_x: 0,
              translation_y: 0,
              rotation_deg: 0,
              scale_x: 1,
              scale_y: 1,
            },
          ],
        },
      };
      return clickRes as unknown as T;
    }
    if (cmd === "roundtrip_opc_binary") {
      const orig = (args?.original_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_export_slide_svg") {
      const slideIdx = (args?.slide_idx as number) ?? 0;
      return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 540" width="100%" height="100%"><rect width="960" height="540" fill="#FFFFFF"/><text x="480" y="270" text-anchor="middle" font-size="24" fill="#000000">Slide ${slideIdx + 1}</text></svg>` as unknown as T;
    }
    if (cmd === "ppt_export_deck_pdf") {
      const pdfBytes = new TextEncoder().encode("%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>\nendobj\n%%EOF");
      return pdfBytes as unknown as T;
    }
    if (cmd === "ppt_convert_odp_to_pptx") {
      const orig = (args?.odp_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "ppt_convert_pptx_to_odp") {
      const orig = (args?.pptx_bytes as number[]) || [0x50, 0x4b, 0x03, 0x04];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "pdf_render_page_canvas") {
      const pageIdx = (args?.page_index as number) ?? 0;
      const scale = (args?.scale as number) ?? 1.0;
      const mockCanvas: RenderedPageCanvasDto = {
        page_index: pageIdx,
        width_pt: 612.0,
        height_pt: 792.0,
        scale,
        vector_paths: [
          {
            d: "M 50.00 50.00 L 200.00 50.00 L 200.00 150.00 L 50.00 150.00 Z",
            fill_color: "#4472C4",
            stroke_color: "#000000",
            stroke_width: 1.0,
            fill_rule: "nonzero",
            opacity: 1.0,
            line_cap: "butt",
            line_join: "miter",
            dash_array: null,
          },
        ],
        text_elements: [
          {
            text: `PDF Page ${pageIdx + 1} Title`,
            x: 50.0,
            y: 50.0,
            width: 250.0,
            height: 24.0,
            font_name: "Helvetica",
            font_size: 18.0,
            color_hex: "#000000",
            transform: [1, 0, 0, 1, 50, 742],
          },
        ],
        image_elements: [],
        svg_markup: `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 612.00 792.00" width="612.00pt" height="792.00pt"><rect width="100%" height="100%" fill="#FFFFFF"/><text x="50.00" y="70.00" font-size="18.00pt">PDF Page ${pageIdx + 1}</text></svg>`,
      };
      return mockCanvas as unknown as T;
    }
    if (cmd === "pdf_render_page_svg") {
      const pageIdx = (args?.page_index as number) ?? 0;
      return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 612.00 792.00" width="612.00pt" height="792.00pt"><rect width="100%" height="100%" fill="#FFFFFF"/><text x="50.00" y="70.00" font-size="18.00pt">PDF Page ${pageIdx + 1}</text></svg>` as unknown as T;
    }
    if (cmd === "pdf_extract_page_images") {
      return [] as unknown as T;
    }
    if (cmd === "pdf_split_document") {
      const orig = (args?.bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return [new Uint8Array(orig)] as unknown as T;
    }
    if (cmd === "pdf_merge_documents") {
      const docs = (args?.documents_bytes as number[][]) || [];
      const first = docs[0] || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(first) as unknown as T;
    }
    if (cmd === "pdf_extract_pages") {
      const orig = (args?.bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "pdf_apply_bates_numbering") {
      const orig = (args?.bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "pdf_inject_ocr_layer") {
      const orig = (args?.bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "pdf_compare_documents") {
      const mockResult: PdfDiffResultDto = {
        summary: {
          total_diffs: 1,
          insertions: 1,
          deletions: 0,
          modifications: 0,
          page_count_a: 1,
          page_count_b: 1,
          pages_with_diffs: 1,
        },
        pages: [
          {
            page_index: 0,
            has_changes: true,
            media_box_changed: false,
            dimension_a: [612, 792],
            dimension_b: [612, 792],
            items: [
              {
                id: "diff-mock-1",
                kind: "Inserted",
                page_index_a: null,
                page_index_b: 0,
                text_a: null,
                text_b: "Revised Clause",
                bbox_a: null,
                bbox_b: { x: 50, y: 700, width: 120, height: 16 },
                description: "Inserted 'Revised Clause'",
              },
            ],
          },
        ],
      };
      return mockResult as unknown as T;
    }
    if (cmd === "pdf_generate_diff_overlay_svg") {
      return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 612.00 792.00" width="612.00pt" height="792.00pt" style="pointer-events: none;"><rect x="50.00" y="76.00" width="120.00" height="16.00" rx="2" fill="rgba(34, 197, 94, 0.25)" stroke="#16a34a" stroke-width="1.5" /></svg>` as unknown as T;
    }
    if (cmd === "pdf_sign_pades") {
      const orig = (args?.doc_bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "pdf_validate_signatures") {
      const mockSigs: PdfSignatureValidationDto[] = [
        {
          field_name: "Signature1",
          is_valid: true,
          signer_common_name: "AetherOffice Verified Signer",
          signer_organization: "AetherOffice Security",
          has_ltv_dss: true,
          has_timestamp: false,
          signing_time: 1700000000,
          byte_range: [0, 1000, 1500, 2000],
          document_hash: "abcd1234ef567890",
          error_message: null,
        },
      ];
      return mockSigs as unknown as T;
    }
    if (cmd === "pdf_apply_redactions") {
      const orig = (args?.doc_bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      const report: PdfRedactionReportDto = {
        total_redactions_applied: 1,
        pages_redacted: [0],
        purged_strings: ["CONFIDENTIAL"],
        forensically_sanitized: true,
      };
      return [new Uint8Array(orig), report] as unknown as T;
    }
    if (cmd === "pdf_export_pdfa") {
      const orig = (args?.doc_bytes as number[]) || [0x25, 0x50, 0x44, 0x46];
      return new Uint8Array(orig) as unknown as T;
    }
    if (cmd === "dep_graph_register_dependency") {
      const edge = args?.edge as DependencyEdge;
      if (!edge || edge.source_doc_id === edge.target_doc_id) {
        throw new Error(`Circular dependency detected: ${edge?.source_doc_id}`);
      }
      mockEdges.set(edge.id, edge);
      return edge.id as unknown as T;
    }
    if (cmd === "dep_graph_remove_dependency") {
      const edgeId = (args?.edge_id as string) || "";
      const existed = mockEdges.delete(edgeId);
      mockSnapshots.delete(edgeId);
      return existed as unknown as T;
    }
    if (cmd === "dep_graph_get_dependents") {
      const docId = (args?.doc_id as string) || "";
      const results: DependencyEdge[] = [];
      for (const edge of mockEdges.values()) {
        if (edge.source_doc_id === docId) results.push(edge);
      }
      return results as unknown as T;
    }
    if (cmd === "dep_graph_get_dependencies") {
      const docId = (args?.doc_id as string) || "";
      const results: DependencyEdge[] = [];
      for (const edge of mockEdges.values()) {
        if (edge.target_doc_id === docId) results.push(edge);
      }
      return results as unknown as T;
    }
    if (cmd === "dep_graph_propagate_invalidation") {
      const sourceDocId = (args?.source_doc_id as string) || "";
      const affectedAnchor = args?.affected_anchor as EmbedAnchor | undefined;
      const affectedDocs: string[] = [];
      for (const edge of mockEdges.values()) {
        if (edge.source_doc_id === sourceDocId) {
          if (!affectedAnchor || JSON.stringify(edge.source_anchor) === JSON.stringify(affectedAnchor)) {
            if (!affectedDocs.includes(edge.target_doc_id)) {
              affectedDocs.push(edge.target_doc_id);
            }
            const snap = mockSnapshots.get(edge.id);
            if (snap) {
              snap.is_stale = true;
            }
          }
        }
      }
      return affectedDocs as unknown as T;
    }
    if (cmd === "dep_graph_update_snapshot") {
      const snap = args?.snapshot as LiveEmbedSnapshot;
      if (snap) {
        mockSnapshots.set(snap.embed_id, snap);
      }
      return undefined as unknown as T;
    }
    if (cmd === "dep_graph_get_snapshot") {
      const embedId = (args?.embed_id as string) || "";
      return (mockSnapshots.get(embedId) || null) as unknown as T;
    }
    if (cmd === "dep_graph_summary") {
      const docSet = new Set<string>();
      for (const edge of mockEdges.values()) {
        docSet.add(edge.source_doc_id);
        docSet.add(edge.target_doc_id);
      }
      let staleCount = 0;
      for (const snap of mockSnapshots.values()) {
        if (snap.is_stale) staleCount++;
      }
      const summary: CrossDocSummaryDto = {
        total_nodes: docSet.size,
        total_edges: mockEdges.size,
        total_snapshots: mockSnapshots.size,
        stale_snapshots: staleCount,
      };
      return summary as unknown as T;
    }
    if (cmd === "wikilink_parse_links") {
      const text = (args?.text as string) || "";
      const regex = /\[\[([^\]\n\r]+)\]\]/g;
      const links: WikiLink[] = [];
      let match: RegExpExecArray | null;
      while ((match = regex.exec(text)) !== null) {
        const raw = match[0];
        const inner = match[1].trim();
        let targetDoc = inner;
        let anchor: string | null = null;
        let alias: string | null = null;

        const pipeIdx = inner.indexOf("|");
        if (pipeIdx >= 0) {
          targetDoc = inner.substring(0, pipeIdx).trim();
          alias = inner.substring(pipeIdx + 1).trim() || null;
        }

        const hashIdx = targetDoc.indexOf("#");
        if (hashIdx >= 0) {
          anchor = targetDoc.substring(hashIdx + 1).trim() || null;
          targetDoc = targetDoc.substring(0, hashIdx).trim();
        }

        links.push({
          target_doc: targetDoc,
          target_anchor: anchor,
          display_text: alias,
          start_offset: match.index,
          end_offset: match.index + raw.length,
          raw_text: raw,
        });
      }
      return links as unknown as T;
    }
    if (cmd === "wikilink_index_document") {
      const docId = (args?.doc_id as string) || "";
      const title = (args?.title as string) || "";
      const content = (args?.content as string) || "";
      const regex = /\[\[([^\]\n\r]+)\]\]/g;
      const links: WikiLink[] = [];
      let match: RegExpExecArray | null;
      while ((match = regex.exec(content)) !== null) {
        const raw = match[0];
        const inner = match[1].trim();
        let targetDoc = inner;
        let anchor: string | null = null;
        let alias: string | null = null;
        const pipeIdx = inner.indexOf("|");
        if (pipeIdx >= 0) {
          targetDoc = inner.substring(0, pipeIdx).trim();
          alias = inner.substring(pipeIdx + 1).trim() || null;
        }
        const hashIdx = targetDoc.indexOf("#");
        if (hashIdx >= 0) {
          anchor = targetDoc.substring(hashIdx + 1).trim() || null;
          targetDoc = targetDoc.substring(0, hashIdx).trim();
        }
        links.push({
          target_doc: targetDoc,
          target_anchor: anchor,
          display_text: alias,
          start_offset: match.index,
          end_offset: match.index + raw.length,
          raw_text: raw,
        });
      }
      mockIndexedDocs.set(docId, { title, content, links });
      return undefined as unknown as T;
    }
    if (cmd === "wikilink_remove_document") {
      const docId = (args?.doc_id as string) || "";
      mockIndexedDocs.delete(docId);
      return undefined as unknown as T;
    }
    if (cmd === "wikilink_get_backlinks") {
      const target = ((args?.target_title_or_id as string) || "").toLowerCase().trim();
      const backlinks: BacklinkReference[] = [];
      for (const [docId, doc] of mockIndexedDocs.entries()) {
        for (const link of doc.links) {
          if (link.target_doc.toLowerCase().trim() === target || docId === target) {
            backlinks.push({
              source_doc_id: docId,
              source_doc_title: doc.title,
              target_anchor: link.target_anchor,
              display_text: link.display_text,
              snippet: doc.content.substring(Math.max(0, link.start_offset - 20), Math.min(doc.content.length, link.end_offset + 20)),
              start_offset: link.start_offset,
              end_offset: link.end_offset,
            });
          }
        }
      }
      return backlinks as unknown as T;
    }
    if (cmd === "wikilink_get_forward_links") {
      const docId = (args?.source_doc_id as string) || "";
      const doc = mockIndexedDocs.get(docId);
      return (doc ? doc.links : []) as unknown as T;
    }
    if (cmd === "wikilink_suggest_targets") {
      const q = ((args?.query as string) || "").toLowerCase().trim();
      const limit = (args?.limit as number) || 10;
      const suggestions: WikiLinkSuggestion[] = [];
      for (const [docId, doc] of mockIndexedDocs.entries()) {
        const titleLower = doc.title.toLowerCase();
        let score = 0;
        if (!q) score = 1;
        else if (titleLower === q) score = 100;
        else if (titleLower.startsWith(q)) score = 80;
        else if (titleLower.includes(q)) score = 50;
        if (score > 0) {
          suggestions.push({
            doc_id: docId,
            title: doc.title,
            anchor: null,
            preview: doc.content.substring(0, 80),
            score,
          });
        }
      }
      suggestions.sort((a, b) => b.score - a.score);
      return suggestions.slice(0, limit) as unknown as T;
    }
    if (cmd === "wikilink_find_unlinked_mentions") {
      const targetTitle = ((args?.target_title as string) || "").toLowerCase().trim();
      const mentions: UnlinkedMention[] = [];
      if (!targetTitle) return mentions as unknown as T;
      for (const [docId, doc] of mockIndexedDocs.entries()) {
        if (doc.title.toLowerCase().trim() === targetTitle) continue;
        const contentLower = doc.content.toLowerCase();
        let idx = 0;
        while ((idx = contentLower.indexOf(targetTitle, idx)) !== -1) {
          const endIdx = idx + targetTitle.length;
          const isLinked = doc.links.some((l) => idx >= l.start_offset && idx < l.end_offset);
          if (!isLinked) {
            mentions.push({
              source_doc_id: docId,
              source_doc_title: doc.title,
              matched_text: doc.content.substring(idx, endIdx),
              snippet: doc.content.substring(Math.max(0, idx - 20), Math.min(doc.content.length, endIdx + 20)),
              start_offset: idx,
              end_offset: endIdx,
            });
          }
          idx = endIdx;
        }
      }
      return mentions as unknown as T;
    }
    if (cmd === "wikilink_stats") {
      let totalLinks = 0;
      const targets = new Set<string>();
      for (const doc of mockIndexedDocs.values()) {
        totalLinks += doc.links.length;
        for (const l of doc.links) {
          targets.add(l.target_doc.toLowerCase());
        }
      }
      const stats: WikiLinkIndexStats = {
        total_documents: mockIndexedDocs.size,
        total_wikilinks: totalLinks,
        total_unique_targets: targets.size,
      };
      return stats as unknown as T;
    }
    if (cmd === "conversion_supported_formats") {
      const from = (args?.from as ConversionFormat) || "Markdown";
      const formatMap: Record<ConversionFormat, ConversionFormat[]> = {
        Markdown: ["Html", "Docx", "PlainText", "Pdf"],
        Html: ["Markdown", "PlainText", "Docx", "Pdf"],
        Docx: ["Markdown", "PlainText", "Html", "Pdf"],
        Csv: ["Xlsx", "Json", "PlainText"],
        Xlsx: ["Csv", "Json", "PlainText"],
        Json: ["Csv", "PlainText"],
        PlainText: ["Markdown", "Html", "Docx", "Pdf"],
        Pptx: ["PlainText", "Pdf"],
        Pdf: ["PlainText"],
      };
      return (formatMap[from] || []) as unknown as T;
    }
    if (cmd === "conversion_estimate_job") {
      const taskCount = (args?.task_count as number) || 0;
      const totalBytes = (args?.total_bytes as number) || 0;
      const durationMs = taskCount * 10 + Math.max(1, Math.floor(totalBytes / 1024) * 5);
      const estimate: ConversionEstimateDto = {
        estimated_duration_ms: durationMs,
        task_count: taskCount,
        total_bytes: totalBytes,
      };
      return estimate as unknown as T;
    }
    if (cmd === "conversion_convert_single") {
      const task = args?.task as ConversionTask;
      const inputStr = new TextDecoder().decode(new Uint8Array(task.input_bytes || []));
      let outStr = inputStr;
      if (task.source_format === "Markdown" && task.target_format === "Html") {
        outStr = `<p>${inputStr.replace(/^#\s*(.*)$/gm, "<h1>$1</h1>")}</p>`;
      } else if (task.source_format === "Html" && task.target_format === "Markdown") {
        outStr = inputStr.replace(/<h1>(.*?)<\/h1>/gi, "# $1\n").replace(/<[^>]+>/g, "");
      } else if (task.source_format === "Csv" && task.target_format === "Json") {
        const lines = inputStr.trim().split("\n");
        const headers = lines[0]?.split(",") || [];
        const rows = lines.slice(1).map((line) => {
          const cells = line.split(",");
          const obj: Record<string, string> = {};
          headers.forEach((h, i) => {
            obj[h.trim()] = cells[i]?.trim() || "";
          });
          return obj;
        });
        outStr = JSON.stringify(rows, null, 2);
      }
      const outBytes = Array.from(new TextEncoder().encode(outStr));
      const res: ConversionResult = {
        task_id: task.id,
        success: true,
        source_format: task.source_format,
        target_format: task.target_format,
        output_bytes: outBytes,
        error_message: null,
        duration_ms: 12,
      };
      return res as unknown as T;
    }
    if (cmd === "conversion_batch_convert") {
      const tasks = (args?.tasks as ConversionTask[]) || [];
      const results: ConversionResult[] = [];
      let succ = 0;
      let fail = 0;
      for (const t of tasks) {
        const inputStr = new TextDecoder().decode(new Uint8Array(t.input_bytes || []));
        const outBytes = Array.from(new TextEncoder().encode(inputStr));
        results.push({
          task_id: t.id,
          success: true,
          source_format: t.source_format,
          target_format: t.target_format,
          output_bytes: outBytes,
          error_message: null,
          duration_ms: 10,
        });
        succ++;
      }
      const report: BatchConversionReport = {
        total_tasks: tasks.length,
        succeeded: succ,
        failed: fail,
        results,
        duration_total_ms: 10 * tasks.length,
      };
      return report as unknown as T;
    }
    if (cmd === "oplog_append_entry") {
      const docId = (args?.doc_id as string) || "doc-1";
      const authorId = (args?.author_id as string) || "author-1";
      const payload = args?.payload as OpPayloadDto;
      const timestampMs = (args?.timestamp_ms as number) || Date.now();
      const clientId = (args?.client_id as number) || 1;

      const entries = mockOpLogs.get(docId) || [];
      const parentIds: OpIdDto[] =
        entries.length > 0
          ? [entries[entries.length - 1].entry_id]
          : [{ lamport: 0, client_id: 0, seq: 0 }];
      const sv = mockStateVectors.get(docId) || {};
      const nextSeq = (sv[clientId] || 0) + 1;
      sv[clientId] = nextSeq;
      mockStateVectors.set(docId, sv);

      const entryId: OpIdDto = {
        lamport: entries.length + 1,
        client_id: clientId,
        seq: nextSeq,
      };
      const entry: OpLogEntryDto = {
        entry_id: entryId,
        parent_ids: parentIds,
        timestamp_ms: timestampMs,
        author_id: authorId,
        doc_id: docId,
        payload,
        hash: `mock-hash-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`,
      };
      entries.push(entry);
      mockOpLogs.set(docId, entries);

      if ("SnapshotCheckpoint" in payload) {
        const cpPayload = payload.SnapshotCheckpoint;
        const cps = mockCheckpoints.get(docId) || [];
        cps.push({
          checkpoint_id: cpPayload.checkpoint_id,
          doc_id: docId,
          label: cpPayload.label,
          op_id: entryId,
          timestamp_ms: timestampMs,
          author_id: authorId,
          snapshot_preview: cpPayload.label,
        });
        mockCheckpoints.set(docId, cps);
      }

      return entry as unknown as T;
    }
    if (cmd === "oplog_get_history") {
      const docId = (args?.doc_id as string) || "doc-1";
      const entries = mockOpLogs.get(docId) || [];
      const off = (args?.offset as number) || 0;
      const lim = (args?.limit as number) || entries.length;
      return entries.slice(off, off + lim) as unknown as T;
    }
    if (cmd === "oplog_time_travel") {
      const docId = (args?.doc_id as string) || "doc-1";
      const targetTime = (args?.target_timestamp_ms as number) || Date.now();
      const entries = (mockOpLogs.get(docId) || []).filter((e) => e.timestamp_ms <= targetTime);
      let text = "";
      const cells: Record<string, unknown> = {};
      const shapes: Record<string, string> = {};
      const activeCheckpoints: string[] = [];

      for (const e of entries) {
        if ("TextInsert" in e.payload) {
          text += e.payload.TextInsert.content;
        } else if ("TextDelete" in e.payload) {
          text = text.slice(0, -1);
        } else if ("CellUpdate" in e.payload) {
          const { row, col, value } = e.payload.CellUpdate;
          cells[`${row},${col}`] = value;
        } else if ("ShapeMutation" in e.payload) {
          const { shape_id, properties_json } = e.payload.ShapeMutation;
          shapes[shape_id] = properties_json;
        } else if ("SnapshotCheckpoint" in e.payload) {
          activeCheckpoints.push(e.payload.SnapshotCheckpoint.checkpoint_id);
        }
      }

      const res: MaterializedDocumentStateDto = {
        doc_id: docId,
        timestamp_ms: targetTime,
        text_content: text,
        cells,
        shapes,
        active_checkpoints: activeCheckpoints,
        total_ops_replayed: entries.length,
        last_op_id: entries.length > 0 ? entries[entries.length - 1].entry_id : null,
      };
      return res as unknown as T;
    }
    if (cmd === "oplog_create_checkpoint") {
      const docId = (args?.doc_id as string) || "doc-1";
      const authorId = (args?.author_id as string) || "author-1";
      const label = (args?.label as string) || "Checkpoint";
      const clientId = (args?.client_id as number) || 1;
      const timestampMs = (args?.timestamp_ms as number) || Date.now();
      const cpId = `cp-${timestampMs}-${clientId}`;

      const cps = mockCheckpoints.get(docId) || [];
      const cp: CheckpointMetaDto = {
        checkpoint_id: cpId,
        doc_id: docId,
        label,
        op_id: { lamport: 1, client_id: clientId, seq: 1 },
        timestamp_ms: timestampMs,
        author_id: authorId,
        snapshot_preview: `Snapshot: ${label}`,
      };
      cps.push(cp);
      mockCheckpoints.set(docId, cps);
      return cp as unknown as T;
    }
    if (cmd === "oplog_get_checkpoints") {
      const docId = (args?.doc_id as string) || "doc-1";
      return (mockCheckpoints.get(docId) || []) as unknown as T;
    }
    if (cmd === "oplog_get_state_vector") {
      const docId = (args?.doc_id as string) || "doc-1";
      const clocks = mockStateVectors.get(docId) || {};
      const sv: StateVectorDto = { clocks };
      return sv as unknown as T;
    }
    if (cmd === "oplog_compute_sync_delta") {
      const docId = (args?.doc_id as string) || "doc-1";
      const remoteVector = (args?.remote_vector as StateVectorDto) || { clocks: {} };
      const entries = mockOpLogs.get(docId) || [];
      const deltas = entries.filter((e) => {
        const remoteSeq = remoteVector.clocks[e.entry_id.client_id] || 0;
        return e.entry_id.seq > remoteSeq;
      });
      return deltas as unknown as T;
    }
    if (cmd === "oplog_apply_remote_delta") {
      const docId = (args?.doc_id as string) || "doc-1";
      const remoteEntries = (args?.remote_entries as OpLogEntryDto[]) || [];
      const local = mockOpLogs.get(docId) || [];
      let applied = 0;
      let skipped = 0;
      for (const e of remoteEntries) {
        if (
          !local.some(
            (l) =>
              l.entry_id.client_id === e.entry_id.client_id &&
              l.entry_id.seq === e.entry_id.seq
          )
        ) {
          local.push(e);
          applied++;
        } else {
          skipped++;
        }
      }
      mockOpLogs.set(docId, local);
      const clocks = mockStateVectors.get(docId) || {};
      for (const e of remoteEntries) {
        clocks[e.entry_id.client_id] = Math.max(
          clocks[e.entry_id.client_id] || 0,
          e.entry_id.seq
        );
      }
      mockStateVectors.set(docId, clocks);
      const report: SyncReportDto = {
        applied,
        skipped,
        conflicts_resolved: 0,
        state_vector: { clocks },
      };
      return report as unknown as T;
    }
    if (cmd === "oplog_fork_version") {
      const newDocId = (args?.new_doc_id as string) || "forked-doc-1";
      const timeMs = (args?.timestamp_ms as number) || Date.now();
      const state: MaterializedDocumentStateDto = {
        doc_id: newDocId,
        timestamp_ms: timeMs,
        text_content: "Forked Content",
        cells: {},
        shapes: {},
        active_checkpoints: ["cp-forked-1"],
        total_ops_replayed: 1,
        last_op_id: { lamport: 1, client_id: 1, seq: 1 },
      };
      return state as unknown as T;
    }
    if (cmd === "plugin_load") {
      const manifestJson = (args?.manifest_json as string) || "{}";
      let parsed: {
        id?: string;
        name?: string;
        version?: string;
        author?: string;
        description?: string;
        capabilities?: string[];
        fuel_limit?: number;
      } = {};
      try {
        parsed = JSON.parse(manifestJson);
      } catch {
        // ignore fallback
      }
      const pluginId = parsed.id || `plugin-${Date.now()}`;
      const status: PluginStatusDto = {
        id: pluginId,
        name: parsed.name || pluginId,
        version: parsed.version || "0.1.0",
        author: parsed.author || "Unknown",
        description: parsed.description || "Plugin extension",
        state: "Loaded",
        capabilities: parsed.capabilities || ["read-document"],
        fuel_limit: parsed.fuel_limit || 10000,
        fuel_remaining: parsed.fuel_limit || 10000,
        dispatched_events: 0,
        exported_hooks: ["on_init", "on_save"],
      };
      mockPlugins.set(pluginId, status);
      return status as unknown as T;
    }
    if (cmd === "plugin_unload") {
      const pluginId = (args?.plugin_id as string) || "";
      const existed = mockPlugins.delete(pluginId);
      return existed as unknown as T;
    }
    if (cmd === "plugin_list") {
      return Array.from(mockPlugins.values()) as unknown as T;
    }
    if (cmd === "plugin_init") {
      const pluginId = (args?.plugin_id as string) || "";
      const p = mockPlugins.get(pluginId);
      if (!p) throw new Error(`Plugin '${pluginId}' not found`);
      p.state = "Active";
      p.fuel_remaining = Math.max(0, p.fuel_remaining - 500);
      p.dispatched_events += 1;
      return p as unknown as T;
    }
    if (cmd === "plugin_suspend") {
      const pluginId = (args?.plugin_id as string) || "";
      const p = mockPlugins.get(pluginId);
      if (!p) throw new Error(`Plugin '${pluginId}' not found`);
      p.state = "Suspended";
      return p as unknown as T;
    }
    if (cmd === "plugin_resume") {
      const pluginId = (args?.plugin_id as string) || "";
      const p = mockPlugins.get(pluginId);
      if (!p) throw new Error(`Plugin '${pluginId}' not found`);
      p.state = "Active";
      return p as unknown as T;
    }
    if (cmd === "plugin_refill_fuel") {
      const pluginId = (args?.plugin_id as string) || "";
      const additional = (args?.additional_fuel as number) || 0;
      const p = mockPlugins.get(pluginId);
      if (!p) throw new Error(`Plugin '${pluginId}' not found`);
      p.fuel_remaining += additional;
      return p.fuel_remaining as unknown as T;
    }
    if (cmd === "plugin_execute_action") {
      const pluginId = (args?.plugin_id as string) || "";
      const action = args?.action as PluginActionDto;
      const p = mockPlugins.get(pluginId);
      if (!p) throw new Error(`Plugin '${pluginId}' not found`);
      if (p.state !== "Active") {
        throw new Error(`Plugin '${pluginId}' is not active (state: ${p.state})`);
      }
      p.fuel_remaining = Math.max(0, p.fuel_remaining - 100);

      let res: ActionResultDto = "MutationSuccess";
      if (action && typeof action === "object") {
        if ("QuerySelection" in action) {
          res = {
            Selection: {
              doc_id: action.QuerySelection.doc_id,
              doc_type: "Word",
              start_offset: 0,
              end_offset: 10,
              selected_text: "Mock selection text",
            },
          };
        } else if ("QueryCellValue" in action) {
          res = { CellValue: "123.45" };
        } else if ("QueryParagraph" in action) {
          res = { ParagraphText: "Mock paragraph text at index " + action.QueryParagraph.para_index };
        } else if ("QuerySlideShapes" in action) {
          res = { SlideShapes: ["shape-1", "shape-2"] };
        } else if ("ExecuteRibbonCommand" in action) {
          res = { RibbonSuccess: true };
        } else {
          res = "MutationSuccess";
        }
      }
      return res as unknown as T;
    }
    if (cmd === "plugin_register_mock_udf") {
      const meta = args?.meta as PluginUdfMetaDto;
      if (meta && meta.name) {
        mockUdfs.set(meta.name.toUpperCase(), meta);
      }
      return true as unknown as T;
    }
    if (cmd === "plugin_list_udfs") {
      return Array.from(mockUdfs.values()) as unknown as T;
    }
    if (cmd === "plugin_call_udf") {
      const funcName = ((args?.func_name as string) || "").toUpperCase();
      const fnArgs = (args?.args as string[]) || [];
      const udf = mockUdfs.get(funcName);
      if (!udf && funcName !== "GEO_DISTANCE" && funcName !== "SENTIMENT_SCORE") {
        return {
          function_name: funcName,
          success: false,
          result_value: null,
          error_message: `Unknown UDF function '${funcName}'`,
        } as unknown as T;
      }
      let val = "0";
      if (funcName === "GEO_DISTANCE") {
        val = "415.82";
      } else if (funcName === "SENTIMENT_SCORE") {
        val = "0.88";
      } else {
        val = fnArgs.length > 0 ? `EVAL(${fnArgs.join(",")})` : "OK";
      }
      return {
        function_name: funcName,
        success: true,
        result_value: val,
        error_message: null,
      } as unknown as T;
    }
    if (cmd === "copilot_get_provider_config") {
      const status: ProviderStatusDto = {
        provider_kind: mockProviderConfig.provider_kind,
        model_name: mockProviderConfig.model_name,
        endpoint: mockProviderConfig.base_url,
        is_available: true,
        latency_ms: 12,
      };
      return status as unknown as T;
    }
    if (cmd === "copilot_configure_provider") {
      const cfg = args?.config as ProviderConfigDto;
      if (cfg) {
        mockProviderConfig = { ...mockProviderConfig, ...cfg };
      }
      const status: ProviderStatusDto = {
        provider_kind: mockProviderConfig.provider_kind,
        model_name: mockProviderConfig.model_name,
        endpoint: mockProviderConfig.base_url,
        is_available: true,
        latency_ms: 12,
      };
      return status as unknown as T;
    }
    if (cmd === "copilot_index_document") {
      const docId = (args?.doc_id as string) || "doc-1";
      const title = (args?.title as string) || "Untitled";
      const text = (args?.text as string) || "";
      mockCopilotDocs.set(docId, { title, text });
      return 2 as unknown as T;
    }
    if (cmd === "copilot_remove_document") {
      const docId = (args?.doc_id as string) || "";
      const existed = mockCopilotDocs.delete(docId);
      return (existed ? 2 : 0) as unknown as T;
    }
    if (cmd === "copilot_chat") {
      const req = (args?.request || {}) as CopilotChatRequestDto;
      mockCopilotMessageCounter++;
      const query = req.query || "";
      const queryLower = query.toLowerCase();
      const sessionId = req.session_id || "doc-1";

      const citations: GroundedCitationDto[] = [];
      let citationIdx = 1;

      for (const [docId, docData] of mockCopilotDocs.entries()) {
        if (
          req.target_doc_ids &&
          req.target_doc_ids.length > 0 &&
          !req.target_doc_ids.includes(docId)
        ) {
          continue;
        }
        citations.push({
          marker: `[${citationIdx}]`,
          doc_id: docId,
          chunk_id: `chunk-${docId}-${citationIdx}`,
          snippet: docData.text.slice(0, 100),
          location: `Section ${citationIdx}`,
        });
        citationIdx++;
      }

      let suggestedDiff: GroundedDiffDto | null = null;
      let responseText = "";

      const isDiff =
        queryLower.includes("revise") ||
        queryLower.includes("rewrite") ||
        queryLower.includes("replace") ||
        queryLower.includes("suggest");

      if (isDiff && req.active_selection && req.active_selection.selected_text) {
        const orig = req.active_selection.selected_text;
        const revised = `Executive Summary: ${orig.trim()}`;
        suggestedDiff = {
          doc_id: sessionId,
          original: orig,
          modified: revised,
          target_element: req.active_selection.selected_range || null,
        };
        responseText = `I have analyzed your selection and prepared a proposed revision: "${revised}". Click [Apply Diff] to update your document.`;
      } else if (citations.length > 0) {
        responseText =
          `Based on your indexed documents, here is the synthesis for "${query}":\n\n` +
          citations
            .map((c) => `According to ${c.marker} (${c.doc_id}): "${c.snippet}..."`)
            .join("\n\n");
      } else {
        responseText = `Based on the currently indexed documents, no relevant passages were found matching "${query}".`;
      }

      const res: CopilotChatResponseDto = {
        message_id: `copilot-msg-${mockCopilotMessageCounter}`,
        response_text: responseText,
        citations,
        suggested_diff: suggestedDiff,
        confidence_score: citations.length > 0 ? 0.92 : 0.65,
        tokens_used: 180,
        model_used: mockProviderConfig.model_name,
      };
      return res as unknown as T;
    }
    if (cmd === "theme_get_tokens") {
      const mode = (args?.theme as ThemeMode) || mockCurrentTheme;
      const rMode = (args?.ribbon_mode as RibbonLayoutMode) || mockRibbonLayout.mode;
      return mockEvaluateThemeTokens(mode, rMode) as unknown as T;
    }
    if (cmd === "theme_set_mode") {
      const mode = (args?.mode as ThemeMode) || "light";
      mockCurrentTheme = mode;
      return mockEvaluateThemeTokens(mode, mockRibbonLayout.mode) as unknown as T;
    }
    if (cmd === "ribbon_set_layout_mode") {
      const mode = (args?.mode as RibbonLayoutMode) || "classic";
      const collapsed = mode === "collapsed";
      const is_single_row = mode === "compact";
      const show_group_headers = mode === "classic";
      const height_px = mode === "classic" ? 96 : mode === "compact" ? 44 : 32;
      mockRibbonLayout = {
        mode,
        collapsed,
        height_px,
        is_single_row,
        show_group_headers,
      };
      return mockRibbonLayout as unknown as T;
    }
    if (cmd === "ribbon_get_layout_config") {
      return mockRibbonLayout as unknown as T;
    }
    if (cmd === "accessibility_get_aria_tree") {
      return mockAriaTree as unknown as T;
    }
    if (cmd === "accessibility_set_aria_tree") {
      const tree = (args?.tree as AriaTreeDto) || mockAriaTree;
      mockAriaTree = tree;
      return (tree.total_nodes || 1) as unknown as T;
    }
    if (cmd === "ime_start_composition") {
      const cx = (args?.caret_x as number) || 0;
      const cy = (args?.caret_y as number) || 0;
      const ch = (args?.caret_h as number) || 18;
      mockImeState = {
        is_composing: true,
        preedit_string: "",
        cursor_position: 0,
        selection_start: 0,
        selection_end: 0,
        caret_x: cx,
        caret_y: cy,
        caret_height: ch,
        candidate_box: { x: cx, y: cy + ch, width: 220, height: 130 },
      };
      return mockImeState.candidate_box as unknown as T;
    }
    if (cmd === "ime_update_composition") {
      const text = (args?.text as string) || "";
      const cursor = (args?.cursor as number) || 0;
      const sel_start = (args?.sel_start as number) || 0;
      const sel_end = (args?.sel_end as number) || 0;
      const cx = (args?.caret_x as number) || mockImeState.caret_x;
      const cy = (args?.caret_y as number) || mockImeState.caret_y;
      const ch = (args?.caret_h as number) || mockImeState.caret_height;
      mockImeState = {
        is_composing: true,
        preedit_string: text,
        cursor_position: cursor,
        selection_start: sel_start,
        selection_end: sel_end,
        caret_x: cx,
        caret_y: cy,
        caret_height: ch,
        candidate_box: { x: cx, y: cy + ch, width: 220, height: 130 },
      };
      return mockImeState.candidate_box as unknown as T;
    }
    if (cmd === "ime_finish_composition") {
      mockImeState.is_composing = false;
      const committed = mockImeState.preedit_string;
      mockImeState.preedit_string = "";
      mockImeState.cursor_position = 0;
      return committed as unknown as T;
    }
    if (cmd === "ime_cancel_composition") {
      mockImeState.is_composing = false;
      mockImeState.preedit_string = "";
      mockImeState.cursor_position = 0;
      return null as unknown as T;
    }
    if (cmd === "bidi_analyze_text") {
      const text = (args?.text as string) || "";
      const isRtl = /[\u0590-\u05FF\u0600-\u06FF\u0750-\u077F\uFB50-\uFDFF\uFE70-\uFEFF]/.test(text);
      const base_direction: BidiDirection = isRtl ? "rtl" : /[a-zA-Z]/.test(text) ? "ltr" : "auto";
      const runs: BidiRunDto[] = [
        {
          text,
          direction: base_direction,
          start_idx: 0,
          length: text.length,
        },
      ];
      const res: BidiAnalysisResultDto = { base_direction, runs };
      return res as unknown as T;
    }
    if (cmd === "keyboard_nav_cycle_pane") {
      const reverse = (args?.reverse as boolean) || false;
      const order: NavFocusPane[] = ["title-bar", "ribbon", "side-pane", "canvas", "status-bar"];
      const currentIdx = order.indexOf(mockNavPane);
      const nextIdx = reverse
        ? (currentIdx - 1 + order.length) % order.length
        : (currentIdx + 1) % order.length;
      mockNavPane = order[nextIdx];
      return mockNavPane as unknown as T;
    }
    if (cmd === "gesture_process_touch_start") {
      const touch = args?.touch as TouchPointDto;
      if (touch) {
        mockActiveTouches.set(touch.id, touch);
      }
      return { type: "none" } as unknown as T;
    }
    if (cmd === "gesture_process_touch_move") {
      const touch = args?.touch as TouchPointDto;
      if (!touch) return { type: "none" } as unknown as T;
      const prev = mockActiveTouches.get(touch.id);
      mockActiveTouches.set(touch.id, touch);

      if (mockActiveTouches.size === 2) {
        const pts = Array.from(mockActiveTouches.values());
        const focal_x = (pts[0].x + pts[1].x) / 2;
        const focal_y = (pts[0].y + pts[1].y) / 2;
        return {
          type: "pinch-zoom",
          payload: { scale: 1.05, focal_x, focal_y },
        } as unknown as T;
      }
      if (mockActiveTouches.size === 1 && prev) {
        const dx = touch.x - prev.x;
        const dy = touch.y - prev.y;
        if (Math.abs(dx) > 1.5 || Math.abs(dy) > 1.5) {
          return {
            type: "pan-scroll",
            payload: { delta_x: dx, delta_y: dy },
          } as unknown as T;
        }
      }
      return { type: "none" } as unknown as T;
    }
    if (cmd === "gesture_process_touch_end") {
      const touchId = args?.touch_id as number;
      mockActiveTouches.delete(touchId);
      return { type: "none" } as unknown as T;
    }
    if (cmd === "gesture_process_touch_cancel") {
      const touchId = args?.touch_id as number;
      if (touchId === undefined || touchId === null || touchId <= 0) {
        mockActiveTouches.clear();
      } else {
        mockActiveTouches.delete(touchId);
      }
      return null as unknown as T;
    }
    if (cmd === "palm_rejection_filter_contact") {
      const touch = args?.touch as TouchPointDto;
      const activeStylus = args?.active_stylus as TouchPointDto | undefined;
      const screenW = (args?.screen_w as number) || 1920;
      const screenH = (args?.screen_h as number) || 1080;
      if (!touch) return false as unknown as T;
      if (touch.is_stylus) return false as unknown as T;
      const major = Math.max(touch.radius_x, touch.radius_y);
      const minor = Math.min(touch.radius_x, touch.radius_y);
      const area = Math.PI * major * minor;
      if (major > 22 || area > 450) return true as unknown as T;
      if (activeStylus) {
        const dist = Math.sqrt(Math.pow(touch.x - activeStylus.x, 2) + Math.pow(touch.y - activeStylus.y, 2));
        if (dist < 120) return true as unknown as T;
      }
      if (touch.x < 15 || touch.x > screenW - 15 || touch.y < 15 || touch.y > screenH - 15) {
        return true as unknown as T;
      }
      return false as unknown as T;
    }
    if (cmd === "adaptive_split_set_layout") {
      const modeStr = ((args?.mode_str as string) || "single").toLowerCase();
      let mode: SplitGridMode = "single";
      if (modeStr.includes("horizontal")) mode = "two-way-horizontal";
      else if (modeStr.includes("vertical")) mode = "two-way-vertical";
      else if (modeStr.includes("four") || modeStr.includes("grid")) mode = "four-way-grid";

      let rx = (args?.ratio_x as number) ?? mockSplitGrid.ratio_x;
      let ry = (args?.ratio_y as number) ?? mockSplitGrid.ratio_y;

      const snap = (v: number) => {
        for (const pt of [0.333, 0.500, 0.667]) {
          if (Math.abs(v - pt) <= 0.04) return pt;
        }
        return Math.min(0.85, Math.max(0.15, v));
      };

      mockSplitGrid = {
        mode,
        ratio_x: snap(rx),
        ratio_y: snap(ry),
        snap_threshold: 0.04,
      };
      return mockSplitGrid as unknown as T;
    }
    if (cmd === "adaptive_split_get_rects") {
      const totalWidth = (args?.total_width as number) || 1200;
      const totalHeight = (args?.total_height as number) || 800;
      const insetTop = (args?.inset_top as number) || 0;
      const insetBottom = (args?.inset_bottom as number) || 0;
      const safeW = totalWidth;
      const safeH = totalHeight - insetTop - insetBottom;
      const safeX = 0;
      const safeY = insetTop;

      let rects: Rect2DDto[] = [];
      if (mockSplitGrid.mode === "single") {
        rects = [{ x: safeX, y: safeY, width: safeW, height: safeH }];
      } else if (mockSplitGrid.mode === "two-way-horizontal") {
        const w1 = safeW * mockSplitGrid.ratio_x;
        rects = [
          { x: safeX, y: safeY, width: w1, height: safeH },
          { x: safeX + w1, y: safeY, width: safeW - w1, height: safeH },
        ];
      } else if (mockSplitGrid.mode === "two-way-vertical") {
        const h1 = safeH * mockSplitGrid.ratio_y;
        rects = [
          { x: safeX, y: safeY, width: safeW, height: h1 },
          { x: safeX, y: safeY + h1, width: safeW, height: safeH - h1 },
        ];
      } else if (mockSplitGrid.mode === "four-way-grid") {
        const w1 = safeW * mockSplitGrid.ratio_x;
        const w2 = safeW - w1;
        const h1 = safeH * mockSplitGrid.ratio_y;
        const h2 = safeH - h1;
        rects = [
          { x: safeX, y: safeY, width: w1, height: h1 },
          { x: safeX + w1, y: safeY, width: w2, height: h1 },
          { x: safeX, y: safeY + h1, width: w1, height: h2 },
          { x: safeX + w1, y: safeY + h1, width: w2, height: h2 },
        ];
      }
      return rects as unknown as T;
    }
    if (cmd === "dlp_evaluate_text") {
      const text = (args?.text as string) || "";
      const matches: DlpMatchDto[] = [];
      const triggered: string[] = [];
      let action: DlpAction = mockDlpPolicy.default_action;
      let highestSev: DlpSeverity | null = null;

      if (/\b\d{3}-\d{2}-\d{4}\b/.test(text)) {
        matches.push({
          rule_id: "rule-ssn-redact",
          pattern_label: "SSN",
          matched_text: "123-45-6789",
          start: 0,
          end: 11,
          severity: "high",
          replacement: "[REDACTED: SSN]",
        });
        triggered.push("rule-ssn-redact");
        highestSev = "high";
        action = "redact-and-allow";
      }
      if (/4532[\s-]?0150[\s-]?0000[\s-]?0007/.test(text)) {
        matches.push({
          rule_id: "rule-pci-dss",
          pattern_label: "CREDIT_CARD",
          matched_text: "4532 0150 0000 0007",
          start: 0,
          end: 19,
          severity: "critical",
          replacement: "[REDACTED: CREDIT_CARD]",
        });
        triggered.push("rule-pci-dss");
        highestSev = "critical";
        action = "block";
      }

      const res: DlpEvaluationResultDto = {
        policy_id: mockDlpPolicy.id,
        policy_version: mockDlpPolicy.version,
        action,
        highest_severity: highestSev,
        matches,
        triggered_rule_ids: triggered,
        is_violation: triggered.length > 0,
        summary: triggered.length > 0 ? `Triggered ${triggered.length} rule(s)` : "Passed with zero violations",
      };
      return res as unknown as T;
    }
    if (cmd === "dlp_evaluate_and_redact") {
      const text = (args?.text as string) || "";
      let redacted = text.replace(/\b\d{3}-\d{2}-\d{4}\b/g, "[REDACTED: SSN]");
      redacted = redacted.replace(/4532[\s-]?0150[\s-]?0000[\s-]?0007/g, "[REDACTED: CREDIT_CARD]");
      const isVio = redacted !== text;
      const res: DlpEvaluationResultDto = {
        policy_id: mockDlpPolicy.id,
        policy_version: mockDlpPolicy.version,
        action: isVio ? "redact-and-allow" : mockDlpPolicy.default_action,
        highest_severity: isVio ? "high" : null,
        matches: isVio
          ? [
              {
                rule_id: "rule-ssn-redact",
                pattern_label: "SSN",
                matched_text: "123-45-6789",
                start: 0,
                end: 11,
                severity: "high",
                replacement: "[REDACTED: SSN]",
              },
            ]
          : [],
        triggered_rule_ids: isVio ? ["rule-ssn-redact"] : [],
        is_violation: isVio,
        summary: isVio ? "Redacted sensitive entities" : "Passed with zero violations",
      };
      return [res, redacted] as unknown as T;
    }
    if (cmd === "dlp_set_policy") {
      const p = args?.policy as DlpPolicyDto;
      if (p) mockDlpPolicy = p;
      return null as unknown as T;
    }
    if (cmd === "dlp_get_policy") {
      return mockDlpPolicy as unknown as T;
    }
    if (cmd === "sanitization_generate_certificate") {
      const docId = (args?.doc_id as string) || "doc-1";
      const policyId = (args?.policy_id as string) || mockDlpPolicy.id;
      const ts = (args?.timestamp_ms as number) || Date.now();
      const redactions = (args?.redactions as Array<[string, number]>) || [];
      const cert: SanitizationCertificateDto = {
        certificate_id: `cert-${ts}-mock`,
        doc_id: docId,
        original_digest: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        sanitized_digest: "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
        policy_id: policyId,
        sanitized_at_ms: ts,
        redactions_count: redactions,
        signature: "5d41402abc4b2a76b9719d911017c5925a92a54826b5275a5e3c75d424b455b8",
      };
      return cert as unknown as T;
    }
    if (cmd === "sanitization_verify_certificate") {
      return true as unknown as T;
    }
    if (cmd === "audit_log_append") {
      const seq = mockAuditEntries.length + 1;
      const prev =
        mockAuditEntries.length > 0
          ? mockAuditEntries[mockAuditEntries.length - 1].entry_hash
          : "0000000000000000000000000000000000000000000000000000000000000000";
      const entry: AuditLogEntryDto = {
        sequence_number: seq,
        timestamp_ms: (args?.timestamp_ms as number) || Date.now(),
        event_type: args?.event_type as AuditEventTypeDto,
        user_id: (args?.user_id as string) || "user",
        doc_id: (args?.doc_id as string) || "doc",
        details: (args?.details as string) || "",
        prev_hash: prev,
        entry_hash: `hash-mock-${seq}-${Date.now()}`,
      };
      mockAuditEntries.push(entry);
      return entry as unknown as T;
    }
    if (cmd === "audit_log_get_entries") {
      const docId = args?.doc_id as string | undefined;
      if (docId) {
        return mockAuditEntries.filter((e) => e.doc_id === docId) as unknown as T;
      }
      return [...mockAuditEntries] as unknown as T;
    }
    if (cmd === "audit_log_verify_integrity") {
      return mockAuditEntries.length as unknown as T;
    }
    if (cmd === "entitlement_get_tier") {
      if (mockActiveLicense) {
        const now = Date.now();
        if (
          !mockActiveLicense.expires_at_ms ||
          now <= mockActiveLicense.expires_at_ms + 14 * 24 * 60 * 60 * 1000
        ) {
          return mockActiveLicense.tier as unknown as T;
        }
      }
      return mockEditionTier as unknown as T;
    }
    if (cmd === "entitlement_check_feature") {
      const feat = (args?.feature as string) || "";
      const now = (args?.now_ms as number) || Date.now();
      let effectiveTier = mockEditionTier;
      if (mockActiveLicense) {
        if (
          !mockActiveLicense.expires_at_ms ||
          now <= mockActiveLicense.expires_at_ms + 14 * 24 * 60 * 60 * 1000
        ) {
          effectiveTier = mockActiveLicense.tier;
        }
      }

      // Check custom features
      if (mockActiveLicense && mockActiveLicense.custom_features.includes(feat)) {
        if (mockActiveLicense.expires_at_ms && now > mockActiveLicense.expires_at_ms) {
          const daysLeft = Math.max(
            0,
            Math.floor(
              (mockActiveLicense.expires_at_ms + 14 * 24 * 60 * 60 * 1000 - now) /
                (24 * 60 * 60 * 1000)
            )
          );
          return {
            status: "grace-period",
            days_remaining: daysLeft,
            message: `Grace period active for custom feature '${feat}'`,
          } as unknown as T;
        }
        return { status: "granted" } as unknown as T;
      }

      const requiredTierMap: Record<string, EditionTierDto> = {
        "basic-editing": "community",
        "advanced-typography": "community",
        "standard-formulas": "community",
        "full-document-io": "community",
        "local-ai-copilot": "professional",
        "macro-debugger": "professional",
        "batch-conversion": "professional",
        "enterprise-dlp": "enterprise",
        "sanitization-certificates": "enterprise",
        "tamper-evident-audit-logs": "enterprise",
        "realtime-collab-webrtc": "enterprise",
        "custom-wasm-plugins": "enterprise",
      };
      const req = requiredTierMap[feat] || "community";
      const tierRank: Record<EditionTierDto, number> = {
        community: 1,
        professional: 2,
        enterprise: 3,
      };

      if (tierRank[effectiveTier] >= tierRank[req]) {
        if (
          mockActiveLicense &&
          mockActiveLicense.expires_at_ms &&
          now > mockActiveLicense.expires_at_ms
        ) {
          const daysLeft = Math.max(
            0,
            Math.floor(
              (mockActiveLicense.expires_at_ms + 14 * 24 * 60 * 60 * 1000 - now) /
                (24 * 60 * 60 * 1000)
            )
          );
          return {
            status: "grace-period",
            days_remaining: daysLeft,
            message: `License in grace period (${daysLeft} day(s) remaining)`,
          } as unknown as T;
        }
        return { status: "granted" } as unknown as T;
      }
      return {
        status: "denied",
        required_tier: req,
        reason: `Feature '${feat}' requires ${req} tier (Current: ${effectiveTier})`,
      } as unknown as T;
    }
    if (cmd === "entitlement_apply_license") {
      const token = args?.token as LicenseTokenDto;
      const key = (args?.verification_key as number[]) || [];
      const now = (args?.now_ms as number) || Date.now();
      if (!token || !token.signature || key.length === 0) {
        throw new Error("Invalid license signature: cryptographic verification failed");
      }
      if (token.expires_at_ms && now > token.expires_at_ms) {
        const graceEnd = token.expires_at_ms + 14 * 24 * 60 * 60 * 1000;
        if (now <= graceEnd) {
          mockActiveLicense = token;
          mockEditionTier = token.tier;
          const daysLeft = Math.floor((graceEnd - now) / (24 * 60 * 60 * 1000));
          return {
            status: "grace-period",
            days_remaining: daysLeft,
            message: `License expired. Operating in grace period with ${daysLeft} day(s) remaining.`,
          } as unknown as T;
        } else {
          throw new Error("License expired beyond allowable grace period");
        }
      }
      mockActiveLicense = token;
      mockEditionTier = token.tier;
      return { status: "granted" } as unknown as T;
    }
    if (cmd === "entitlement_get_license_info") {
      return (mockActiveLicense ? { ...mockActiveLicense } : null) as unknown as T;
    }
    if (cmd === "entitlement_set_tier") {
      const tier = (args?.tier as EditionTierDto) || "community";
      mockEditionTier = tier;
      return tier as unknown as T;
    }
    if (cmd === "entitlement_remove_license") {
      mockActiveLicense = null;
      mockEditionTier = "community";
      return true as unknown as T;
    }
    if (cmd === "sla_run_benchmarks") {
      const report: SlaBenchmarkReportDto = {
        timestamp_ms: Date.now(),
        all_passed: true,
        metrics: [
          {
            metric_name: "cold_start_latency",
            measured_value: 42.0,
            budget_threshold: 100.0,
            unit: "ms",
            passed: true,
            p50_ms: 42.0,
            p99_ms: 42.0,
          },
          {
            metric_name: "keystroke_cell_edit_latency",
            measured_value: 0.45,
            budget_threshold: 1.0,
            unit: "ms",
            passed: true,
            p50_ms: 0.32,
            p99_ms: 0.45,
          },
          {
            metric_name: "frame_render_120fps_budget",
            measured_value: 4.8,
            budget_threshold: 8.333,
            unit: "ms",
            passed: true,
            p50_ms: 3.5,
            p99_ms: 4.8,
          },
          {
            metric_name: "idle_memory_footprint",
            measured_value: 18.5,
            budget_threshold: 35.0,
            unit: "MB",
            passed: true,
          },
          {
            metric_name: "active_20_session_memory",
            measured_value: 125.0,
            budget_threshold: 250.0,
            unit: "MB",
            passed: true,
          },
        ],
        summary: "SLA Certification: 5/5 metrics passed (Status: COMPLIANT)",
      };
      return report as unknown as T;
    }
    if (cmd === "sla_verify_budgets") {
      const report = args?.report as SlaBenchmarkReportDto;
      const limits = (args?.thresholds as SlaBudgetThresholdsDto) || {
        max_cold_start_ms: 100.0,
        max_idle_memory_mb: 35.0,
        max_active_session_memory_mb: 250.0,
        max_keystroke_latency_ms: 1.0,
        max_frame_render_latency_ms: 8.333,
      };
      if (!report || !report.metrics) return false as unknown as T;
      for (const m of report.metrics) {
        if (m.metric_name === "cold_start_latency" && m.measured_value > limits.max_cold_start_ms) {
          return false as unknown as T;
        }
        if (m.metric_name === "idle_memory_footprint" && m.measured_value > limits.max_idle_memory_mb) {
          return false as unknown as T;
        }
        if (
          m.metric_name === "active_20_session_memory" &&
          m.measured_value > limits.max_active_session_memory_mb
        ) {
          return false as unknown as T;
        }
        if (
          m.metric_name === "keystroke_cell_edit_latency" &&
          m.measured_value > limits.max_keystroke_latency_ms
        ) {
          return false as unknown as T;
        }
        if (
          m.metric_name === "frame_render_120fps_budget" &&
          m.measured_value > limits.max_frame_render_latency_ms
        ) {
          return false as unknown as T;
        }
      }
      return report.all_passed as unknown as T;
    }
    if (cmd === "fuzz_test_document_buffer") {
      const payload = (args?.payload as number[]) || [];
      const mutation = (args?.mutation_type as string) || "truncated-zip";
      if (mutation.includes("truncated") && payload.length > 4) {
        return payload.slice(0, Math.floor(payload.length / 2)) as unknown as T;
      }
      return [...payload, 0xff, 0x00] as unknown as T;
    }
    if (cmd === "differential_audit_evaluate") {
      const base = args?.base_display as {
        commands?: unknown[];
        bounds?: { x: number; y: number; width: number; height: number };
      };
      const candidate = args?.candidate_display as {
        commands?: unknown[];
        bounds?: { x: number; y: number; width: number; height: number };
      };
      const divergences: string[] = [];
      const baseCmds = base?.commands?.length || 0;
      const candCmds = candidate?.commands?.length || 0;
      if (baseCmds !== candCmds) {
        divergences.push(`Command count mismatch: Base has ${baseCmds}, Candidate has ${candCmds}`);
      }
      const res: DifferentialAuditResultDto = {
        is_identical: divergences.length === 0,
        total_base_commands: baseCmds,
        total_candidate_commands: candCmds,
        divergences,
      };
      return res as unknown as T;
    }
    if (cmd === "release_matrix_get_manifest") {
      const manifest: InstallerMatrixManifestDto = {
        app_name: "AetherOffice",
        version: "0.1.0",
        release_date_iso: "2026-11-01T00:00:00Z",
        artifacts: [
          {
            id: "win-x64-msi",
            platform: "Windows",
            arch: "X86_64",
            bundle_format: "Msi",
            file_name: "AetherOffice-0.1.0-Windows-x86_64.msi",
            sha256_checksum: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            file_size_bytes: 48234496,
            min_os_version: "Windows 10 Build 19041+",
            is_signed: true,
            cert_fingerprint: "SHA256:7B8F9A1C3E5D2A4F6B8E0D2C4A6F8B0E2D4C6A8F0B2D4C6E8A0F2B4D6C8E0A2F",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Windows-x86_64.msi",
          },
          {
            id: "win-x64-nsis",
            platform: "Windows",
            arch: "X86_64",
            bundle_format: "Nsis",
            file_name: "AetherOffice-0.1.0-Windows-x86_64-setup.exe",
            sha256_checksum: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            file_size_bytes: 46137344,
            min_os_version: "Windows 10 Build 19041+",
            is_signed: true,
            cert_fingerprint: "SHA256:7B8F9A1C3E5D2A4F6B8E0D2C4A6F8B0E2D4C6A8F0B2D4C6E8A0F2B4D6C8E0A2F",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Windows-x86_64-setup.exe",
          },
          {
            id: "win-x64-portable",
            platform: "Windows",
            arch: "X86_64",
            bundle_format: "StandaloneExe",
            file_name: "AetherOffice-0.1.0-Windows-x86_64-portable.exe",
            sha256_checksum: "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
            file_size_bytes: 52428800,
            min_os_version: "Windows 10 Build 19041+",
            is_signed: true,
            cert_fingerprint: "SHA256:7B8F9A1C3E5D2A4F6B8E0D2C4A6F8B0E2D4C6A8F0B2D4C6E8A0F2B4D6C8E0A2F",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Windows-x86_64-portable.exe",
          },
          {
            id: "linux-x64-appimage",
            platform: "Linux",
            arch: "X86_64",
            bundle_format: "AppImage",
            file_name: "AetherOffice-0.1.0-Linux-x86_64.AppImage",
            sha256_checksum: "88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589",
            file_size_bytes: 58720256,
            min_os_version: "glibc 2.31+ / Ubuntu 20.04+",
            is_signed: true,
            cert_fingerprint: "PGP:F2C4E6A8B0D2F4A6B8C0D2E4F6A8B0C2D4E6F8A0",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Linux-x86_64.AppImage",
          },
          {
            id: "linux-x64-deb",
            platform: "Linux",
            arch: "X86_64",
            bundle_format: "Deb",
            file_name: "AetherOffice_0.1.0_amd64.deb",
            sha256_checksum: "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
            file_size_bytes: 44040192,
            min_os_version: "Debian 11+ / Ubuntu 20.04+",
            is_signed: true,
            cert_fingerprint: "PGP:F2C4E6A8B0D2F4A6B8C0D2E4F6A8B0C2D4E6F8A0",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice_0.1.0_amd64.deb",
          },
          {
            id: "linux-x64-rpm",
            platform: "Linux",
            arch: "X86_64",
            bundle_format: "Rpm",
            file_name: "AetherOffice-0.1.0-1.x86_64.rpm",
            sha256_checksum: "fc94a7d34193563fe740f952f49551c6e1ca87807a0f6797a1f592fc68e090df",
            file_size_bytes: 45088768,
            min_os_version: "Fedora 36+ / RHEL 9+",
            is_signed: true,
            cert_fingerprint: "PGP:F2C4E6A8B0D2F4A6B8C0D2E4F6A8B0C2D4E6F8A0",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-1.x86_64.rpm",
          },
          {
            id: "macos-universal-dmg",
            platform: "MacOS",
            arch: "Universal",
            bundle_format: "Dmg",
            file_name: "AetherOffice-0.1.0-macOS-Universal.dmg",
            sha256_checksum: "4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a",
            file_size_bytes: 67108864,
            min_os_version: "macOS 12.0 Monterey+",
            is_signed: true,
            cert_fingerprint: "Apple Developer ID: AetherOffice Foundation (9XYZ1234AB)",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-macOS-Universal.dmg",
          },
          {
            id: "android-arm64-apk",
            platform: "Android",
            arch: "Aarch64",
            bundle_format: "Apk",
            file_name: "AetherOffice-0.1.0-Android-arm64-v8a.apk",
            sha256_checksum: "ef2d127de37b942baad06145e54b0c619a1f22327b2ebbcfbec78f5564afe39d",
            file_size_bytes: 33554432,
            min_os_version: "Android 8.0 (API Level 26+)",
            is_signed: true,
            cert_fingerprint: "Android Keystore: AETHER_RELEASE_KEY_V1",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Android-arm64-v8a.apk",
          },
          {
            id: "android-universal-aab",
            platform: "Android",
            arch: "Universal",
            bundle_format: "Aab",
            file_name: "AetherOffice-0.1.0-Android-Universal.aab",
            sha256_checksum: "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f",
            file_size_bytes: 41943040,
            min_os_version: "Android 8.0 (API Level 26+)",
            is_signed: true,
            cert_fingerprint: "Android Play Store App Signing",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-Android-Universal.aab",
          },
          {
            id: "ios-arm64-ipa",
            platform: "Ios",
            arch: "Aarch64",
            bundle_format: "Ipa",
            file_name: "AetherOffice-0.1.0-iOS-arm64.ipa",
            sha256_checksum: "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",
            file_size_bytes: 38797312,
            min_os_version: "iOS 15.0+",
            is_signed: true,
            cert_fingerprint: "Apple Distribution: AetherOffice Foundation (9XYZ1234AB)",
            download_url: "https://releases.aetheroffice.org/v0.1.0/AetherOffice-0.1.0-iOS-arm64.ipa",
          },
        ],
      };
      return manifest as unknown as T;
    }
    if (cmd === "sbom_generate_cyclonedx") {
      const doc = {
        bomFormat: "CycloneDX",
        specVersion: "1.5",
        serialNumber: "urn:uuid:7c9e6679-7425-40de-944b-e07fc1f90ae7",
        version: 1,
        metadata: {
          timestamp: "2026-11-01T00:00:00Z",
          component: {
            type: "application",
            name: "AetherOffice",
            version: "0.1.0",
          },
        },
        components: [
          {
            type: "library",
            name: "render-core",
            version: "0.1.0",
            purl: "pkg:cargo/render-core@0.1.0",
            license: "Apache-2.0 OR MIT",
            hashes: [{ alg: "SHA-256", content: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" }],
          },
        ],
      };
      return JSON.stringify(doc, null, 2) as unknown as T;
    }
    if (cmd === "sbom_generate_spdx") {
      const format = (args?.format_type as string) || "json";
      if (format.includes("tag")) {
        return "SPDXVersion: SPDX-2.3\nDataLicense: CC0-1.0\nSPDXID: SPDXRef-DOCUMENT\nDocumentName: AetherOffice-v0.1.0\n" as unknown as T;
      }
      const doc = {
        spdxVersion: "SPDX-2.3",
        dataLicense: "CC0-1.0",
        SPDXID: "SPDXRef-DOCUMENT",
        name: "AetherOffice-v0.1.0",
        packages: [
          {
            SPDXID: "SPDXRef-Package-1",
            name: "render-core",
            versionInfo: "0.1.0",
            licenseConcluded: "Apache-2.0 OR MIT",
          },
        ],
      };
      return JSON.stringify(doc, null, 2) as unknown as T;
    }
    if (cmd === "sbom_audit_licenses") {
      const report: LicenseAuditReportDto = {
        total_components: 18,
        compliant_count: 18,
        non_compliant_count: 0,
        unlicensed_count: 0,
        licenses_found: ["Apache-2.0 OR MIT", "MIT", "MIT OR Apache-2.0"],
        has_gpl_contamination: false,
        is_clean_room_compliant: true,
        violation_details: [],
      };
      return report as unknown as T;
    }
    if (cmd === "installer_verify_checksum") {
      const artId = (args?.artifact_id as string) || "";
      const hex = (args?.checksum_hex as string) || "";
      const payload = (args?.payload as number[]) || [];
      if (artId === "win-x64-msi") {
        if (hex) {
          return (hex.toLowerCase() === "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855") as unknown as T;
        }
        if (payload.length === 0) {
          return true as unknown as T;
        }
      }
      return false as unknown as T;
    }
    if (cmd === "workspace_search") {
      const q = (args?.query as string) || "";
      const matches: HybridSearchResult[] = [
        {
          chunk: {
            id: `chunk-${Date.now()}-1`,
            doc_id: "doc-mock-1",
            kind: "Word",
            raw_text: `Matched text containing query: ${q}`,
            structural_context: "Document > Section 1",
            composite_text: `Document > Section 1\nMatched text containing query: ${q}`,
            token_count_estimate: 24,
          },
          semantic_score: 0.88,
          lexical_score: 0.95,
          score: 0.90,
        },
      ];
      return matches as unknown as T;
    }
    if (cmd === "workspace_index_text") {
      return 1 as unknown as T;
    }
    if (cmd === "workspace_get_index_summary") {
      return {
        total_documents: 1,
        total_chunks: 1,
        chunks_by_kind: { Word: 1 },
        total_estimated_tokens: 24,
      } as unknown as T;
    }
    if (cmd === "federated_search") {
      const q = (args?.query as string) || "";
      const matches: FederatedSearchMatch[] = [
        {
          match_id: `match-mock-1`,
          session_id: "sess-mock-1",
          doc_type: "Word",
          document_title: "Mock Document.docx",
          location: { type: "Word", details: { line: 1, char_offset: 0 } },
          snippet: `Found matching text: ${q}`,
          matched_text: q,
          start_offset: 0,
          end_offset: q.length,
        },
      ];
      return matches as unknown as T;
    }
    if (cmd === "federated_replace_match") {
      return 1 as unknown as T;
    }
    if (cmd === "federated_replace_all") {
      const summary: FederatedReplaceSummary = {
        total_replacements: 2,
        documents_affected: 1,
        replacements_by_session: { "sess-mock-1": 2 },
      };
      return summary as unknown as T;
    }
    if (cmd === "omnibox_query") {
      const q = (args?.query as string) || "";
      const items: OmniboxItem[] = [
        {
          id: "cmd-new-tab",
          category: "Command",
          title: "New Document",
          description: "Create a new document in a fresh workspace tab",
          shortcut: "Ctrl+N",
          icon: "file-plus",
          score: 500,
          payload: "NewTab",
        },
        {
          id: "cmd-save-doc",
          category: "Command",
          title: "Save Document",
          description: "Save active document changes to disk",
          shortcut: "Ctrl+S",
          icon: "save",
          score: 480,
          payload: "SaveDocument",
        },
        {
          id: "tab-mock-1",
          category: "File",
          title: "Document.docx",
          description: "Word Document • Saved",
          icon: "file-text",
          score: 450,
          payload: "tab-mock-1",
        },
      ];
      if (!q) return items as unknown as T;
      const filtered = items.filter((i) =>
        i.title.toLowerCase().includes(q.toLowerCase())
      );
      return (filtered.length > 0 ? filtered : items) as unknown as T;
    }
    if (cmd === "export_pdf") {
      const path = (args?.output_path as string) || "Exported_Document.pdf";
      return path as unknown as T;
    }
    if (cmd === "get_document_info") {
      const sessId = (args?.session_id as string) || "sess-1";
      return {
        session_id: sessId,
        title: "Active_Document.docx",
        doc_type: "Word",
        file_path: "/workspace/Active_Document.docx",
        is_dirty: false,
        file_size_bytes: 14200,
        last_modified_str: "Just now",
      } as unknown as T;
    }
    if (cmd === "collab_join_session") {
      const sessId = (args?.session_id as string) || "collab-room-1";
      const user = (args?.username as string) || "LocalUser";
      const mode = (args?.app_mode as string) || "writer";
      const tmode = (args?.transport_mode as string) || "webrtc";
      mockCollabSession = {
        session_id: sessId,
        client_id: 101,
        username: user,
        app_mode: mode,
        status: "Connected",
        transport_mode: tmode === "websocket" ? "WebSocketRelay" : "WebRtcDataChannel",
        peers: [],
      };
      return mockCollabSession as unknown as T;
    }
    if (cmd === "collab_leave_session") {
      mockCollabSession = null;
      return true as unknown as T;
    }
    if (cmd === "collab_broadcast_delta") {
      return true as unknown as T;
    }
    if (cmd === "collab_get_peers") {
      return (mockCollabSession?.peers || []) as unknown as T;
    }
    return {} as T;
  }
}

export const IPC = {
  _resetMockState: (): void => {
    mockInstances.length = 0;
    mockCollabSession = null;
    WasmBridge.reset();
  },
  getWasmEngine: (): WasmCoreEngine => WasmBridge,
  getBootConfig: (): Promise<BootConfig> =>
    invokeTauri("get_boot_config"),
  launchStandaloneApp: (appMode: string, filePath?: string): Promise<AppInstance> =>
    invokeTauri("launch_standalone_app", { app_mode: appMode, file_path: filePath }),
  listAppInstances: (): Promise<AppInstance[]> =>
    invokeTauri("list_app_instances"),
  closeAppInstance: (instanceId: string): Promise<boolean> =>
    invokeTauri("close_app_instance", { instance_id: instanceId }),
  focusAppInstance: (instanceId: string): Promise<boolean> =>
    invokeTauri("focus_app_instance", { instance_id: instanceId }),
  createSession: (docType: DocumentType, title: string): Promise<SessionSummary> =>
    invokeTauri("create_session", { doc_type: docType, title }),

  openDocument: (filePath: string): Promise<SessionSummary> =>
    invokeTauri("open_document", { file_path: filePath }),

  openDocumentAsync: async (
    filePath: string,
    onProgress?: (progress: IngestionProgress) => void,
    sessionId?: string
  ): Promise<SessionSummary> => {
    let unlisten: (() => void) | null = null;
    const trackingId = sessionId || `track-${Date.now()}`;
    if (isTauriAvailable() && onProgress) {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<IngestionProgress>("ingestion-progress", (event) => {
          onProgress(event.payload);
        });
      } catch {
        // Event listener fallback
      }
    } else if (onProgress) {
      // Mock progress simulation for browser/unit test fallback
      onProgress({
        stage: "ReadingCentralDirectory",
        bytes_processed: 1024,
        total_bytes: 4096,
        percentage: 25.0,
      });
      onProgress({
        stage: "ExtractingParts",
        bytes_processed: 2048,
        total_bytes: 4096,
        percentage: 50.0,
      });
      onProgress({
        stage: "BuildingDom",
        bytes_processed: 3072,
        total_bytes: 4096,
        percentage: 75.0,
      });
      onProgress({
        stage: "Complete",
        bytes_processed: 4096,
        total_bytes: 4096,
        percentage: 100.0,
      });
    }

    try {
      const res = await invokeTauri<SessionSummary>("open_document_async", {
        file_path: filePath,
        session_id: trackingId,
      });
      return res;
    } finally {
      if (unlisten) {
        unlisten();
      }
    }
  },

  cancelIngestion: (sessionId: string): Promise<boolean> =>
    invokeTauri("cancel_ingestion", { session_id: sessionId }),

  saveDocument: (sessionId: string, filePath?: string): Promise<string> =>
    invokeTauri("save_document", { session_id: sessionId, file_path: filePath }),

  exportPdf: (sessionId: string, outputPath: string): Promise<string> =>
    invokeTauri("export_pdf", { session_id: sessionId, output_path: outputPath }),

  getDocumentInfo: (sessionId: string): Promise<DocumentMetadata> =>
    invokeTauri("get_document_info", { session_id: sessionId }),

  closeSession: (sessionId: string, force: boolean): Promise<string | null> =>
    invokeTauri("close_session", { session_id: sessionId, force }),

  executeUndo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_undo", { session_id: sessionId }),

  executeRedo: (sessionId: string): Promise<void> =>
    invokeTauri("execute_redo", { session_id: sessionId }),

  exportSvg: (sessionId: string): Promise<string> =>
    invokeTauri("export_svg", { session_id: sessionId }),

  dispatchUiAction: (sessionId: string, action: UiActionEvent): Promise<UiDispatchResult> =>
    invokeTauri("dispatch_ui_action", { session_id: sessionId, action }),

  collabJoinSession: (
    sessionId: string,
    username: string,
    appMode: string,
    roomKey?: string,
    transportMode?: string
  ): Promise<CollabSessionInfo> =>
    invokeTauri("collab_join_session", {
      session_id: sessionId,
      username,
      app_mode: appMode,
      room_key: roomKey,
      transport_mode: transportMode,
    }),

  collabLeaveSession: (sessionId: string, reason?: string): Promise<boolean> =>
    invokeTauri("collab_leave_session", { session_id: sessionId, reason }),

  collabBroadcastDelta: (
    sessionId: string,
    deltaType: string,
    deltaJson: string
  ): Promise<boolean> =>
    invokeTauri("collab_broadcast_delta", {
      session_id: sessionId,
      delta_type: deltaType,
      delta_json: deltaJson,
    }),

  collabGetPeers: (sessionId: string): Promise<PeerPresence[]> =>
    invokeTauri("collab_get_peers", { session_id: sessionId }),

  spawnAppWindow: (appType: string, filePath?: string): Promise<string> =>
    invokeTauri("spawn_app_window", { app_type: appType, file_path: filePath }),

  saveFileBytes: (fileName: string, bytes: number[] | Uint8Array, customPath?: string): Promise<string> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri("save_file_bytes", { file_name: fileName, bytes: byteArr, custom_path: customPath });
  },

  readFileBytes: (filePath: string): Promise<Uint8Array> => {
    return invokeTauri<number[] | Uint8Array>("read_file_bytes", { file_path: filePath }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  // ---------------------------------------------------------------------------
  // Zero-Copy Binary Buffer Bridge IPC Methods
  // ---------------------------------------------------------------------------

  readFileBinary: (filePath: string): Promise<Uint8Array> => {
    return invokeTauri<number[] | Uint8Array>("read_file_binary", { file_path: filePath }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  saveFileBinary: (
    fileName: string,
    bytes: number[] | Uint8Array,
    customPath?: string
  ): Promise<string> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri("save_file_binary", { file_name: fileName, bytes: byteArr, custom_path: customPath });
  },

  renderPdfPageBinary: (
    bytes: number[] | Uint8Array,
    pageIndex = 0
  ): Promise<Uint8Array> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri<number[] | Uint8Array>("render_pdf_page_binary", { bytes: byteArr, page_index: pageIndex }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  cryptoHashBinary: (
    bytes: number[] | Uint8Array,
    algorithm = "sha256"
  ): Promise<string> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri("crypto_hash_binary", { bytes: byteArr, algorithm });
  },

  collabSyncBinary: (
    sessionId: string,
    stateVector: number[] | Uint8Array
  ): Promise<Uint8Array> => {
    const byteArr = stateVector instanceof Uint8Array ? Array.from(stateVector) : stateVector;
    return invokeTauri<number[] | Uint8Array>("collab_sync_binary", {
      session_id: sessionId,
      state_vector: byteArr,
    }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  macroExecuteCode: (
    code: string,
    routineName = "",
    capabilities: string[] = [],
    gasLimit?: number
  ): Promise<MacroExecutionResult> => {
    return invokeTauri("macro_execute_code", {
      code,
      routine_name: routineName,
      capabilities,
      gas_limit: gasLimit,
    });
  },

  copilotQuery: (
    sessionId: string,
    prompt: string,
    docType?: DocumentType
  ): Promise<CopilotQueryResult> => {
    return invokeTauri("copilot_query", {
      session_id: sessionId,
      prompt,
      doc_type: docType,
    });
  },

  dataConnectorParseCsv: (
    bytes: number[] | Uint8Array,
    delimiter?: number
  ): Promise<DataConnectorResult> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri("data_connector_parse_csv", {
      bytes: byteArr,
      delimiter,
    });
  },

  dataConnectorImport: (
    format: string,
    bytes: number[] | Uint8Array,
    optionsJson?: string
  ): Promise<DataConnectorImportResult> => {
    const byteArr = bytes instanceof Uint8Array ? Array.from(bytes) : bytes;
    return invokeTauri<DataConnectorImportResult>("data_connector_import", {
      format,
      bytes: byteArr,
      options_json: optionsJson,
    });
  },

  dataConnectorExport: (payloadJson: string): Promise<Uint8Array> => {
    return invokeTauri<number[] | Uint8Array>("data_connector_export", {
      payload_json: payloadJson,
    }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  roundtripOpcBinary: (
    format: "docx" | "xlsx" | "pptx",
    originalBytes: number[] | Uint8Array,
    updatesJson: string
  ): Promise<Uint8Array> => {
    const byteArr = originalBytes instanceof Uint8Array ? Array.from(originalBytes) : originalBytes;
    return invokeTauri<number[] | Uint8Array>("roundtrip_opc_binary", {
      format,
      original_bytes: byteArr,
      updates_json: updatesJson,
    }).then((res) => {
      if (res instanceof Uint8Array) return res;
      return new Uint8Array(res || []);
    });
  },

  workspaceSearch: (query: string, topK?: number): Promise<HybridSearchResult[]> =>
    invokeTauri("workspace_search", { query, top_k: topK }),

  workspaceIndexText: (docId: string, title: string, text: string): Promise<number> =>
    invokeTauri("workspace_index_text", { doc_id: docId, title, text }),

  workspaceGetIndexSummary: (): Promise<IndexSummary> =>
    invokeTauri("workspace_get_index_summary"),

  federatedSearch: (
    query: string,
    options?: Partial<SearchOptions>
  ): Promise<FederatedSearchMatch[]> =>
    invokeTauri("federated_search", {
      query,
      case_sensitive: options?.case_sensitive ?? false,
      whole_word: options?.whole_word ?? false,
      use_regex: options?.use_regex ?? false,
    }),

  federatedReplaceMatch: (
    sessionId: string,
    query: string,
    replacement: string,
    options?: Partial<SearchOptions>
  ): Promise<number> =>
    invokeTauri("federated_replace_match", {
      session_id: sessionId,
      query,
      replacement,
      case_sensitive: options?.case_sensitive ?? false,
      whole_word: options?.whole_word ?? false,
      use_regex: options?.use_regex ?? false,
    }),

  federatedReplaceAll: (
    query: string,
    replacement: string,
    options?: Partial<SearchOptions>,
    targetSessionIds?: string[]
  ): Promise<FederatedReplaceSummary> =>
    invokeTauri("federated_replace_all", {
      query,
      replacement,
      case_sensitive: options?.case_sensitive ?? false,
      whole_word: options?.whole_word ?? false,
      use_regex: options?.use_regex ?? false,
      target_session_ids: targetSessionIds,
    }),

  omniboxQuery: (query: string, maxResults?: number): Promise<OmniboxItem[]> =>
    invokeTauri("omnibox_query", { query, max_results: maxResults }),

  pptGetSlideMasterTree: (packageBytes: Uint8Array): Promise<MasterLayoutTreeDto> =>
    invokeTauri("ppt_get_slide_master_tree", { package_bytes: Array.from(packageBytes) }),

  pptApplySlideMasterLayout: (
    packageBytes: Uint8Array,
    slideIdx: number,
    layoutPartName: string
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_apply_slide_master_layout", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      layout_part_name: layoutPartName,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptCreateSlideLayout: (
    packageBytes: Uint8Array,
    masterPartName: string,
    layoutType: string,
    name: string
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_create_slide_layout", {
      package_bytes: Array.from(packageBytes),
      master_part_name: masterPartName,
      layout_type: layoutType,
      name,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptUpdatePlaceholderTransform: (
    packageBytes: Uint8Array,
    targetPart: string,
    shapeId: number,
    x: number,
    y: number,
    cx: number,
    cy: number
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_update_placeholder_transform", {
      package_bytes: Array.from(packageBytes),
      target_part: targetPart,
      shape_id: shapeId,
      x,
      y,
      cx,
      cy,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptBuildSmartArt: (
    packageBytes: Uint8Array,
    slideIdx: number,
    request: SmartArtBuildRequest
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_build_smartart", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      request_json: JSON.stringify(request),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptGroupShapes: (
    packageBytes: Uint8Array,
    slideIdx: number,
    shapeIds: number[]
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_group_shapes", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      shape_ids: shapeIds,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptUngroupShapes: (
    packageBytes: Uint8Array,
    slideIdx: number,
    groupId: number
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_ungroup_shapes", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      group_id: groupId,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptReorderZOrder: (
    packageBytes: Uint8Array,
    slideIdx: number,
    shapeId: number,
    action: "front" | "back" | "forward" | "backward"
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_reorder_z_order", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      shape_id: shapeId,
      action,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptGetSlideLayerTree: (
    packageBytes: Uint8Array,
    slideIdx: number
  ): Promise<SlideLayerTreeDto> =>
    invokeTauri("ppt_get_slide_layer_tree", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
    }),

  renderSmartGuidesSnap: (
    moving: RectBounds,
    neighbors: RectBounds[],
    canvas?: RectBounds | null,
    threshold?: number
  ): Promise<SnapResult> =>
    invokeTauri("render_smart_guides_snap", {
      moving_bounds_json: JSON.stringify(moving),
      neighbors_json: JSON.stringify(neighbors),
      canvas_bounds_json: canvas ? JSON.stringify(canvas) : null,
      threshold,
    }),

  pptEvaluateMorphTransition: (
    packageBytes: Uint8Array,
    sourceSlideIdx: number,
    targetSlideIdx: number,
    progress: number,
    morphOption: string = "byObject"
  ): Promise<MorphedSlideStateDto> =>
    invokeTauri("ppt_evaluate_morph_transition", {
      package_bytes: Array.from(packageBytes),
      source_slide_idx: sourceSlideIdx,
      target_slide_idx: targetSlideIdx,
      progress,
      morph_option: morphOption,
    }),

  pptGetSlideAnimationTracks: (
    packageBytes: Uint8Array,
    slideIdx: number
  ): Promise<SlideAnimationTracksDto> =>
    invokeTauri("ppt_get_slide_animation_tracks", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
    }),

  pptScrubSlideAnimation: (
    packageBytes: Uint8Array,
    slideIdx: number,
    timeMs: number,
    clickIndex?: number | null
  ): Promise<AnimatedSceneStateDto> =>
    invokeTauri("ppt_scrub_slide_animation", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      time_ms: timeMs,
      click_index: clickIndex ?? null,
    }),

  pptAdvanceAnimationClick: (
    packageBytes: Uint8Array,
    slideIdx: number,
    currentClick: number,
    forward: boolean = true
  ): Promise<AnimationClickResultDto> =>
    invokeTauri("ppt_advance_animation_click", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
      current_click: currentClick,
      forward,
    }),

  pptExportSlideSvg: (
    packageBytes: Uint8Array,
    slideIdx: number
  ): Promise<string> =>
    invokeTauri("ppt_export_slide_svg", {
      package_bytes: Array.from(packageBytes),
      slide_idx: slideIdx,
    }),

  pptExportDeckPdf: (
    packageBytes: Uint8Array
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_export_deck_pdf", {
      package_bytes: Array.from(packageBytes),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptConvertOdpToPptx: (
    odpBytes: Uint8Array
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_convert_odp_to_pptx", {
      odp_bytes: Array.from(odpBytes),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pptConvertPptxToOdp: (
    pptxBytes: Uint8Array
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("ppt_convert_pptx_to_odp", {
      pptx_bytes: Array.from(pptxBytes),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfRenderPageCanvas: (
    bytes: Uint8Array,
    pageIndex: number,
    scale?: number
  ): Promise<RenderedPageCanvasDto> =>
    invokeTauri("pdf_render_page_canvas", {
      bytes: Array.from(bytes),
      page_index: pageIndex,
      scale: scale ?? null,
    }),

  pdfRenderPageSvg: (
    bytes: Uint8Array,
    pageIndex: number
  ): Promise<string> =>
    invokeTauri("pdf_render_page_svg", {
      bytes: Array.from(bytes),
      page_index: pageIndex,
    }),

  pdfExtractPageImages: (
    bytes: Uint8Array,
    pageIndex: number
  ): Promise<RenderedImageElementDto[]> =>
    invokeTauri("pdf_extract_page_images", {
      bytes: Array.from(bytes),
      page_index: pageIndex,
    }),

  pdfSplitDocument: (
    bytes: Uint8Array,
    splitIndices: number[]
  ): Promise<Uint8Array[]> =>
    invokeTauri<(number[] | Uint8Array)[]>("pdf_split_document", {
      bytes: Array.from(bytes),
      split_indices: splitIndices,
    }).then((res) =>
      res.map((b) => (b instanceof Uint8Array ? b : new Uint8Array(b || [])))
    ),

  pdfMergeDocuments: (
    documentsBytes: Uint8Array[]
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_merge_documents", {
      documents_bytes: documentsBytes.map((b) => Array.from(b)),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfExtractPages: (
    bytes: Uint8Array,
    pageIndices: number[]
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_extract_pages", {
      bytes: Array.from(bytes),
      page_indices: pageIndices,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfApplyBatesNumbering: (
    bytes: Uint8Array,
    config: BatesNumberingConfig
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_apply_bates_numbering", {
      bytes: Array.from(bytes),
      config_json: JSON.stringify(config),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfInjectOcrLayer: (
    bytes: Uint8Array,
    pageIndex: number,
    ocrData: OcrPageData
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_inject_ocr_layer", {
      bytes: Array.from(bytes),
      page_index: pageIndex,
      ocr_data_json: JSON.stringify(ocrData),
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfCompareDocuments: (
    docABytes: Uint8Array,
    docBBytes: Uint8Array,
    options?: PdfDiffOptions
  ): Promise<PdfDiffResultDto> =>
    invokeTauri<PdfDiffResultDto>("pdf_compare_documents", {
      doc_a_bytes: Array.from(docABytes),
      doc_b_bytes: Array.from(docBBytes),
      options_json: options ? JSON.stringify(options) : null,
    }),

  pdfGenerateDiffOverlaySvg: (
    docBytes: Uint8Array,
    pageIndex: number,
    diffItems: DiffItemDto[],
    isRevised: boolean
  ): Promise<string> =>
    invokeTauri<string>("pdf_generate_diff_overlay_svg", {
      doc_bytes: Array.from(docBytes),
      page_index: pageIndex,
      diff_items_json: JSON.stringify(diffItems),
      is_revised: isRevised,
    }),

  pdfSignPades: (
    docBytes: Uint8Array,
    certPem: string,
    privateKeyPem: string,
    options?: PdfSigningOptions
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_sign_pades", {
      doc_bytes: Array.from(docBytes),
      cert_pem: certPem,
      private_key_pem: privateKeyPem,
      options_json: options ? JSON.stringify(options) : null,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  pdfValidateSignatures: (
    docBytes: Uint8Array
  ): Promise<PdfSignatureValidationDto[]> =>
    invokeTauri<PdfSignatureValidationDto[]>("pdf_validate_signatures", {
      doc_bytes: Array.from(docBytes),
    }),

  pdfApplyRedactions: (
    docBytes: Uint8Array,
    redactions: PdfRedactionTarget[]
  ): Promise<[Uint8Array, PdfRedactionReportDto]> =>
    invokeTauri<[number[] | Uint8Array, PdfRedactionReportDto]>("pdf_apply_redactions", {
      doc_bytes: Array.from(docBytes),
      redactions_json: JSON.stringify(redactions),
    }).then(([bytes, report]) => [
      bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes || []),
      report,
    ]),

  pdfExportPdfa: (
    docBytes: Uint8Array,
    options?: PdfaExportOptions
  ): Promise<Uint8Array> =>
    invokeTauri<number[] | Uint8Array>("pdf_export_pdfa", {
      doc_bytes: Array.from(docBytes),
      options_json: options ? JSON.stringify(options) : null,
    }).then((res) => (res instanceof Uint8Array ? res : new Uint8Array(res || []))),

  depGraphRegisterDependency: (edge: DependencyEdge): Promise<string> =>
    invokeTauri<string>("dep_graph_register_dependency", { edge }),

  depGraphRemoveDependency: (edgeId: string): Promise<boolean> =>
    invokeTauri<boolean>("dep_graph_remove_dependency", { edge_id: edgeId }),

  depGraphGetDependents: (docId: string): Promise<DependencyEdge[]> =>
    invokeTauri<DependencyEdge[]>("dep_graph_get_dependents", { doc_id: docId }),

  depGraphGetDependencies: (docId: string): Promise<DependencyEdge[]> =>
    invokeTauri<DependencyEdge[]>("dep_graph_get_dependencies", { doc_id: docId }),

  depGraphPropagateInvalidation: (
    sourceDocId: string,
    affectedAnchor?: EmbedAnchor
  ): Promise<string[]> =>
    invokeTauri<string[]>("dep_graph_propagate_invalidation", {
      source_doc_id: sourceDocId,
      affected_anchor: affectedAnchor ?? null,
    }),

  depGraphUpdateSnapshot: (snapshot: LiveEmbedSnapshot): Promise<void> =>
    invokeTauri<void>("dep_graph_update_snapshot", { snapshot }),

  depGraphGetSnapshot: (embedId: string): Promise<LiveEmbedSnapshot | null> =>
    invokeTauri<LiveEmbedSnapshot | null>("dep_graph_get_snapshot", { embed_id: embedId }),

  depGraphSummary: (): Promise<CrossDocSummaryDto> =>
    invokeTauri<CrossDocSummaryDto>("dep_graph_summary"),

  wikilinkParseLinks: (text: string): Promise<WikiLink[]> =>
    invokeTauri<WikiLink[]>("wikilink_parse_links", { text }),

  wikilinkIndexDocument: (docId: string, title: string, content: string): Promise<void> =>
    invokeTauri<void>("wikilink_index_document", { doc_id: docId, title, content }),

  wikilinkRemoveDocument: (docId: string): Promise<void> =>
    invokeTauri<void>("wikilink_remove_document", { doc_id: docId }),

  wikilinkGetBacklinks: (targetTitleOrId: string): Promise<BacklinkReference[]> =>
    invokeTauri<BacklinkReference[]>("wikilink_get_backlinks", { target_title_or_id: targetTitleOrId }),

  wikilinkGetForwardLinks: (sourceDocId: string): Promise<WikiLink[]> =>
    invokeTauri<WikiLink[]>("wikilink_get_forward_links", { source_doc_id: sourceDocId }),

  wikilinkSuggestTargets: (query: string, limit?: number): Promise<WikiLinkSuggestion[]> =>
    invokeTauri<WikiLinkSuggestion[]>("wikilink_suggest_targets", { query, limit: limit ?? null }),

  wikilinkFindUnlinkedMentions: (targetTitle: string): Promise<UnlinkedMention[]> =>
    invokeTauri<UnlinkedMention[]>("wikilink_find_unlinked_mentions", { target_title: targetTitle }),

  wikilinkStats: (): Promise<WikiLinkIndexStats> =>
    invokeTauri<WikiLinkIndexStats>("wikilink_stats"),

  conversionConvertSingle: (task: ConversionTask): Promise<ConversionResult> =>
    invokeTauri<ConversionResult>("conversion_convert_single", { task }),

  conversionBatchConvert: (tasks: ConversionTask[]): Promise<BatchConversionReport> =>
    invokeTauri<BatchConversionReport>("conversion_batch_convert", { tasks }),

  conversionSupportedFormats: (from: ConversionFormat): Promise<ConversionFormat[]> =>
    invokeTauri<ConversionFormat[]>("conversion_supported_formats", { from }),

  conversionEstimateJob: (taskCount: number, totalBytes: number): Promise<ConversionEstimateDto> =>
    invokeTauri<ConversionEstimateDto>("conversion_estimate_job", {
      task_count: taskCount,
      total_bytes: totalBytes,
    }),

  oplogAppendEntry: (
    docId: string,
    authorId: string,
    payload: OpPayloadDto,
    timestampMs: number,
    clientId: number
  ): Promise<OpLogEntryDto> =>
    invokeTauri<OpLogEntryDto>("oplog_append_entry", {
      doc_id: docId,
      author_id: authorId,
      payload,
      timestamp_ms: timestampMs,
      client_id: clientId,
    }),

  oplogGetHistory: (
    docId: string,
    limit?: number,
    offset?: number
  ): Promise<OpLogEntryDto[]> =>
    invokeTauri<OpLogEntryDto[]>("oplog_get_history", {
      doc_id: docId,
      limit: limit ?? null,
      offset: offset ?? null,
    }),

  oplogTimeTravel: (
    docId: string,
    targetTimestampMs: number
  ): Promise<MaterializedDocumentStateDto> =>
    invokeTauri<MaterializedDocumentStateDto>("oplog_time_travel", {
      doc_id: docId,
      target_timestamp_ms: targetTimestampMs,
    }),

  oplogCreateCheckpoint: (
    docId: string,
    authorId: string,
    label: string,
    clientId: number,
    timestampMs: number
  ): Promise<CheckpointMetaDto> =>
    invokeTauri<CheckpointMetaDto>("oplog_create_checkpoint", {
      doc_id: docId,
      author_id: authorId,
      label,
      client_id: clientId,
      timestamp_ms: timestampMs,
    }),

  oplogGetCheckpoints: (docId: string): Promise<CheckpointMetaDto[]> =>
    invokeTauri<CheckpointMetaDto[]>("oplog_get_checkpoints", { doc_id: docId }),

  oplogComputeSyncDelta: (
    docId: string,
    remoteVector: StateVectorDto
  ): Promise<OpLogEntryDto[]> =>
    invokeTauri<OpLogEntryDto[]>("oplog_compute_sync_delta", {
      doc_id: docId,
      remote_vector: remoteVector,
    }),

  oplogApplyRemoteDelta: (
    docId: string,
    remoteEntries: OpLogEntryDto[]
  ): Promise<SyncReportDto> =>
    invokeTauri<SyncReportDto>("oplog_apply_remote_delta", {
      doc_id: docId,
      remote_entries: remoteEntries,
    }),

  oplogGetStateVector: (docId: string): Promise<StateVectorDto> =>
    invokeTauri<StateVectorDto>("oplog_get_state_vector", { doc_id: docId }),

  oplogForkVersion: (
    docId: string,
    targetOpId: OpIdDto,
    newDocId: string,
    newAuthorId: string,
    clientId: number,
    timestampMs: number
  ): Promise<MaterializedDocumentStateDto> =>
    invokeTauri<MaterializedDocumentStateDto>("oplog_fork_version", {
      doc_id: docId,
      target_op_id: targetOpId,
      new_doc_id: newDocId,
      new_author_id: newAuthorId,
      client_id: clientId,
      timestamp_ms: timestampMs,
    }),

  pluginLoad: (manifestJson: string): Promise<PluginStatusDto> =>
    invokeTauri<PluginStatusDto>("plugin_load", { manifest_json: manifestJson }),

  pluginUnload: (pluginId: string): Promise<boolean> =>
    invokeTauri<boolean>("plugin_unload", { plugin_id: pluginId }),

  pluginList: (): Promise<PluginStatusDto[]> =>
    invokeTauri<PluginStatusDto[]>("plugin_list"),

  pluginInit: (pluginId: string): Promise<PluginStatusDto> =>
    invokeTauri<PluginStatusDto>("plugin_init", { plugin_id: pluginId }),

  pluginSuspend: (pluginId: string): Promise<PluginStatusDto> =>
    invokeTauri<PluginStatusDto>("plugin_suspend", { plugin_id: pluginId }),

  pluginResume: (pluginId: string): Promise<PluginStatusDto> =>
    invokeTauri<PluginStatusDto>("plugin_resume", { plugin_id: pluginId }),

  pluginRefillFuel: (pluginId: string, additionalFuel: number): Promise<number> =>
    invokeTauri<number>("plugin_refill_fuel", {
      plugin_id: pluginId,
      additional_fuel: additionalFuel,
    }),

  pluginExecuteAction: (
    pluginId: string,
    action: PluginActionDto
  ): Promise<ActionResultDto> =>
    invokeTauri<ActionResultDto>("plugin_execute_action", {
      plugin_id: pluginId,
      action,
    }),

  pluginCallUdf: (
    funcName: string,
    args: string[]
  ): Promise<PluginUdfCallResultDto> =>
    invokeTauri<PluginUdfCallResultDto>("plugin_call_udf", {
      func_name: funcName,
      args,
    }),

  pluginListUdfs: (): Promise<PluginUdfMetaDto[]> =>
    invokeTauri<PluginUdfMetaDto[]>("plugin_list_udfs"),

  pluginRegisterMockUdf: (meta: PluginUdfMetaDto): Promise<boolean> =>
    invokeTauri<boolean>("plugin_register_mock_udf", { meta }),

  copilotChat: (request: CopilotChatRequestDto): Promise<CopilotChatResponseDto> =>
    invokeTauri<CopilotChatResponseDto>("copilot_chat", { request }),

  copilotConfigureProvider: (config: ProviderConfigDto): Promise<ProviderStatusDto> =>
    invokeTauri<ProviderStatusDto>("copilot_configure_provider", { config }),

  copilotGetProviderConfig: (): Promise<ProviderStatusDto> =>
    invokeTauri<ProviderStatusDto>("copilot_get_provider_config"),

  copilotIndexDocument: (docId: string, title: string, text: string): Promise<number> =>
    invokeTauri<number>("copilot_index_document", {
      doc_id: docId,
      title,
      text,
    }),

  copilotRemoveDocument: (docId: string): Promise<number> =>
    invokeTauri<number>("copilot_remove_document", { doc_id: docId }),

  themeGetTokens: (theme?: ThemeMode, ribbonMode?: RibbonLayoutMode): Promise<ThemeTokensDto> =>
    invokeTauri<ThemeTokensDto>("theme_get_tokens", { theme, ribbon_mode: ribbonMode }),

  themeSetMode: (mode: ThemeMode): Promise<ThemeTokensDto> =>
    invokeTauri<ThemeTokensDto>("theme_set_mode", { mode }),

  ribbonSetLayoutMode: (mode: RibbonLayoutMode): Promise<RibbonLayoutConfigDto> =>
    invokeTauri<RibbonLayoutConfigDto>("ribbon_set_layout_mode", { mode }),

  ribbonGetLayoutConfig: (): Promise<RibbonLayoutConfigDto> =>
    invokeTauri<RibbonLayoutConfigDto>("ribbon_get_layout_config"),

  accessibilityGetAriaTree: (): Promise<AriaTreeDto> =>
    invokeTauri<AriaTreeDto>("accessibility_get_aria_tree"),

  accessibilitySetAriaTree: (tree: AriaTreeDto): Promise<number> =>
    invokeTauri<number>("accessibility_set_aria_tree", { tree }),

  imeStartComposition: (
    caretX: number,
    caretY: number,
    caretH: number
  ): Promise<AriaBoundingBoxDto> =>
    invokeTauri<AriaBoundingBoxDto>("ime_start_composition", {
      caret_x: caretX,
      caret_y: caretY,
      caret_h: caretH,
    }),

  imeUpdateComposition: (
    text: string,
    cursor: number,
    selStart: number,
    selEnd: number,
    caretX?: number,
    caretY?: number,
    caretH?: number
  ): Promise<AriaBoundingBoxDto> =>
    invokeTauri<AriaBoundingBoxDto>("ime_update_composition", {
      text,
      cursor,
      sel_start: selStart,
      sel_end: selEnd,
      caret_x: caretX,
      caret_y: caretY,
      caret_h: caretH,
    }),

  imeFinishComposition: (): Promise<string> =>
    invokeTauri<string>("ime_finish_composition"),

  imeCancelComposition: (): Promise<void> =>
    invokeTauri<void>("ime_cancel_composition"),

  bidiAnalyzeText: (text: string): Promise<BidiAnalysisResultDto> =>
    invokeTauri<BidiAnalysisResultDto>("bidi_analyze_text", { text }),

  keyboardNavCyclePane: (reverse?: boolean): Promise<NavFocusPane> =>
    invokeTauri<NavFocusPane>("keyboard_nav_cycle_pane", { reverse }),

  gestureProcessTouchStart: (touch: TouchPointDto): Promise<TouchGestureDto> =>
    invokeTauri<TouchGestureDto>("gesture_process_touch_start", { touch }),

  gestureProcessTouchMove: (touch: TouchPointDto): Promise<TouchGestureDto> =>
    invokeTauri<TouchGestureDto>("gesture_process_touch_move", { touch }),

  gestureProcessTouchEnd: (
    touchId: number,
    timestampMs: number
  ): Promise<TouchGestureDto> =>
    invokeTauri<TouchGestureDto>("gesture_process_touch_end", {
      touch_id: touchId,
      timestamp_ms: timestampMs,
    }),

  gestureProcessTouchCancel: (touchId: number): Promise<void> =>
    invokeTauri<void>("gesture_process_touch_cancel", { touch_id: touchId }),

  palmRejectionFilterContact: (
    touch: TouchPointDto,
    activeStylus?: TouchPointDto | null,
    screenW?: number,
    screenH?: number
  ): Promise<boolean> =>
    invokeTauri<boolean>("palm_rejection_filter_contact", {
      touch,
      active_stylus: activeStylus ?? null,
      screen_w: screenW ?? 1920,
      screen_h: screenH ?? 1080,
    }),

  adaptiveSplitSetLayout: (
    mode: SplitGridMode | string,
    ratioX?: number,
    ratioY?: number
  ): Promise<AdaptiveSplitGridDto> =>
    invokeTauri<AdaptiveSplitGridDto>("adaptive_split_set_layout", {
      mode_str: mode,
      ratio_x: ratioX,
      ratio_y: ratioY,
    }),

  adaptiveSplitGetRects: (
    totalWidth: number,
    totalHeight: number,
    insetTop?: number,
    insetBottom?: number
  ): Promise<Rect2DDto[]> =>
    invokeTauri<Rect2DDto[]>("adaptive_split_get_rects", {
      total_width: totalWidth,
      total_height: totalHeight,
      inset_top: insetTop,
      inset_bottom: insetBottom,
    }),

  dlpEvaluateText: (text: string): Promise<DlpEvaluationResultDto> =>
    invokeTauri<DlpEvaluationResultDto>("dlp_evaluate_text", { text }),

  dlpEvaluateAndRedact: (
    text: string
  ): Promise<[DlpEvaluationResultDto, string]> =>
    invokeTauri<[DlpEvaluationResultDto, string]>("dlp_evaluate_and_redact", {
      text,
    }),

  dlpSetPolicy: (policy: DlpPolicyDto): Promise<void> =>
    invokeTauri<void>("dlp_set_policy", { policy }),

  dlpGetPolicy: (): Promise<DlpPolicyDto> =>
    invokeTauri<DlpPolicyDto>("dlp_get_policy"),

  sanitizationGenerateCertificate: (
    docId: string,
    originalBytes: Uint8Array | number[],
    sanitizedBytes: Uint8Array | number[],
    policyId: string,
    timestampMs: number,
    redactions: Array<[string, number]>,
    secretKey: Uint8Array | number[]
  ): Promise<SanitizationCertificateDto> =>
    invokeTauri<SanitizationCertificateDto>(
      "sanitization_generate_certificate",
      {
        doc_id: docId,
        original_bytes: Array.from(originalBytes),
        sanitized_bytes: Array.from(sanitizedBytes),
        policy_id: policyId,
        timestamp_ms: timestampMs,
        redactions,
        secret_key: Array.from(secretKey),
      }
    ),

  sanitizationVerifyCertificate: (
    cert: SanitizationCertificateDto,
    sanitizedBytes?: Uint8Array | number[] | null,
    secretKey?: Uint8Array | number[]
  ): Promise<boolean> =>
    invokeTauri<boolean>("sanitization_verify_certificate", {
      cert,
      sanitized_bytes: sanitizedBytes ? Array.from(sanitizedBytes) : null,
      secret_key: secretKey ? Array.from(secretKey) : [],
    }),

  auditLogAppend: (
    timestampMs: number,
    eventType: AuditEventTypeDto | string,
    userId: string,
    docId: string,
    details: string
  ): Promise<AuditLogEntryDto> =>
    invokeTauri<AuditLogEntryDto>("audit_log_append", {
      timestamp_ms: timestampMs,
      event_type: eventType,
      user_id: userId,
      doc_id: docId,
      details,
    }),

  auditLogGetEntries: (docId?: string): Promise<AuditLogEntryDto[]> =>
    invokeTauri<AuditLogEntryDto[]>("audit_log_get_entries", {
      doc_id: docId ?? null,
    }),

  auditLogVerifyIntegrity: (): Promise<number> =>
    invokeTauri<number>("audit_log_verify_integrity"),

  entitlementGetTier: (): Promise<string> =>
    invokeTauri<string>("entitlement_get_tier"),

  entitlementCheckFeature: (
    feature: FeatureEntitlementDto | string,
    nowMs?: number
  ): Promise<EntitlementStatusDto> =>
    invokeTauri<EntitlementStatusDto>("entitlement_check_feature", {
      feature,
      now_ms: nowMs ?? null,
    }),

  entitlementApplyLicense: (
    token: LicenseTokenDto,
    verificationKey: Uint8Array | number[],
    nowMs?: number
  ): Promise<EntitlementStatusDto> =>
    invokeTauri<EntitlementStatusDto>("entitlement_apply_license", {
      token,
      verification_key: Array.from(verificationKey),
      now_ms: nowMs ?? null,
    }),

  entitlementGetLicenseInfo: (): Promise<LicenseTokenDto | null> =>
    invokeTauri<LicenseTokenDto | null>("entitlement_get_license_info"),

  entitlementSetTier: (tier: EditionTierDto | string): Promise<string> =>
    invokeTauri<string>("entitlement_set_tier", { tier }),

  entitlementRemoveLicense: (): Promise<boolean> =>
    invokeTauri<boolean>("entitlement_remove_license"),

  slaRunBenchmarks: (): Promise<SlaBenchmarkReportDto> =>
    invokeTauri<SlaBenchmarkReportDto>("sla_run_benchmarks"),

  slaVerifyBudgets: (
    report: SlaBenchmarkReportDto,
    thresholds?: SlaBudgetThresholdsDto
  ): Promise<boolean> =>
    invokeTauri<boolean>("sla_verify_budgets", {
      report,
      thresholds: thresholds ?? null,
    }),

  fuzzTestDocumentBuffer: (
    payload: Uint8Array | number[],
    mutationType: FuzzMutationTypeDto | string
  ): Promise<number[]> =>
    invokeTauri<number[]>("fuzz_test_document_buffer", {
      payload: Array.from(payload),
      mutation_type: mutationType,
    }),

  differentialAuditEvaluate: (
    baseDisplay: unknown,
    candidateDisplay: unknown
  ): Promise<DifferentialAuditResultDto> =>
    invokeTauri<DifferentialAuditResultDto>("differential_audit_evaluate", {
      base_display: baseDisplay,
      candidate_display: candidateDisplay,
    }),

  releaseMatrixGetManifest: (): Promise<InstallerMatrixManifestDto> =>
    invokeTauri<InstallerMatrixManifestDto>("release_matrix_get_manifest"),

  sbomGenerateCycloneDx: (): Promise<string> =>
    invokeTauri<string>("sbom_generate_cyclonedx"),

  sbomGenerateSpdx: (formatType?: "json" | "tag-value"): Promise<string> =>
    invokeTauri<string>("sbom_generate_spdx", {
      format_type: formatType ?? null,
    }),

  sbomAuditLicenses: (): Promise<LicenseAuditReportDto> =>
    invokeTauri<LicenseAuditReportDto>("sbom_audit_licenses"),

  installerVerifyChecksum: (
    artifactId: string,
    payload?: Uint8Array | number[],
    checksumHex?: string
  ): Promise<boolean> =>
    invokeTauri<boolean>("installer_verify_checksum", {
      artifact_id: artifactId,
      payload: payload ? Array.from(payload) : null,
      checksum_hex: checksumHex ?? null,
    }),
};
