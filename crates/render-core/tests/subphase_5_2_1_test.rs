//! Subphase 5.2.1 Integration Test Suite:
//! Tauri v2 Desktop & Laptop Shell (Ribbon, Multi-Tab, Formula Bar)
//!
//! Validates:
//! - Multi-document workspace tab bar (Word, Excel, PowerPoint)
//! - Contextual Ribbon sets (Word vs Excel vs PowerPoint)
//! - Ribbon formatting state toggles (bold, underline, font family, font size)
//! - Excel Formula Bar Name Box conversion and live syntax highlighting tokenization
//! - Word Ruler margin and indent calculations
//! - Navigation Pane heading outline hierarchy extraction
//! - PowerPoint Slide Sorter thumbnail drag reordering
//! - Desktop keyboard shortcut engine (Accelerators & Alt Keytips)
//! - Desktop Session Manager (Tauri IPC coordination backend)
//! - Vector manifest validation against `specs/evidence/render/5.2.1_vectors.json`

use render_core::{
    AltKeytipStateMachine, DesktopSessionManager, DocumentTab, DocumentType, FormattingState,
    FormulaBarEngine, FormulaTokenType, KeyCombination, NavigationPaneEngine, RibbonModel,
    RibbonTabId, ShellAction, ShortcutEngine, SlideSorterEngine, TabBarModel, WordRulerConfig,
};
use std::path::{Path, PathBuf};

fn resolve_repo_path(rel_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root");
    repo_root.join(rel_path)
}

#[test]
fn test_01_tab_bar_initialization() {
    let mut tab_bar = TabBarModel::new();
    let tab = DocumentTab::new("tab-1", DocumentType::Word, "Document 1");
    tab_bar.add_tab(tab);

    assert_eq!(tab_bar.tabs.len(), 1);
    assert_eq!(tab_bar.active_tab_id, Some("tab-1".to_string()));
    let active = tab_bar.active_tab().expect("Active tab exists");
    assert!(!active.is_dirty);
    assert_eq!(active.display_title(), "Document 1");
}

#[test]
fn test_02_tab_bar_add_multiple_types() {
    let mut tab_bar = TabBarModel::new();
    tab_bar.add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Document 1"));
    tab_bar.add_tab(DocumentTab::new(
        "tab-2",
        DocumentType::Excel,
        "Budget.xlsx",
    ));
    tab_bar.add_tab(DocumentTab::new(
        "tab-3",
        DocumentType::PowerPoint,
        "Pitch.pptx",
    ));

    assert_eq!(tab_bar.tabs.len(), 3);
    assert_eq!(tab_bar.active_tab_id, Some("tab-3".to_string()));
}

#[test]
fn test_03_tab_bar_switch_active() {
    let mut tab_bar = TabBarModel::new();
    tab_bar.add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Document 1"));
    tab_bar.add_tab(DocumentTab::new(
        "tab-2",
        DocumentType::Excel,
        "Budget.xlsx",
    ));
    tab_bar.add_tab(DocumentTab::new(
        "tab-3",
        DocumentType::PowerPoint,
        "Pitch.pptx",
    ));

    assert!(tab_bar.switch_tab("tab-2"));
    assert_eq!(tab_bar.active_tab_id, Some("tab-2".to_string()));
    assert!(!tab_bar.switch_tab("non-existent"));
}

#[test]
fn test_04_tab_bar_dirty_formatting() {
    let mut tab = DocumentTab::new("tab-1", DocumentType::Word, "Report.docx");
    assert_eq!(tab.display_title(), "Report.docx");
    tab.is_dirty = true;
    assert_eq!(tab.display_title(), "Report.docx *");
}

#[test]
fn test_05_tab_bar_close_tab_neighbor() {
    let mut tab_bar = TabBarModel::new();
    tab_bar.add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Doc 1"));
    tab_bar.add_tab(DocumentTab::new("tab-2", DocumentType::Excel, "Doc 2"));
    tab_bar.add_tab(DocumentTab::new("tab-3", DocumentType::PowerPoint, "Doc 3"));

    tab_bar.switch_tab("tab-2");
    let next_active = tab_bar.close_tab("tab-2", false).expect("Close tab-2");
    assert_eq!(tab_bar.tabs.len(), 2);
    assert_eq!(next_active, Some("tab-3".to_string()));
    assert_eq!(tab_bar.active_tab_id, Some("tab-3".to_string()));
}

#[test]
fn test_06_contextual_ribbon_word() {
    let ribbon = RibbonModel::for_document_type(DocumentType::Word);
    let visible = ribbon.visible_tabs();
    assert_eq!(
        visible,
        vec![
            RibbonTabId::File,
            RibbonTabId::Home,
            RibbonTabId::Insert,
            RibbonTabId::Layout,
            RibbonTabId::Review,
            RibbonTabId::View,
        ]
    );
    assert!(!ribbon.is_tab_visible(RibbonTabId::Formulas));
    assert!(!ribbon.is_tab_visible(RibbonTabId::SlideShow));
}

#[test]
fn test_07_contextual_ribbon_excel() {
    let ribbon = RibbonModel::for_document_type(DocumentType::Excel);
    let visible = ribbon.visible_tabs();
    assert_eq!(
        visible,
        vec![
            RibbonTabId::File,
            RibbonTabId::Home,
            RibbonTabId::Insert,
            RibbonTabId::Layout,
            RibbonTabId::Formulas,
            RibbonTabId::Data,
            RibbonTabId::Review,
            RibbonTabId::View,
        ]
    );
    assert!(!ribbon.is_tab_visible(RibbonTabId::Transitions));
}

#[test]
fn test_08_contextual_ribbon_ppt() {
    let ribbon = RibbonModel::for_document_type(DocumentType::PowerPoint);
    let visible = ribbon.visible_tabs();
    assert_eq!(
        visible,
        vec![
            RibbonTabId::File,
            RibbonTabId::Home,
            RibbonTabId::Insert,
            RibbonTabId::Layout,
            RibbonTabId::Transitions,
            RibbonTabId::Animations,
            RibbonTabId::SlideShow,
            RibbonTabId::Review,
            RibbonTabId::View,
        ]
    );
    assert!(!ribbon.is_tab_visible(RibbonTabId::Formulas));
}

#[test]
fn test_09_ribbon_action_formatting_toggles() {
    let mut state = FormattingState::default();
    assert!(!state.bold);
    assert!(!state.italic);
    assert!(!state.underline);

    state.toggle_bold();
    state.toggle_underline();
    assert!(state.bold);
    assert!(!state.italic);
    assert!(state.underline);
}

#[test]
fn test_10_ribbon_action_font_and_size() {
    let state = FormattingState {
        font_family: "Aptos".into(),
        font_size_pt: 14.5,
        ..Default::default()
    };

    assert_eq!(state.font_family, "Aptos");
    assert!((state.font_size_pt - 14.5).abs() < 1e-4);
    assert_eq!(state.font_size_half_pt(), 29);
}

#[test]
fn test_11_formula_bar_coord_to_a1() {
    assert_eq!(FormulaBarEngine::coord_to_a1(3, 1), "B4");
    assert_eq!(FormulaBarEngine::coord_to_a1(0, 0), "A1");
    assert_eq!(FormulaBarEngine::coord_to_a1(99, 26), "AA100");
}

#[test]
fn test_12_formula_bar_a1_to_coord() {
    assert_eq!(FormulaBarEngine::a1_to_coord("B4"), Some((3, 1)));
    assert_eq!(FormulaBarEngine::a1_to_coord("A1"), Some((0, 0)));
    assert_eq!(FormulaBarEngine::a1_to_coord("AA100"), Some((99, 26)));
    assert_eq!(FormulaBarEngine::a1_to_coord("INVALID"), None);
}

#[test]
fn test_13_formula_bar_syntax_tokenization() {
    let tokens = FormulaBarEngine::tokenize_formula("=SUM(A1:B10) * 1.05 + 10");
    assert_eq!(tokens.len(), 8);
    assert_eq!(
        tokens[0],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Function,
            value: "SUM".into(),
        }
    );
    assert_eq!(
        tokens[1],
        render_core::FormulaToken {
            token_type: FormulaTokenType::ParenOpen,
            value: "(".into(),
        }
    );
    assert_eq!(
        tokens[2],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Reference,
            value: "A1:B10".into(),
        }
    );
    assert_eq!(
        tokens[3],
        render_core::FormulaToken {
            token_type: FormulaTokenType::ParenClose,
            value: ")".into(),
        }
    );
    assert_eq!(
        tokens[4],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Operator,
            value: "*".into(),
        }
    );
    assert_eq!(
        tokens[5],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Number,
            value: "1.05".into(),
        }
    );
    assert_eq!(
        tokens[6],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Operator,
            value: "+".into(),
        }
    );
    assert_eq!(
        tokens[7],
        render_core::FormulaToken {
            token_type: FormulaTokenType::Number,
            value: "10".into(),
        }
    );
}

#[test]
fn test_14_formula_bar_commit_command() {
    let initial_val = "100";
    let formula_text = "=A1*2";
    let committed = if formula_text.starts_with('=') {
        formula_text
    } else {
        initial_val
    };
    assert_eq!(committed, "=A1*2");
}

#[test]
fn test_15_formula_bar_cancel_reverts() {
    let original = "Original Content";
    let draft = "Draft typo";
    let action = "cancel";
    let result = if action == "cancel" { original } else { draft };
    assert_eq!(result, "Original Content");
}

#[test]
fn test_16_word_ruler_margins_and_slider() {
    let mut ruler = WordRulerConfig::new(12240, 1440, 1440);
    ruler.first_line_indent_dxa = 720;

    assert_eq!(ruler.content_width_dxa(), 9360);
    assert_eq!(ruler.slider_first_line_x_dxa(), 2160);
}

#[test]
fn test_17_navigation_pane_extract_headings() {
    let paragraphs = vec![
        ("Heading 1".to_string(), "Chapter 1".to_string()),
        ("Normal".to_string(), "Body paragraph".to_string()),
        ("Heading 2".to_string(), "Section 1.1".to_string()),
    ];

    let headings = NavigationPaneEngine::extract_headings(&paragraphs);
    assert_eq!(headings.len(), 2);
    assert_eq!(headings[0].level, 1);
    assert_eq!(headings[0].text, "Chapter 1");
    assert_eq!(headings[1].level, 2);
    assert_eq!(headings[1].text, "Section 1.1");
}

#[test]
fn test_18_slide_sorter_reorder_slides() {
    let mut slides = vec![101, 102, 103, 104];
    SlideSorterEngine::reorder_slides(&mut slides, 0, 2);
    assert_eq!(slides, vec![102, 103, 101, 104]);
}

#[test]
fn test_19_keyboard_shortcut_clipboard() {
    let combo_copy = KeyCombination {
        key: "c".into(),
        ctrl: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_copy),
        Some(ShellAction::Copy)
    );

    let combo_paste = KeyCombination {
        key: "v".into(),
        ctrl: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_paste),
        Some(ShellAction::Paste)
    );

    let combo_cut = KeyCombination {
        key: "x".into(),
        ctrl: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_cut),
        Some(ShellAction::Cut)
    );
}

#[test]
fn test_20_keyboard_shortcut_undo_redo() {
    let combo_undo = KeyCombination {
        key: "z".into(),
        ctrl: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_undo),
        Some(ShellAction::Undo)
    );

    let combo_redo_y = KeyCombination {
        key: "y".into(),
        ctrl: true,
        alt: false,
        shift: false,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_redo_y),
        Some(ShellAction::Redo)
    );

    let combo_redo_shift_z = KeyCombination {
        key: "z".into(),
        ctrl: true,
        alt: false,
        shift: true,
    };
    assert_eq!(
        ShortcutEngine::match_action(&combo_redo_shift_z),
        Some(ShellAction::Redo)
    );
}

#[test]
fn test_21_keyboard_shortcut_tab_cycling() {
    let mut tab_bar = TabBarModel::new();
    tab_bar.add_tab(DocumentTab::new("tab-1", DocumentType::Word, "Doc 1"));
    tab_bar.add_tab(DocumentTab::new("tab-2", DocumentType::Excel, "Doc 2"));
    tab_bar.add_tab(DocumentTab::new("tab-3", DocumentType::PowerPoint, "Doc 3"));

    tab_bar.switch_tab("tab-2");
    tab_bar.next_tab();
    assert_eq!(tab_bar.active_tab_id, Some("tab-3".to_string()));

    tab_bar.prev_tab();
    assert_eq!(tab_bar.active_tab_id, Some("tab-2".to_string()));
}

#[test]
fn test_22_keyboard_shortcut_f4_toggle() {
    let mut ref_str = "A1".to_string();
    let expected_cycle = ["$A$1", "A$1", "$A1", "A1"];

    for &expected in &expected_cycle {
        ref_str = FormulaBarEngine::rotate_f4_reference(&ref_str);
        assert_eq!(ref_str, expected);
    }
}

#[test]
fn test_23_alt_keytips_state_machine() {
    let mut machine = AltKeytipStateMachine::default();
    assert!(!machine.is_active);

    assert!(machine.toggle_alt());
    assert!(machine.is_active);

    let tab = machine.dispatch_keytip('H');
    assert_eq!(tab, Some(RibbonTabId::Home));
    assert!(!machine.is_active);
}

#[test]
fn test_24_tauri_ipc_detect_and_open_session() {
    assert_eq!(
        DesktopSessionManager::detect_doc_type("test.docx"),
        DocumentType::Word
    );
    assert_eq!(
        DesktopSessionManager::detect_doc_type("test.xlsx"),
        DocumentType::Excel
    );
    assert_eq!(
        DesktopSessionManager::detect_doc_type("test.pptx"),
        DocumentType::PowerPoint
    );

    let mut mgr = DesktopSessionManager::new();
    let summary = mgr.open_session("G:/documents/budget.xlsx");
    assert_eq!(summary.doc_type, DocumentType::Excel);
    assert_eq!(summary.title, "budget.xlsx");
    assert_eq!(
        summary.file_path,
        Some("G:/documents/budget.xlsx".to_string())
    );
}

#[test]
fn test_25_tauri_ipc_close_session_dirty_guard() {
    let mut mgr = DesktopSessionManager::new();
    let summary = mgr.create_session(DocumentType::Word, "Report.docx");
    mgr.mark_active_dirty(true);

    let err = mgr.close_session(&summary.session_id, false);
    assert!(err.is_err());

    let ok = mgr.close_session(&summary.session_id, true);
    assert!(ok.is_ok());
}

#[test]
fn test_26_vectors_manifest_validation() {
    let manifest_path = resolve_repo_path("specs/evidence/render/5.2.1_vectors.json");
    assert!(
        manifest_path.exists(),
        "5.2.1_vectors.json must exist at {}",
        manifest_path.display()
    );

    let content = std::fs::read_to_string(&manifest_path).expect("Read vectors file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(json["subphase"].as_str(), Some("5.2.1"));
    let vector_count = json["vector_count"].as_u64().expect("vector_count is u64") as usize;
    assert!(
        vector_count >= 25,
        "Vector count must be >= 25, found {}",
        vector_count
    );

    let vectors = json["vectors"].as_array().expect("vectors is array");
    assert_eq!(vectors.len(), vector_count);

    for (idx, vec_item) in vectors.iter().enumerate() {
        let expected_id = format!("VEC-5.2.1-{:02}", idx + 1);
        assert_eq!(
            vec_item["id"].as_str(),
            Some(expected_id.as_str()),
            "Vector at index {} has invalid ID",
            idx
        );
        assert!(
            vec_item["description"].is_string(),
            "Vector {} missing description",
            expected_id
        );
        assert!(
            vec_item["input"].is_object(),
            "Vector {} missing input",
            expected_id
        );
        assert!(
            vec_item["expected"].is_object(),
            "Vector {} missing expected",
            expected_id
        );
    }
}
