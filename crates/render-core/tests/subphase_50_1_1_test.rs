//! Sub-Phase 50.1.1 Integration Tests: Cross-Document Dependency Graph & Live Embeds
//!
//! Complies strictly with GEC-v7.0 Article I (Architectural Integrity & Zero Global Document Singletons)
//! and Article III (Real Binary File Open & Save, Zero Data Loss):
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in production library code.
//! - Directed Acyclic Graph (DAG) cycle detection (Tarjan / BFS cycle detection).
//! - Topological invalidation propagation and snapshot staleness tracking.

use render_core::cross_doc::{
    CrossDocDependencyGraph, CrossDocError, DependencyEdge, DependencyKind, EmbedAnchor,
    EmbedTarget, LiveEmbedSnapshot,
};

fn make_edge(
    id: &str,
    source: &str,
    target: &str,
    source_anchor: EmbedAnchor,
    target_container: EmbedTarget,
) -> DependencyEdge {
    DependencyEdge {
        id: id.to_string(),
        source_doc_id: source.to_string(),
        source_anchor,
        target_doc_id: target.to_string(),
        target_container,
        kind: DependencyKind::LiveRangeEmbed,
        created_at: 1000,
        version: 1,
    }
}

#[test]
fn test_edge_registration_and_queries() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor = EmbedAnchor::CalcRange {
        sheet_name: "Q4_Revenue".to_string(),
        range_a1: "B2:F12".to_string(),
    };
    let target = EmbedTarget::WordInline {
        container_id: "report-table-1".to_string(),
    };

    let edge = make_edge("edge-1", "doc-calc-1", "doc-word-1", anchor.clone(), target);
    let res = graph.register_dependency(edge.clone());
    assert_eq!(res, Ok("edge-1".to_string()));

    let dependents = graph.get_dependents("doc-calc-1");
    assert_eq!(dependents.len(), 1);
    assert_eq!(dependents[0].id, "edge-1");
    assert_eq!(dependents[0].target_doc_id, "doc-word-1");

    let dependencies = graph.get_dependencies("doc-word-1");
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].id, "edge-1");
    assert_eq!(dependencies[0].source_doc_id, "doc-calc-1");

    assert!(graph.get_dependents("doc-word-1").is_empty());
    assert!(graph.get_dependencies("doc-calc-1").is_empty());
}

#[test]
fn test_self_referential_cycle_rejection() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor = EmbedAnchor::CalcCell {
        sheet_name: "Sheet1".to_string(),
        coordinate: "A1".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "ref-1".to_string(),
    };

    let edge = make_edge("self-edge", "doc-a", "doc-a", anchor, target);
    let res = graph.register_dependency(edge);

    assert_eq!(
        res,
        Err(CrossDocError::CircularDependency {
            cycle: vec!["doc-a".to_string()]
        })
    );
}

#[test]
fn test_direct_cycle_rejection() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor_a = EmbedAnchor::WordParagraph {
        paragraph_id: "p1".to_string(),
    };
    let target_b = EmbedTarget::NotesBlock {
        block_id: "b1".to_string(),
    };

    let edge_ab = make_edge("edge-ab", "doc-a", "doc-b", anchor_a, target_b);
    assert!(graph.register_dependency(edge_ab).is_ok());

    let anchor_b = EmbedAnchor::GenericAnchor {
        key: "key-1".to_string(),
    };
    let target_a = EmbedTarget::WordInline {
        container_id: "c1".to_string(),
    };

    let edge_ba = make_edge("edge-ba", "doc-b", "doc-a", anchor_b, target_a);
    let res = graph.register_dependency(edge_ba);

    assert!(matches!(res, Err(CrossDocError::CircularDependency { .. })));
    assert_eq!(graph.get_dependents("doc-b").len(), 0);
}

#[test]
fn test_transitive_cycle_rejection() {
    let mut graph = CrossDocDependencyGraph::new();

    // A -> B -> C -> D
    let anchor = EmbedAnchor::GenericAnchor {
        key: "k".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "t".to_string(),
    };

    assert!(graph
        .register_dependency(make_edge(
            "e1",
            "doc-a",
            "doc-b",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge(
            "e2",
            "doc-b",
            "doc-c",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge(
            "e3",
            "doc-c",
            "doc-d",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());

    // Attempt D -> A (would create cycle D -> A -> B -> C -> D)
    let res = graph.register_dependency(make_edge("e4", "doc-d", "doc-a", anchor, target));
    assert!(matches!(res, Err(CrossDocError::CircularDependency { .. })));
}

#[test]
fn test_diamond_dag_allowed() {
    let mut graph = CrossDocDependencyGraph::new();

    // Diamond: A -> B, A -> C, B -> D, C -> D (Valid DAG)
    let anchor = EmbedAnchor::GenericAnchor {
        key: "k".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "t".to_string(),
    };

    assert!(graph
        .register_dependency(make_edge(
            "e-ab",
            "doc-a",
            "doc-b",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge(
            "e-ac",
            "doc-a",
            "doc-c",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge(
            "e-bd",
            "doc-b",
            "doc-d",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge("e-cd", "doc-c", "doc-d", anchor, target))
        .is_ok());

    let summary = graph.graph_summary();
    assert_eq!(summary.total_nodes, 4);
    assert_eq!(summary.total_edges, 4);
}

#[test]
fn test_edge_removal_and_cleanup() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor = EmbedAnchor::CalcChart {
        sheet_name: "Sales".to_string(),
        chart_id: "Chart_1".to_string(),
    };
    let target = EmbedTarget::SlideShapeContainer {
        slide_index: 2,
        shape_id: 10,
    };

    let edge = make_edge("edge-rem", "doc-calc", "doc-slides", anchor, target);
    assert!(graph.register_dependency(edge).is_ok());

    let removed = graph.remove_dependency("edge-rem");
    assert_eq!(removed, Ok(true));

    assert!(graph.get_dependents("doc-calc").is_empty());
    assert!(graph.get_dependencies("doc-slides").is_empty());

    let not_found = graph.remove_dependency("edge-rem");
    assert_eq!(
        not_found,
        Err(CrossDocError::EdgeNotFound("edge-rem".to_string()))
    );
}

#[test]
fn test_topological_invalidation_propagation() {
    let mut graph = CrossDocDependencyGraph::new();

    // A -> B -> C and A -> D
    let anchor = EmbedAnchor::GenericAnchor {
        key: "k".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "t".to_string(),
    };

    assert!(graph
        .register_dependency(make_edge(
            "e1",
            "doc-a",
            "doc-b",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge(
            "e2",
            "doc-b",
            "doc-c",
            anchor.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge("e3", "doc-a", "doc-d", anchor, target))
        .is_ok());

    let affected = graph.propagate_invalidation("doc-a", None);
    assert_eq!(affected.len(), 3);
    assert!(affected.contains(&"doc-b".to_string()));
    assert!(affected.contains(&"doc-c".to_string()));
    assert!(affected.contains(&"doc-d".to_string()));
}

#[test]
fn test_anchor_targeted_invalidation() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor_1 = EmbedAnchor::CalcRange {
        sheet_name: "Sheet1".to_string(),
        range_a1: "A1:B10".to_string(),
    };
    let anchor_2 = EmbedAnchor::CalcRange {
        sheet_name: "Sheet1".to_string(),
        range_a1: "C1:D10".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "t".to_string(),
    };

    assert!(graph
        .register_dependency(make_edge(
            "e1",
            "doc-calc",
            "doc-word-1",
            anchor_1.clone(),
            target.clone()
        ))
        .is_ok());
    assert!(graph
        .register_dependency(make_edge("e2", "doc-calc", "doc-word-2", anchor_2, target))
        .is_ok());

    // Invalidate only anchor_1
    let affected = graph.propagate_invalidation("doc-calc", Some(&anchor_1));
    assert_eq!(affected, vec!["doc-word-1".to_string()]);
}

#[test]
fn test_live_embed_snapshot_caching_and_staleness() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor = EmbedAnchor::PdfRegion {
        page_index: 0,
        bbox: [50, 50, 300, 200],
    };
    let target = EmbedTarget::NotesBlock {
        block_id: "note-1".to_string(),
    };

    assert!(graph
        .register_dependency(make_edge(
            "e-pdf",
            "doc-pdf",
            "doc-notes",
            anchor.clone(),
            target
        ))
        .is_ok());

    let snapshot = LiveEmbedSnapshot {
        embed_id: "e-pdf".to_string(),
        source_doc_id: "doc-pdf".to_string(),
        version: 1,
        rendered_html: Some("<svg>PDF Excerpt</svg>".to_string()),
        rendered_svg: Some("<svg></svg>".to_string()),
        tabular_data: None,
        scalar_value: None,
        last_updated_timestamp: 5000,
        is_stale: false,
    };

    graph.update_snapshot(snapshot);
    assert!(!graph.get_snapshot("e-pdf").unwrap().is_stale);

    // Propagate invalidation -> marks snapshot stale
    graph.propagate_invalidation("doc-pdf", Some(&anchor));
    assert!(graph.get_snapshot("e-pdf").unwrap().is_stale);

    // Update with fresh snapshot -> clears staleness
    let updated_snapshot = LiveEmbedSnapshot {
        embed_id: "e-pdf".to_string(),
        source_doc_id: "doc-pdf".to_string(),
        version: 2,
        rendered_html: Some("<svg>Updated Excerpt</svg>".to_string()),
        rendered_svg: Some("<svg></svg>".to_string()),
        tabular_data: None,
        scalar_value: None,
        last_updated_timestamp: 6000,
        is_stale: false,
    };
    graph.update_snapshot(updated_snapshot);
    assert!(!graph.get_snapshot("e-pdf").unwrap().is_stale);
}

#[test]
fn test_cross_doc_summary_dto() {
    let mut graph = CrossDocDependencyGraph::new();

    let anchor = EmbedAnchor::GenericAnchor {
        key: "key".to_string(),
    };
    let target = EmbedTarget::GenericTarget {
        id: "target".to_string(),
    };

    graph
        .register_dependency(make_edge("e1", "d1", "d2", anchor.clone(), target.clone()))
        .unwrap();
    graph
        .register_dependency(make_edge("e2", "d2", "d3", anchor, target))
        .unwrap();

    let snap1 = LiveEmbedSnapshot {
        embed_id: "e1".to_string(),
        source_doc_id: "d1".to_string(),
        version: 1,
        rendered_html: None,
        rendered_svg: None,
        tabular_data: None,
        scalar_value: Some("42".to_string()),
        last_updated_timestamp: 100,
        is_stale: false,
    };
    let snap2 = LiveEmbedSnapshot {
        embed_id: "e2".to_string(),
        source_doc_id: "d2".to_string(),
        version: 1,
        rendered_html: None,
        rendered_svg: None,
        tabular_data: None,
        scalar_value: Some("84".to_string()),
        last_updated_timestamp: 100,
        is_stale: true,
    };

    graph.update_snapshot(snap1);
    graph.update_snapshot(snap2);

    let summary = graph.graph_summary();
    assert_eq!(summary.total_nodes, 3);
    assert_eq!(summary.total_edges, 2);
    assert_eq!(summary.total_snapshots, 2);
    assert_eq!(summary.stale_snapshots, 1);
}
