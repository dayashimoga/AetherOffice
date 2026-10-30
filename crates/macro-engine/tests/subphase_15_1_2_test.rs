use macro_engine::{
    BinaryOp, RangeAddress, RoutineKind, TokenKind, VbaEvaluator, VbaExpr, VbaLexer, VbaLiteral,
    VbaParser, VbaStatement, VbaValue, VirtualOfficeDom,
};

#[test]
fn test_01_vba_lexer_tokenization() {
    let source = "
    ' This is a comment
    Sub Main()
        Dim x As Integer
        x = &H1A + 42
        Dim s As String
        s = \"Hello \"\"World\"\"\"
        Dim y As Double
        y = 12.75 _
            + 0.5
    End Sub
    ";

    let mut lexer = VbaLexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
    assert!(kinds.contains(&TokenKind::Sub));
    assert!(kinds.contains(&TokenKind::Dim));
    assert!(kinds.contains(&TokenKind::As));
    assert!(kinds.contains(&TokenKind::Int(0x1A))); // 26 from &H1A
    assert!(kinds.contains(&TokenKind::Int(42)));
    assert!(kinds.contains(&TokenKind::Str("Hello \"World\"".to_string())));
    assert!(kinds.contains(&TokenKind::Float(12.75)));
    assert!(kinds.contains(&TokenKind::Float(0.5)));
    assert!(kinds.contains(&TokenKind::End));
}

#[test]
fn test_02_pratt_expression_precedence() {
    // 1 + 2 * 3
    let mut parser = VbaParser::from_source("1 + 2 * 3").unwrap();
    let expr = parser.parse_expression().unwrap();

    match expr {
        VbaExpr::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOp::Add);
            assert_eq!(*left, VbaExpr::Literal(VbaLiteral::Int(1)));
            match *right {
                VbaExpr::BinaryOp {
                    op: right_op,
                    left: r_left,
                    right: r_right,
                } => {
                    assert_eq!(right_op, BinaryOp::Mul);
                    assert_eq!(*r_left, VbaExpr::Literal(VbaLiteral::Int(2)));
                    assert_eq!(*r_right, VbaExpr::Literal(VbaLiteral::Int(3)));
                }
                other => panic!("Expected multiplication on right, found {other:?}"),
            }
        }
        other => panic!("Expected binary add, found {other:?}"),
    }

    // 2 ^ 3 ^ 2 (right associative)
    let mut parser = VbaParser::from_source("2 ^ 3 ^ 2").unwrap();
    let pow_expr = parser.parse_expression().unwrap();
    match pow_expr {
        VbaExpr::BinaryOp { op, left, right } => {
            assert_eq!(op, BinaryOp::Pow);
            assert_eq!(*left, VbaExpr::Literal(VbaLiteral::Int(2)));
            match *right {
                VbaExpr::BinaryOp { op: r_op, .. } => assert_eq!(r_op, BinaryOp::Pow),
                other => panic!("Expected right associative power, found {other:?}"),
            }
        }
        other => panic!("Expected power op, found {other:?}"),
    }
}

#[test]
fn test_03_routine_parsing_sub_and_function() {
    let source = "
    Public Sub CalculateTotal(ByVal count As Integer, Optional mode As String)
        Dim res As Double
    End Sub

    Function ComputeTax(amount As Double, rate As Double) As Double
        ComputeTax = amount * rate
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    assert_eq!(program.routines.len(), 2);

    let sub = &program.routines[0];
    assert_eq!(sub.kind, RoutineKind::Sub);
    assert_eq!(sub.name, "CalculateTotal");
    assert_eq!(sub.params.len(), 2);
    assert!(sub.params[0].is_by_val);
    assert_eq!(sub.params[0].name, "count");
    assert_eq!(sub.params[0].param_type, Some("Integer".to_string()));
    assert!(sub.params[1].is_optional);

    let func = &program.routines[1];
    assert_eq!(func.kind, RoutineKind::Function);
    assert_eq!(func.name, "ComputeTax");
    assert_eq!(func.return_type, Some("Double".to_string()));
    assert_eq!(func.params.len(), 2);
}

#[test]
fn test_04_control_flow_if_elseif_else() {
    let source = "
    Sub CheckScore()
        If score > 90 Then
            grade = \"A\"
        ElseIf score > 75 Then
            grade = \"B\"
        Else
            grade = \"C\"
        End If
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let sub = &program.routines[0];

    assert_eq!(sub.body.len(), 1);
    match &sub.body[0] {
        VbaStatement::If {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        } => {
            assert!(matches!(condition, VbaExpr::BinaryOp { .. }));
            assert_eq!(then_branch.len(), 1);
            assert_eq!(else_ifs.len(), 1);
            assert!(else_branch.is_some());
        }
        other => panic!("Expected If statement, found {other:?}"),
    }
}

#[test]
fn test_05_control_flow_for_next_and_do_loop() {
    let source = "
    Sub LoopTesting()
        For i = 1 To 10 Step 2
            x = x + i
        Next i

        Do While x < 100
            x = x * 2
        Loop

        While x > 50
            x = x - 10
        Wend
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let sub = &program.routines[0];

    assert_eq!(sub.body.len(), 3);
    match &sub.body[0] {
        VbaStatement::For {
            counter,
            start,
            end,
            step,
            body,
        } => {
            assert_eq!(counter, "i");
            assert_eq!(*start, VbaExpr::Literal(VbaLiteral::Int(1)));
            assert_eq!(*end, VbaExpr::Literal(VbaLiteral::Int(10)));
            assert_eq!(*step, Some(VbaExpr::Literal(VbaLiteral::Int(2))));
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected For loop, found {other:?}"),
    }

    assert!(matches!(&sub.body[1], VbaStatement::DoLoop { .. }));
    assert!(matches!(&sub.body[2], VbaStatement::WhileWend { .. }));
}

#[test]
fn test_06_with_block_parsing() {
    let source = "
    Sub StyleRange()
        With Range(\"A1:B5\")
            .Value = 100
            .Color = 255
        End With
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let sub = &program.routines[0];

    assert_eq!(sub.body.len(), 1);
    match &sub.body[0] {
        VbaStatement::With { object, body } => {
            assert!(matches!(object, VbaExpr::IndexOrCall { .. }));
            assert_eq!(body.len(), 2);
            assert!(matches!(
                &body[0],
                VbaStatement::Assign {
                    target: VbaExpr::ImplicitMember(m),
                    ..
                } if m == "Value"
            ));
        }
        other => panic!("Expected With block, found {other:?}"),
    }
}

#[test]
fn test_07_virtual_dom_ranges_and_cells() {
    let mut dom = VirtualOfficeDom::new();
    let sheet = dom.active_sheet_mut().unwrap();

    let addr = RangeAddress::from_a1("B2:D5").unwrap();
    assert_eq!(addr.start_row, 2);
    assert_eq!(addr.start_col, 2); // Col B = 2
    assert_eq!(addr.end_row, 5);
    assert_eq!(addr.end_col, 4); // Col D = 4

    let offset = addr.offset(1, 2).unwrap();
    assert_eq!(offset.start_row, 3);
    assert_eq!(offset.start_col, 4);

    sheet.set_range_value(addr, VbaValue::Integer(77));
    assert_eq!(sheet.get_value(2, 2), VbaValue::Integer(77));
    assert_eq!(sheet.get_value(5, 4), VbaValue::Integer(77));

    sheet.clear_range(addr);
    assert_eq!(sheet.get_value(2, 2), VbaValue::Empty);
}

#[test]
fn test_08_evaluator_range_mutations_and_with_block() {
    let source = "
    Sub FormatHeader()
        With Range(\"A1\")
            .Value = \"Revenue\"
            .Color = 65535
        End With
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let mut evaluator = VbaEvaluator::new(&mut dom);
    evaluator
        .execute_routine(&program, "FormatHeader", &[])
        .unwrap();

    let sheet = dom.active_sheet().unwrap();
    assert_eq!(
        sheet.get_value(1, 1),
        VbaValue::String("Revenue".to_string())
    );
    let cell = sheet.get_cell(1, 1);
    assert_eq!(cell.interior_color, Some(65535));
}

#[test]
fn test_09_evaluator_loop_and_cell_computations() {
    let source = "
    Sub GenerateSeries()
        Dim i As Integer
        For i = 1 To 5
            Cells(i, 1) = i * 10
        Next i
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let mut evaluator = VbaEvaluator::new(&mut dom);
    evaluator
        .execute_routine(&program, "GenerateSeries", &[])
        .unwrap();

    let sheet = dom.active_sheet().unwrap();
    for i in 1..=5 {
        assert_eq!(sheet.get_value(i, 1), VbaValue::Integer((i * 10) as i64));
    }
}

#[test]
fn test_10_evaluator_function_execution_and_range_copy() {
    let source = "
    Function DoubleIt(val As Double) As Double
        DoubleIt = val * 2
    End Function

    Sub CopyCell()
        Range(\"A1\").Value = 999
        Range(\"A1\").Copy Range(\"B1\")
    End Sub
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let mut dom = VirtualOfficeDom::new();
    let mut evaluator = VbaEvaluator::new(&mut dom);

    // Test function evaluation
    let res = evaluator
        .execute_routine(&program, "DoubleIt", &[VbaValue::Double(21.0)])
        .unwrap();
    assert_eq!(res, VbaValue::Double(42.0));

    // Test range copy execution
    evaluator
        .execute_routine(&program, "CopyCell", &[])
        .unwrap();

    let sheet = dom.active_sheet().unwrap();
    assert_eq!(sheet.get_value(1, 1), VbaValue::Integer(999));
    assert_eq!(sheet.get_value(1, 2), VbaValue::Integer(999)); // Col B is 2
}
