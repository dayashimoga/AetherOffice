use copilot_core::{
    ActiveSelectionContext, AssemblerConfig, CalcPromptTemplate, CitationGroundingEngine,
    ContextAssembler, DocumentChunk, DocumentKind, DocumentLocation, SearchResult,
    SlidesPromptTemplate, WordPromptTemplate,
};
use std::collections::HashMap;

#[test]
fn test_citation_grounding_verification_valid() {
    let engine = CitationGroundingEngine::new();

    let chunks = vec![DocumentChunk {
        id: "chunk_001".to_string(),
        doc_id: "financial_report".to_string(),
        kind: DocumentKind::Word,
        location: DocumentLocation::Word {
            section: 1,
            paragraph_index: 3,
            heading_level: Some(2),
            heading_title: Some("Revenue Growth".to_string()),
        },
        structural_context: "[Document: Annual Report > Section 1 > H2: Revenue Growth]".to_string(),
        raw_text: "In the third quarter, enterprise subscription revenue increased by 24% year over year to $1,500,000."
            .to_string(),
        composite_text: "Third quarter enterprise subscription revenue increased by 24% to $1,500,000."
            .to_string(),
        token_count_estimate: 25,
        metadata: HashMap::new(),
    }];

    let generated_response =
        "Enterprise subscription revenue demonstrated impressive acceleration, growing 24% year over year to reach $1,500,000 [1].";

    let result = engine
        .verify_grounding(generated_response, &chunks)
        .expect("grounding verification failed");

    assert!(result.is_grounded);
    assert_eq!(result.verified_citations.len(), 1);
    assert_eq!(result.verified_citations[0].marker, "[1]");
    assert_eq!(result.verified_citations[0].chunk_id, "chunk_001");
    assert!(result.verified_citations[0].confidence_score >= 0.25);
    assert!(result.detected_hallucinations.is_empty());
}

#[test]
fn test_hallucination_guard_detects_unsupported_numbers() {
    let engine = CitationGroundingEngine::new();

    let chunks = vec![DocumentChunk {
        id: "chunk_002".to_string(),
        doc_id: "sales_data".to_string(),
        kind: DocumentKind::Calc,
        location: DocumentLocation::Calc {
            sheet_name: "Q3".to_string(),
            row_start: 2,
            row_end: 5,
            col_start: 1,
            col_end: 3,
            a1_reference: "A2:C5".to_string(),
        },
        structural_context: "[Spreadsheet: Sales > Sheet: Q3]".to_string(),
        raw_text: "North America sales totaled $350,000 with 12% margin.".to_string(),
        composite_text: "North America sales totaled $350,000 with 12% margin.".to_string(),
        token_count_estimate: 15,
        metadata: HashMap::new(),
    }];

    // Response fabricates a figure of $8,900,000 and 75%
    let hallucinated_response =
        "North America region achieved phenomenal sales of $8,900,000 with a 75% profit margin [1].";

    let result = engine
        .verify_grounding(hallucinated_response, &chunks)
        .expect("grounding verification failed");

    assert!(!result.is_grounded);
    assert!(!result.detected_hallucinations.is_empty());
    assert!(result
        .detected_hallucinations
        .iter()
        .any(|h| h.contains("$8,900,000")));
}

#[test]
fn test_context_assembler_budget_and_truncation() {
    let config = AssemblerConfig {
        max_token_budget: 80,
        max_chunks: 2,
        prioritize_active_selection: true,
    };
    let assembler = ContextAssembler::new(config);

    let chunks = vec![
        SearchResult {
            chunk: DocumentChunk {
                id: "c1".to_string(),
                doc_id: "doc1".to_string(),
                kind: DocumentKind::Word,
                location: DocumentLocation::Generic {
                    offset: 0,
                    length: 100,
                },
                structural_context: "Intro".to_string(),
                raw_text: "This is a detailed overview of enterprise architecture and service-oriented design patterns."
                    .to_string(),
                composite_text: "Intro overview".to_string(),
                token_count_estimate: 20,
                metadata: HashMap::new(),
            },
            similarity: 0.95,
        },
        SearchResult {
            chunk: DocumentChunk {
                id: "c2".to_string(),
                doc_id: "doc1".to_string(),
                kind: DocumentKind::Word,
                location: DocumentLocation::Generic {
                    offset: 100,
                    length: 100,
                },
                structural_context: "Body".to_string(),
                raw_text: "Microservices communicate using low-latency asynchronous message protocols across instances."
                    .to_string(),
                composite_text: "Body comms".to_string(),
                token_count_estimate: 20,
                metadata: HashMap::new(),
            },
            similarity: 0.85,
        },
        SearchResult {
            chunk: DocumentChunk {
                id: "c3".to_string(),
                doc_id: "doc1".to_string(),
                kind: DocumentKind::Word,
                location: DocumentLocation::Generic {
                    offset: 200,
                    length: 100,
                },
                structural_context: "Details".to_string(),
                raw_text: "Database replication utilizes Raft consensus with log compaction for deterministic state."
                    .to_string(),
                composite_text: "Details Raft".to_string(),
                token_count_estimate: 20,
                metadata: HashMap::new(),
            },
            similarity: 0.70,
        },
    ];

    let meta = HashMap::new();
    let assembled = assembler
        .assemble("Explain architecture", &chunks, None, &meta)
        .expect("assemble failed");

    // Max chunks was 2, so third chunk is truncated
    assert!(assembled.selected_chunks.len() <= 2);
    assert!(assembled.is_truncated);
    assert!(assembled.context_block.contains("[1]"));
    assert!(assembled.citation_mapping.iter().any(|(m, _)| m == "[1]"));
}

#[test]
fn test_context_assembler_prioritizes_active_selection() {
    let assembler = ContextAssembler::default();

    let selection = ActiveSelectionContext {
        kind: DocumentKind::Word,
        selected_text: Some("Selected strategic paragraph on zero-trust security.".to_string()),
        selected_range: Some("Paragraph 5".to_string()),
        location: None,
    };

    let meta = HashMap::new();
    let assembled = assembler
        .assemble("Security analysis", &[], Some(&selection), &meta)
        .expect("assemble failed");

    assert!(assembled
        .context_block
        .contains("[Active Selection: Word (Paragraph 5)]"));
    assert!(assembled.context_block.contains("zero-trust security"));
}

#[test]
fn test_word_prompt_templates_synthesis() {
    let summarize = WordPromptTemplate::SummarizeSection {
        section_name: "Executive Summary".to_string(),
        focus_bullet_count: 3,
        target_length: "150 words".to_string(),
    };
    let compiled_sum = summarize.compile();
    assert!(compiled_sum.user_prompt.contains("Executive Summary"));
    assert!(compiled_sum.user_prompt.contains("exactly 3 bullet points"));

    let rewrite = WordPromptTemplate::RewriteConcise {
        original_text: "In order to facilitate the completion of the project, we must utilize all available resources."
            .to_string(),
        target_reduction_pct: 40,
        preserve_terms: vec!["project".to_string(), "resources".to_string()],
    };
    let compiled_rw = rewrite.compile();
    assert!(compiled_rw.user_prompt.contains("40% more concise"));
    assert!(compiled_rw.user_prompt.contains("project, resources"));

    let tone = WordPromptTemplate::FixToneAndGrammar {
        original_text: "hey boss we gotta fix this asap".to_string(),
        target_tone: "Executive Formal".to_string(),
    };
    let compiled_tone = tone.compile();
    assert!(compiled_tone.user_prompt.contains("Executive Formal"));
}

#[test]
fn test_calc_prompt_templates_synthesis() {
    let explain = CalcPromptTemplate::ExplainFormula {
        formula: "=XLOOKUP(A2, Products!A:A, Products!C:C, 0)".to_string(),
        cell_ref: "D2".to_string(),
        referenced_cells: vec![("A2".to_string(), "SKU-992".to_string())],
    };
    let compiled_exp = explain.compile();
    assert!(compiled_exp.user_prompt.contains("=XLOOKUP"));
    assert!(compiled_exp.user_prompt.contains("cell D2"));
    assert!(compiled_exp.user_prompt.contains("Cell A2: SKU-992"));

    let synth = CalcPromptTemplate::SynthesizeFormula {
        intent: "Calculate weighted average price".to_string(),
        available_columns: vec!["Quantity".to_string(), "UnitPrice".to_string()],
        target_cell: "E15".to_string(),
    };
    let compiled_synth = synth.compile();
    assert!(compiled_synth.user_prompt.contains("target cell E15"));
    assert!(compiled_synth
        .user_prompt
        .contains("Calculate weighted average price"));
    assert!(compiled_synth.user_prompt.contains("Quantity, UnitPrice"));
}

#[test]
fn test_slides_prompt_templates_synthesis() {
    let outline = SlidesPromptTemplate::GenerateSlideFromOutline {
        outline_title: "Product Differentiation".to_string(),
        outline_points: vec![
            "Zero Cloud Telemetry".to_string(),
            "Instant 120fps Native Rust Rendering".to_string(),
            "Complete Offline Privacy".to_string(),
        ],
        target_shape_count: 3,
    };
    let compiled_outline = outline.compile();
    assert!(compiled_outline
        .user_prompt
        .contains("Product Differentiation"));
    assert!(compiled_outline
        .user_prompt
        .contains("3 distinct visual cards/shapes"));

    let layout = SlidesPromptTemplate::SuggestLayout {
        slide_title: "Financial Comparison".to_string(),
        bullet_count: 4,
        has_media: false,
        has_table: true,
    };
    let compiled_layout = layout.compile();
    assert!(compiled_layout.user_prompt.contains("Financial Comparison"));
    assert!(compiled_layout.user_prompt.contains("Table Present: true"));

    let notes = SlidesPromptTemplate::CreateSpeakerNotes {
        slide_title: "Closing Keynote".to_string(),
        slide_bullets: vec!["Call to Action".to_string(), "Next Milestones".to_string()],
        target_duration_secs: 60,
    };
    let compiled_notes = notes.compile();
    assert!(compiled_notes.user_prompt.contains("Closing Keynote"));
    assert!(compiled_notes.user_prompt.contains("~60 seconds"));
}
