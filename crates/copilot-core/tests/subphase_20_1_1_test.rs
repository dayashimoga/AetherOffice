use copilot_core::{
    cosine_similarity, ChunkerConfig, DocumentKind, LocalEmbeddingGenerator, LocalTokenizer,
    QuantizedVector, SemanticChunker, SlideInput, SlideShapeInput, VectorIndex, WordParagraphInput,
    EMBEDDING_DIM,
};

#[test]
fn test_tokenizer_special_tokens_and_subwords() {
    let tokenizer = LocalTokenizer::new();
    let text = "Quarterly revenue report for executive presentation";
    let tokens = tokenizer.tokenize(text);

    assert!(tokens.len() >= 4);
    assert_eq!(tokens.first().copied(), Some(copilot_core::CLS_TOKEN_ID));
    assert_eq!(tokens.last().copied(), Some(copilot_core::SEP_TOKEN_ID));

    let decoded = tokenizer.decode(&tokens);
    assert!(decoded.contains("revenue"));
    assert!(decoded.contains("presentation"));
}

#[test]
fn test_semantic_chunker_word_hierarchical_context() {
    let chunker = SemanticChunker::new(ChunkerConfig {
        max_tokens_per_chunk: 50,
        overlap_tokens: 15,
        min_chunk_tokens: 10,
    });

    let paragraphs = vec![
        WordParagraphInput {
            section: 1,
            paragraph_index: 0,
            heading_level: Some(1),
            text: "Financial Overview".to_string(),
        },
        WordParagraphInput {
            section: 1,
            paragraph_index: 1,
            heading_level: Some(2),
            text: "Q3 Earnings".to_string(),
        },
        WordParagraphInput {
            section: 1,
            paragraph_index: 2,
            heading_level: None,
            text: "In the third quarter, enterprise subscription revenue increased by 24% year over year. Operating margins improved to 31.5% driven by strong cloud adoption.".to_string(),
        },
        WordParagraphInput {
            section: 1,
            paragraph_index: 3,
            heading_level: None,
            text: "Operating expenses remained flat due to optimized infrastructure spending. Net cash flow reached an all-time record.".to_string(),
        },
    ];

    let chunks = chunker
        .chunk_word_document("doc_annual_report", "Annual Report 2026", &paragraphs)
        .expect("word chunking failed");

    assert!(!chunks.is_empty());
    assert_eq!(chunks[0].kind, DocumentKind::Word);
    assert!(chunks[0].structural_context.contains("Annual Report 2026"));
    assert!(chunks[0]
        .structural_context
        .contains("H1: Financial Overview"));
    assert!(chunks[0].structural_context.contains("H2: Q3 Earnings"));
    assert!(chunks[0].composite_text.contains("subscription revenue"));
}

#[test]
fn test_semantic_chunker_calc_spreadsheet_ranges() {
    let chunker = SemanticChunker::default();
    let headers = vec![
        "Department".to_string(),
        "Budget".to_string(),
        "Actual".to_string(),
        "Variance".to_string(),
    ];
    let rows = vec![
        vec![
            "Engineering".to_string(),
            "$1,500,000".to_string(),
            "$1,420,000".to_string(),
            "-$80,000".to_string(),
        ],
        vec![
            "Marketing".to_string(),
            "$800,000".to_string(),
            "$850,000".to_string(),
            "+$50,000".to_string(),
        ],
        vec![
            "Operations".to_string(),
            "$400,000".to_string(),
            "$390,000".to_string(),
            "-$10,000".to_string(),
        ],
    ];

    let chunks = chunker
        .chunk_calc_sheet("calc_budget_2026", "Summary", &headers, &rows, 2)
        .expect("calc chunking failed");

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].kind, DocumentKind::Calc);
    assert!(chunks[0].structural_context.contains("Sheet: Summary"));
    assert!(chunks[0].structural_context.contains("Range: A2:D3"));
    assert!(chunks[0].raw_text.contains("Department: Engineering"));
    assert!(chunks[0].raw_text.contains("Budget: $1,500,000"));
}

#[test]
fn test_semantic_chunker_slides_presentation() {
    let chunker = SemanticChunker::default();
    let slides = vec![SlideInput {
        slide_number: 1,
        slide_title: Some("Vision & Roadmap".to_string()),
        shapes: vec![
            SlideShapeInput {
                shape_id: "shape_1".to_string(),
                shape_name: "Title".to_string(),
                text: "Vision & Roadmap 2027".to_string(),
            },
            SlideShapeInput {
                shape_id: "shape_2".to_string(),
                shape_name: "BulletList".to_string(),
                text: "Accelerate Rust-native Office suite adoption across enterprise clients."
                    .to_string(),
            },
        ],
        notes: Some("Emphasize zero cloud telemetry and local neural copilot privacy.".to_string()),
    }];

    let chunks = chunker
        .chunk_slides("pres_strategy", "Corporate Strategy", &slides)
        .expect("slides chunking failed");

    // Expect 2 chunks: one for slide body and one for speaker notes
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].kind, DocumentKind::Slides);
    assert!(chunks[0]
        .structural_context
        .contains("Slide 1: Vision & Roadmap"));
    assert!(chunks[0].raw_text.contains("Rust-native Office suite"));

    assert_eq!(chunks[1].kind, DocumentKind::Slides);
    assert!(chunks[1].structural_context.contains("Speaker Notes"));
    assert!(chunks[1].raw_text.contains("zero cloud telemetry"));
}

#[test]
fn test_embedding_determinism_and_dimension() {
    let generator = LocalEmbeddingGenerator::new();
    let text = "Quarterly financial forecast showing revenue growth and profit margins.";

    let emb1 = generator.embed_text(text);
    let emb2 = generator.embed_text(text);

    assert_eq!(emb1.len(), EMBEDDING_DIM);
    assert_eq!(emb2.len(), EMBEDDING_DIM);

    // Deterministic equality
    assert_eq!(emb1, emb2);

    // Unit norm check: sum(v_i^2) ≈ 1.0
    let norm_sq: f32 = emb1.iter().map(|v| v * v).sum();
    assert!((norm_sq - 1.0).abs() < 1e-4);

    // Self cosine similarity is 1.0
    let sim_self = cosine_similarity(&emb1, &emb2).expect("cosine failed");
    assert!((sim_self - 1.0).abs() < 1e-5);
}

#[test]
fn test_semantic_embedding_relevance() {
    let generator = LocalEmbeddingGenerator::new();

    let text_fin1 = "Q3 financial revenue report with earnings per share and profit.";
    let text_fin2 = "Quarterly fiscal budget statement detailing corporate income and expense.";
    let text_astronomy = "Jupiter moons and astronomical observation telescope focal length.";

    let vec_fin1 = generator.embed_text(text_fin1);
    let vec_fin2 = generator.embed_text(text_fin2);
    let vec_astro = generator.embed_text(text_astronomy);

    let sim_related = cosine_similarity(&vec_fin1, &vec_fin2).expect("sim failed");
    let sim_unrelated = cosine_similarity(&vec_fin1, &vec_astro).expect("sim failed");

    // Related financial documents should have significantly higher similarity than unrelated astronomy text
    assert!(sim_related > sim_unrelated);
    assert!(sim_related > 0.4);
}

#[test]
fn test_quantized_vector_precision_and_dot_product() {
    let generator = LocalEmbeddingGenerator::new();
    let vec_a = generator.embed_text("Operating income statement");
    let vec_b = generator.embed_text("Annual operating expenditure");

    let quant_a = QuantizedVector::from_f32(&vec_a);
    let quant_b = QuantizedVector::from_f32(&vec_b);

    assert_eq!(quant_a.data.len(), EMBEDDING_DIM);
    assert_eq!(quant_b.data.len(), EMBEDDING_DIM);

    let f32_sim = cosine_similarity(&vec_a, &vec_b).expect("sim failed");
    let quant_sim = quant_a.dot_product(&quant_b).expect("quant sim failed");

    // Int8 dot product should be very close to f32 dot product (within 0.05)
    assert!((f32_sim - quant_sim).abs() < 0.05);
}

#[test]
fn test_vector_index_add_and_search_precision() {
    let generator = LocalEmbeddingGenerator::new();
    let chunker = SemanticChunker::default();
    let mut index = VectorIndex::new();

    let p1 = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: Some(1),
        text: "Cloud Infrastructure Architecture and Rust Microservices".to_string(),
    }];
    let p2 = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: Some(1),
        text: "Fiscal Year 2026 Net Profit, EBITDA and Revenue Projections".to_string(),
    }];

    let c1 = chunker
        .chunk_word_document("doc_tech", "Tech Spec", &p1)
        .unwrap();
    let c2 = chunker
        .chunk_word_document("doc_fin", "Financials", &p2)
        .unwrap();

    let emb1 = generator.embed_text(&c1[0].composite_text);
    let emb2 = generator.embed_text(&c2[0].composite_text);

    index.add_chunk(c1[0].clone(), emb1).unwrap();
    index.add_chunk(c2[0].clone(), emb2).unwrap();

    assert_eq!(index.total_chunks(), 2);
    assert_eq!(index.total_documents(), 2);

    // Query for finance
    let query_vec = generator.embed_text("revenue projections and profit margin");
    let results = index.search(&query_vec, 1, 0.0).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].chunk.doc_id, "doc_fin");
}

#[test]
fn test_vector_index_incremental_update_and_purge() {
    let generator = LocalEmbeddingGenerator::new();
    let chunker = SemanticChunker::default();
    let mut index = VectorIndex::new();

    let p = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: None,
        text: "Draft v1 of marketing agreement.".to_string(),
    }];
    let chunks = chunker
        .chunk_word_document("doc_draft", "Draft", &p)
        .unwrap();
    let emb = generator.embed_text(&chunks[0].composite_text);
    index.add_chunk(chunks[0].clone(), emb).unwrap();

    assert_eq!(index.total_chunks(), 1);

    // Purge document
    let removed = index.remove_document("doc_draft");
    assert_eq!(removed, 1);
    assert_eq!(index.total_chunks(), 0);
    assert_eq!(index.total_documents(), 0);

    // Search on empty index
    let query_vec = generator.embed_text("marketing agreement");
    let results = index.search(&query_vec, 5, 0.0).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_vector_index_filtered_search() {
    let generator = LocalEmbeddingGenerator::new();
    let chunker = SemanticChunker::default();
    let mut index = VectorIndex::new();

    let p1 = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: None,
        text: "Document A paragraph text.".to_string(),
    }];
    let p2 = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: None,
        text: "Document B paragraph text.".to_string(),
    }];

    let c1 = chunker.chunk_word_document("doc_a", "Doc A", &p1).unwrap();
    let c2 = chunker.chunk_word_document("doc_b", "Doc B", &p2).unwrap();

    index
        .add_chunk(c1[0].clone(), generator.embed_text(&c1[0].composite_text))
        .unwrap();
    index
        .add_chunk(c2[0].clone(), generator.embed_text(&c2[0].composite_text))
        .unwrap();

    let query_vec = generator.embed_text("paragraph text");
    // Filter to only doc_b
    let filtered_results = index
        .search_filtered(&query_vec, 10, 0.0, |chunk| chunk.doc_id == "doc_b")
        .unwrap();

    assert_eq!(filtered_results.len(), 1);
    assert_eq!(filtered_results[0].chunk.doc_id, "doc_b");
}

#[test]
fn test_vector_index_serialization_roundtrip() {
    let generator = LocalEmbeddingGenerator::new();
    let chunker = SemanticChunker::default();
    let mut index = VectorIndex::new();

    let p = vec![WordParagraphInput {
        section: 1,
        paragraph_index: 0,
        heading_level: None,
        text: "Serialization validation content.".to_string(),
    }];
    let chunks = chunker
        .chunk_word_document("doc_ser", "Serializer", &p)
        .unwrap();
    index
        .add_chunk(
            chunks[0].clone(),
            generator.embed_text(&chunks[0].composite_text),
        )
        .unwrap();

    let json = index.to_json().expect("to_json failed");
    let restored = VectorIndex::from_json(&json).expect("from_json failed");

    assert_eq!(restored.total_chunks(), 1);
    assert_eq!(restored.total_documents(), 1);
}
