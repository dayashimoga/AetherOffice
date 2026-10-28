//! Subphase 2.2.1 Integration Test Suite: Pratt Formula Parser, 3D Ranges & Spill AST.
//! Strictly conforms to GEC-v1.0 (Zero Contamination, Zero Panics, SLAs, Lossless Round-Trip).

use excel_engine::formula::{
    compile_formula, decompile_ast, decompile_bytecode, decompile_formula, parse_formula, AstNode,
    BinaryOperator, BytecodeProgram, CellAddress, Lexer, Literal, OpCode, RangeAddress, Reference,
    SheetRef, StructuredTableRef, TableSection, Token, UnaryOperator,
};
use excel_engine::grid::{CellError, CompressedSparsePageGrid};
use std::fs;
use std::path::PathBuf;

fn resolve_fixture(relative: &str) -> PathBuf {
    let direct = PathBuf::from(relative);
    if direct.exists() {
        return direct;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from_manifest = manifest_dir.join("../../").join(relative);
    if from_manifest.exists() {
        return from_manifest;
    }
    direct
}

// ==============================================================================
// 1. LEXICAL ANALYSIS TESTS
// ==============================================================================

#[test]
#[allow(clippy::approx_constant)]
fn test_01_lexer_number_literals_integers_floats_scientific() {
    let mut lexer = Lexer::new("=42 + 3.14159 + 1.23E+10 + .5");
    let tokens = lexer.tokenize().expect("tokenize numbers");
    assert!(matches!(tokens[0].item, Token::Number(n) if (n - 42.0).abs() < f64::EPSILON));
    assert!(matches!(tokens[1].item, Token::Plus));
    assert!(matches!(tokens[2].item, Token::Number(n) if (n - 3.14159).abs() < 1e-5));
    assert!(matches!(tokens[3].item, Token::Plus));
    assert!(matches!(tokens[4].item, Token::Number(n) if (n - 1.23e10).abs() < 1.0));
    assert!(matches!(tokens[5].item, Token::Plus));
    assert!(matches!(tokens[6].item, Token::Number(n) if (n - 0.5).abs() < f64::EPSILON));
}

#[test]
fn test_02_lexer_string_literals_and_escaped_quotes() {
    let mut lexer = Lexer::new("=\"Hello \"\"World\"\"!\"");
    let tokens = lexer.tokenize().expect("tokenize string");
    assert_eq!(
        tokens[0].item,
        Token::String("Hello \"World\"!".to_string())
    );
}

#[test]
fn test_03_lexer_booleans_true_false() {
    let mut lexer = Lexer::new("=TRUE = FALSE");
    let tokens = lexer.tokenize().expect("tokenize booleans");
    assert_eq!(tokens[0].item, Token::Boolean(true));
    assert_eq!(tokens[1].item, Token::Equal);
    assert_eq!(tokens[2].item, Token::Boolean(false));
}

#[test]
fn test_04_lexer_all_cell_error_types() {
    let formula = "=#NULL! + #DIV/0! + #VALUE! + #REF! + #NAME? + #NUM! + #N/A + #SPILL! + #CALC!";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize errors");
    assert_eq!(tokens[0].item, Token::Error(CellError::Null));
    assert_eq!(tokens[2].item, Token::Error(CellError::Div0));
    assert_eq!(tokens[4].item, Token::Error(CellError::Value));
    assert_eq!(tokens[6].item, Token::Error(CellError::Ref));
    assert_eq!(tokens[8].item, Token::Error(CellError::Name));
    assert_eq!(tokens[10].item, Token::Error(CellError::Num));
    assert_eq!(tokens[12].item, Token::Error(CellError::NA));
    assert_eq!(tokens[14].item, Token::Error(CellError::Spill));
    assert_eq!(tokens[16].item, Token::Error(CellError::Calc));
}

#[test]
fn test_05_lexer_single_cell_a1_relative_absolute_mixed() {
    let formula = "=A1 + $B$2 + C$3 + $D4";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize cells");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::new(0, 0, false, false).unwrap()
        })
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::new(1, 1, true, true).unwrap()
        })
    );
    assert_eq!(
        tokens[4].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::new(2, 2, true, false).unwrap()
        })
    );
    assert_eq!(
        tokens[6].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::new(3, 3, false, true).unwrap()
        })
    );
}

#[test]
fn test_06_lexer_2d_cell_ranges() {
    let formula = "=A1:B10 + $C$1:$D$5";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize 2d range");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Range {
            sheet: None,
            range: RangeAddress::new(
                CellAddress::new(0, 0, false, false).unwrap(),
                CellAddress::new(9, 1, false, false).unwrap()
            )
        })
    );
}

#[test]
fn test_07_lexer_whole_column_ranges() {
    let formula = "=A:C + $D:$E";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize whole cols");

    assert!(matches!(
        &tokens[0].item,
        Token::Reference(Reference::Range { range, .. }) if range.is_whole_col && range.start.col == 0 && range.end.col == 2
    ));
}

#[test]
fn test_08_lexer_whole_row_ranges() {
    let formula = "=1:10";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize whole rows");

    assert!(matches!(
        &tokens[0].item,
        Token::Reference(Reference::Range { range, .. }) if range.is_whole_row && range.start.row == 0 && range.end.row == 9
    ));
}

#[test]
fn test_09_lexer_r1c1_absolute_and_relative() {
    let formula = "=R1C1 + R[1]C[2] + RC[-1]";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize r1c1");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::parse_r1c1("R1C1", None).unwrap()
        })
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Cell {
            sheet: None,
            cell: CellAddress::parse_r1c1("R[1]C[2]", Some((0, 0))).unwrap()
        })
    );
}

#[test]
fn test_10_lexer_r1c1_ranges() {
    let formula = "=R1C1:R10C5";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize r1c1 range");

    assert!(matches!(
        &tokens[0].item,
        Token::Reference(Reference::Range { range, .. }) if range.start.row == 0 && range.end.row == 9
    ));
}

#[test]
fn test_11_lexer_unquoted_sheet_reference() {
    let formula = "=Sheet1!A1 + Data_2024!B5:C10";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize unquoted sheet");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Cell {
            sheet: Some(SheetRef::Single("Sheet1".into())),
            cell: CellAddress::parse_a1("A1").unwrap()
        })
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Range {
            sheet: Some(SheetRef::Single("Data_2024".into())),
            range: RangeAddress::parse_a1("B5:C10").unwrap()
        })
    );
}

#[test]
fn test_12_lexer_quoted_sheet_reference_with_spaces_and_symbols() {
    let formula = "='Sales 2024'!B5:D20 + 'John''s Data'!A1";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize quoted sheet");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Range {
            sheet: Some(SheetRef::Single("Sales 2024".into())),
            range: RangeAddress::parse_a1("B5:D20").unwrap()
        })
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Cell {
            sheet: Some(SheetRef::Single("John's Data".into())),
            cell: CellAddress::parse_a1("A1").unwrap()
        })
    );
}

#[test]
fn test_13_lexer_3d_sheet_range_spans() {
    let formula = "=Sheet1:Sheet3!A1:B10 + 'Jan:Dec'!C5:C15";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize 3d sheet spans");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Range {
            sheet: Some(SheetRef::Span("Sheet1".into(), "Sheet3".into())),
            range: RangeAddress::parse_a1("A1:B10").unwrap()
        })
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Range {
            sheet: Some(SheetRef::Span("Jan".into(), "Dec".into())),
            range: RangeAddress::parse_a1("C5:C15").unwrap()
        })
    );
}

#[test]
fn test_14_lexer_structured_table_single_column() {
    let formula = "=Table1[Sales]";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize table single column");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Table(StructuredTableRef {
            table_name: Some("Table1".into()),
            sections: vec![],
            column: Some("Sales".into()),
            end_column: None,
            is_at_syntax: false,
        }))
    );
}

#[test]
fn test_15_lexer_structured_table_headers_and_data_sections() {
    let formula = "=Table1[[#Headers],[Sales]] + DeptSales[[#Data],[Jan]:[Dec]]";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize table sections");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Table(StructuredTableRef {
            table_name: Some("Table1".into()),
            sections: vec![TableSection::Headers],
            column: Some("Sales".into()),
            end_column: None,
            is_at_syntax: false,
        }))
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Table(StructuredTableRef {
            table_name: Some("DeptSales".into()),
            sections: vec![TableSection::Data],
            column: Some("Jan".into()),
            end_column: Some("Dec".into()),
            is_at_syntax: false,
        }))
    );
}

#[test]
fn test_16_lexer_structured_table_this_row_and_at_syntax() {
    let formula = "=Table1[@Amount] + [@Quantity]";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize table this row");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Table(StructuredTableRef {
            table_name: Some("Table1".into()),
            sections: vec![],
            column: Some("Amount".into()),
            end_column: None,
            is_at_syntax: true,
        }))
    );
    assert_eq!(
        tokens[2].item,
        Token::Reference(Reference::Table(StructuredTableRef {
            table_name: None,
            sections: vec![],
            column: Some("Quantity".into()),
            end_column: None,
            is_at_syntax: true,
        }))
    );
}

#[test]
fn test_17_lexer_dynamic_array_spill_operator_hash() {
    let formula = "=A1# + Sheet1!B5:C10#";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize spill operator");

    assert_eq!(
        tokens[0].item,
        Token::Reference(Reference::Spill(Box::new(Reference::Cell {
            sheet: None,
            cell: CellAddress::parse_a1("A1").unwrap()
        })))
    );
}

#[test]
fn test_18_lexer_dynamic_array_implicit_intersection_at() {
    let formula = "=@A1:A10";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize implicit intersection");

    assert_eq!(tokens[0].item, Token::At);
    assert!(matches!(&tokens[1].item, Token::Reference(_)));
}

#[test]
fn test_19_lexer_intersection_space_operator_between_ranges() {
    let formula = "=A1:C5 B2:D10";
    let mut lexer = Lexer::new(formula);
    let tokens = lexer.tokenize().expect("tokenize intersection space");

    assert!(matches!(&tokens[0].item, Token::Reference(_)));
    assert_eq!(tokens[1].item, Token::Intersection);
    assert!(matches!(&tokens[2].item, Token::Reference(_)));
}

// ==============================================================================
// 2. PRATT OPERATOR PRECEDENCE TESTS
// ==============================================================================

#[test]
fn test_20_pratt_exact_arithmetic_precedence() {
    // 1 + 2 * 3 must parse as 1 + (2 * 3)
    let ast = parse_formula("=1 + 2 * 3").expect("parse 1 + 2 * 3");
    match ast {
        AstNode::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*left, AstNode::Literal(Literal::Number(1.0)));
            match *right {
                AstNode::BinaryOp {
                    op: mul_op,
                    left: m_l,
                    right: m_r,
                } => {
                    assert_eq!(mul_op, BinaryOperator::Mul);
                    assert_eq!(*m_l, AstNode::Literal(Literal::Number(2.0)));
                    assert_eq!(*m_r, AstNode::Literal(Literal::Number(3.0)));
                }
                _ => panic!("Expected multiplication on right"),
            }
        }
        _ => panic!("Expected binary addition at root"),
    }

    // (1 + 2) * 3 must parse as (1 + 2) * 3
    let ast2 = parse_formula("=(1 + 2) * 3").expect("parse (1 + 2) * 3");
    assert!(matches!(
        ast2,
        AstNode::BinaryOp {
            op: BinaryOperator::Mul,
            ..
        }
    ));
}

#[test]
fn test_21_pratt_unary_minus_precedence_over_power() {
    // In Excel, -2^2 evaluates as (-2)^2 = 4 (unary - has higher precedence than ^)
    let ast = parse_formula("=-2^2").expect("parse -2^2");
    match ast {
        AstNode::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOperator::Pow);
            match *left {
                AstNode::UnaryOp { op: u_op, expr } => {
                    assert_eq!(u_op, UnaryOperator::Neg);
                    assert_eq!(*expr, AstNode::Literal(Literal::Number(2.0)));
                }
                _ => panic!("Expected unary neg on left of power"),
            }
            assert_eq!(*right, AstNode::Literal(Literal::Number(2.0)));
        }
        _ => panic!("Expected binary power at root"),
    }
}

#[test]
fn test_22_pratt_power_associativity() {
    // In Excel, 2^3^2 is left-associative: (2^3)^2 = 64
    let ast = parse_formula("=2^3^2").expect("parse 2^3^2");
    match ast {
        AstNode::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOperator::Pow);
            assert_eq!(*right, AstNode::Literal(Literal::Number(2.0)));
            match *left {
                AstNode::BinaryOp {
                    op: inner_op,
                    left: l2,
                    right: r2,
                } => {
                    assert_eq!(inner_op, BinaryOperator::Pow);
                    assert_eq!(*l2, AstNode::Literal(Literal::Number(2.0)));
                    assert_eq!(*r2, AstNode::Literal(Literal::Number(3.0)));
                }
                _ => panic!("Expected left-nested power"),
            }
        }
        _ => panic!("Expected binary power at root"),
    }
}

#[test]
fn test_23_pratt_percent_postfix_operator() {
    let ast = parse_formula("=10 + 20% * 5").expect("parse percent");
    match ast {
        AstNode::BinaryOp { op, right, .. } => {
            assert_eq!(op, BinaryOperator::Add);
            match *right {
                AstNode::BinaryOp {
                    op: mul_op,
                    left: p_left,
                    ..
                } => {
                    assert_eq!(mul_op, BinaryOperator::Mul);
                    assert!(matches!(
                        *p_left,
                        AstNode::UnaryOp {
                            op: UnaryOperator::Percent,
                            ..
                        }
                    ));
                }
                _ => panic!("Expected mul on right"),
            }
        }
        _ => panic!("Expected add at root"),
    }
}

#[test]
fn test_24_pratt_concatenation_and_comparisons() {
    let ast = parse_formula("=\"Hello \" & \"World\" = \"Hello World\"").expect("parse concat");
    match ast {
        AstNode::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOperator::Equal);
            assert!(matches!(
                *left,
                AstNode::BinaryOp {
                    op: BinaryOperator::Concat,
                    ..
                }
            ));
            assert_eq!(
                *right,
                AstNode::Literal(Literal::String("Hello World".into()))
            );
        }
        _ => panic!("Expected comparison at root"),
    }
}

#[test]
fn test_25_pratt_comparison_chaining() {
    let ast = parse_formula("=A1 = B1").expect("parse comparison");
    assert!(matches!(
        ast,
        AstNode::BinaryOp {
            op: BinaryOperator::Equal,
            ..
        }
    ));
}

#[test]
fn test_26_pratt_union_comma_in_parentheses() {
    let ast = parse_formula("=(A1:B10, C1:D10)").expect("parse union");
    match ast {
        AstNode::Parenthesized(inner) => match *inner {
            AstNode::BinaryOp { op, .. } => assert_eq!(op, BinaryOperator::Union),
            _ => panic!("Expected union inside parentheses"),
        },
        _ => panic!("Expected parenthesized union"),
    }
}

#[test]
fn test_27_pratt_intersection_space_precedence() {
    let ast = parse_formula("=A1:C5 B2:D10").expect("parse intersection");
    assert!(matches!(
        ast,
        AstNode::BinaryOp {
            op: BinaryOperator::Intersection,
            ..
        }
    ));
}

// ==============================================================================
// 3. FUNCTION SYNTAX & MATRIX LITERAL TESTS
// ==============================================================================

#[test]
fn test_28_function_single_argument() {
    let ast = parse_formula("=SUM(A1:A10)").expect("parse sum");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_29_function_multiple_arguments() {
    let ast = parse_formula("=AVERAGE(A1, B1, 42)").expect("parse average");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "AVERAGE");
            assert_eq!(args.len(), 3);
            assert_eq!(args[2], AstNode::Literal(Literal::Number(42.0)));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_30_function_empty_omitted_arguments() {
    let ast = parse_formula("=IF(A1>0, 1, )").expect("parse if empty arg");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "IF");
            assert_eq!(args.len(), 3);
            assert_eq!(args[2], AstNode::Literal(Literal::Empty));
        }
        _ => panic!("Expected function call"),
    }

    let ast2 = parse_formula("=OFFSET(A1, 1, , 2, 2)").expect("parse offset empty col");
    match ast2 {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "OFFSET");
            assert_eq!(args.len(), 5);
            assert_eq!(args[2], AstNode::Literal(Literal::Empty));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_31_function_nested_calls() {
    let ast = parse_formula("=INDEX(A1:C10, MATCH(\"target\", A1:A10, 0), 2)").expect("nested");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "INDEX");
            assert_eq!(args.len(), 3);
            assert!(matches!(&args[1], AstNode::FunctionCall { name, .. } if name == "MATCH"));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_32_function_openxml_prefixed_names() {
    let ast = parse_formula("=_xlfn.XLOOKUP(A1, B1:B10, C1:C10)").expect("openxml func");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "_xlfn.XLOOKUP");
            assert_eq!(args.len(), 3);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_33_function_dot_named_functions() {
    let ast = parse_formula("=STDEV.S(A1:A50)").expect("dot named func");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "STDEV.S");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_34_array_literal_1d_row() {
    let ast = parse_formula("={1, 2, 3}").expect("array 1d");
    match ast {
        AstNode::ArrayLiteral { rows } => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 3);
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_35_array_literal_2d_matrix() {
    let ast = parse_formula("={1, 2; 3, 4}").expect("array 2d");
    match ast {
        AstNode::ArrayLiteral { rows } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 2);
        }
        _ => panic!("Expected array literal"),
    }
}

#[test]
fn test_36_array_literal_mixed_types() {
    let ast = parse_formula("={\"Apple\", 10; \"Banana\", 20}").expect("array mixed");
    match ast {
        AstNode::ArrayLiteral { rows } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(
                rows[0][0],
                AstNode::Literal(Literal::String("Apple".into()))
            );
            assert_eq!(rows[0][1], AstNode::Literal(Literal::Number(10.0)));
        }
        _ => panic!("Expected array literal"),
    }
}

// ==============================================================================
// 4. COMPLEX FORMULA SYNTAX TREES
// ==============================================================================

#[test]
fn test_37_complex_formula_sumifs_multi_criteria() {
    let formula = "=SUMIFS(C1:C100, A1:A100, \">10\", B1:B100, \"East\")";
    let ast = parse_formula(formula).expect("sumifs");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "SUMIFS");
            assert_eq!(args.len(), 5);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_38_complex_formula_xlookup_with_spill() {
    let formula = "=_xlfn.XLOOKUP(E1#, A1:A100, B1:B100)";
    let ast = parse_formula(formula).expect("xlookup spill");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "_xlfn.XLOOKUP");
            assert!(matches!(&args[0], AstNode::Reference(Reference::Spill(_))));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_39_complex_formula_let_lambda() {
    let formula = "=_xlfn.LET(x, 10, y, 20, x * y)";
    let ast = parse_formula(formula).expect("let");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "_xlfn.LET");
            assert_eq!(args.len(), 5);
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_40_complex_formula_structured_table_aggregation() {
    let formula = "=SUM(DeptSales[[#Data],[Sales]])";
    let ast = parse_formula(formula).expect("table sum");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "SUM");
            assert!(matches!(&args[0], AstNode::Reference(Reference::Table(_))));
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_41_complex_formula_3d_sum_across_worksheets() {
    let formula = "=SUM(Jan:Dec!B5:B10)";
    let ast = parse_formula(formula).expect("3d sum");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "SUM");
            match &args[0] {
                AstNode::Reference(Reference::Range { sheet, .. }) => {
                    assert_eq!(*sheet, Some(SheetRef::Span("Jan".into(), "Dec".into())));
                }
                _ => panic!("Expected 3D range reference"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

#[test]
fn test_42_complex_formula_dynamic_filter_spill() {
    let formula = "=FILTER(A1:C50, B1:B50=\"Active\")#";
    let ast = parse_formula(formula).expect("filter spill");
    match ast {
        AstNode::UnaryOp { op, expr } => {
            assert_eq!(op, UnaryOperator::Spill);
            assert!(matches!(*expr, AstNode::FunctionCall { .. }));
        }
        _ => panic!("Expected spill unary operator"),
    }
}

#[test]
fn test_43_complex_formula_intersection_in_areas() {
    let formula = "=AREAS((A1:C5 B2:D6, E1:G5))";
    let ast = parse_formula(formula).expect("areas");
    match ast {
        AstNode::FunctionCall { name, args } => {
            assert_eq!(name, "AREAS");
            assert_eq!(args.len(), 1);
            match &args[0] {
                AstNode::Parenthesized(inner) => match &**inner {
                    AstNode::BinaryOp { op, left, .. } => {
                        assert_eq!(*op, BinaryOperator::Union);
                        assert!(matches!(
                            &**left,
                            AstNode::BinaryOp {
                                op: BinaryOperator::Intersection,
                                ..
                            }
                        ));
                    }
                    _ => panic!("Expected union"),
                },
                _ => panic!("Expected parenthesized union"),
            }
        }
        _ => panic!("Expected function call"),
    }
}

// ==============================================================================
// 5. BYTECODE & DECOMPILATION TESTS
// ==============================================================================

#[test]
fn test_44_bytecode_compilation_literals_and_ops() {
    let formula = "=1 + 2 * 3";
    let program = compile_formula(formula).expect("compile formula");
    assert_eq!(program.instructions.len(), 5);
    assert_eq!(program.instructions[0], OpCode::PushNumber(1.0));
    assert_eq!(program.instructions[1], OpCode::PushNumber(2.0));
    assert_eq!(program.instructions[2], OpCode::PushNumber(3.0));
    assert_eq!(
        program.instructions[3],
        OpCode::BinaryOp(BinaryOperator::Mul)
    );
    assert_eq!(
        program.instructions[4],
        OpCode::BinaryOp(BinaryOperator::Add)
    );
}

#[test]
fn test_45_bytecode_compilation_nested_functions() {
    let formula = "=SUM(A1:A10, AVERAGE(B1:B10))";
    let program = compile_formula(formula).expect("compile nested funcs");
    assert!(program.instructions.contains(&OpCode::CallFunction {
        name: "AVERAGE".into(),
        arg_count: 1,
    }));
    assert!(program.instructions.contains(&OpCode::CallFunction {
        name: "SUM".into(),
        arg_count: 2,
    }));
}

#[test]
fn test_46_bytecode_serialization_round_trip() {
    let formula = "=SUM(A1:B10) + 42";
    let program = compile_formula(formula).expect("compile");
    let bytes = program.to_bytes().expect("serialize bytecode");
    assert!(!bytes.is_empty());

    let restored = BytecodeProgram::from_bytes(&bytes).expect("deserialize bytecode");
    assert_eq!(program, restored);
}

#[test]
fn test_47_decompiler_ast_to_formula_string() {
    let formula = "=(1 + 2) * 3";
    let ast = parse_formula(formula).expect("parse");
    let decompiled = decompile_ast(&ast);
    assert_eq!(decompiled, "=(1 + 2) * 3");
}

#[test]
fn test_48_decompiler_bytecode_stack_simulation() {
    let formula = "=SUM(A1:A10, 42)";
    let program = compile_formula(formula).expect("compile");
    let decompiled = decompile_bytecode(&program).expect("decompile");
    assert_eq!(decompiled, "=SUM(A1:A10,42)");
}

#[test]
fn test_49_round_trip_parse_compile_decompile_equivalence() {
    let test_cases = [
        "=42",
        "=A1 + B1",
        "=SUM(A1:A10)",
        "=(A1 + B1) * C1",
        "=Sheet1!A1:B10",
        "=-2^2",
        "={1,2;3,4}",
    ];

    for case in &test_cases {
        let program = compile_formula(case).expect("compile");
        let decompiled = decompile_formula(&program).expect("decompile");
        let program2 = compile_formula(&decompiled).expect("recompile");
        assert_eq!(program, program2, "Equivalence failure on: {case}");
    }
}

// ==============================================================================
// 6. ROBUSTNESS & CSPG GRID INTEGRATION TESTS
// ==============================================================================

#[test]
fn test_50_zero_panic_error_handling_malformed_formulas() {
    let malformed = [
        "=(1 + ",
        "=SUM(",
        "=\"unterminated string",
        "='unclosed sheet!A1",
        "=Table1[[unclosed bracket",
        "={1, 2; 3",
    ];

    for bad in &malformed {
        let result = parse_formula(bad);
        assert!(result.is_err(), "Expected error for malformed: {bad}");
    }
}

#[test]
fn test_51_grid_store_and_retrieve_formula_bytecode() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_formula(0, 0, "=SUM(A1:A10) + 5", Some(55.0))
        .unwrap();

    let formula_str = grid.get_formula_str(0, 0).expect("formula str");
    assert_eq!(formula_str, "=SUM(A1:A10) + 5");

    let bytecode = grid.get_formula_bytecode(0, 0).expect("formula bytecode");
    assert!(bytecode.instructions.contains(&OpCode::CallFunction {
        name: "SUM".into(),
        arg_count: 1,
    }));
}

#[test]
fn test_52_vectors_manifest_validation() {
    let path = resolve_fixture("specs/evidence/excel/2.2.1_vectors.json");
    assert!(path.exists(), "Vectors manifest must exist at {:?}", path);

    let content = fs::read_to_string(path).expect("read vectors manifest");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse json");

    let total = json["total_vectors"].as_u64().unwrap_or(0);
    assert_eq!(total, 52);

    let vectors = json["vectors"].as_array().expect("vectors array");
    assert_eq!(vectors.len(), 52);

    for v in vectors {
        let f = v["formula"].as_str().expect("formula string");
        let parsed = parse_formula(f);
        assert!(
            parsed.is_ok(),
            "Vector formula failed to parse: {f}: {:?}",
            parsed.err()
        );
    }
}
