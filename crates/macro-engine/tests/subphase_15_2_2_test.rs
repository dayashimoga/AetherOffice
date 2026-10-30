use excel_engine::formula::ast::{AstNode, Literal};
use excel_engine::functions::{dispatch_function, EvalContext};
use excel_engine::grid::{CellError, CellValue};
use macro_engine::{
    cell_value_to_vba_value, create_vba_udf_handler, vba_value_to_cell_value, VbaParser, VbaValue,
};

#[test]
fn test_01_udf_bridge_scalar_number_roundtrip() {
    let cv = CellValue::Number(123.45);
    let vv = cell_value_to_vba_value(&cv);
    assert_eq!(vv, VbaValue::Double(123.45));

    let back = vba_value_to_cell_value(&vv);
    assert_eq!(back, CellValue::Number(123.45));
}

#[test]
fn test_02_udf_bridge_string_and_bool_conversions() {
    let cv_str = CellValue::InlineString("Revenue".to_string());
    let vv_str = cell_value_to_vba_value(&cv_str);
    assert_eq!(vv_str, VbaValue::String("Revenue".to_string()));

    let cv_bool = CellValue::Boolean(true);
    let vv_bool = cell_value_to_vba_value(&cv_bool);
    assert_eq!(vv_bool, VbaValue::Boolean(true));
}

#[test]
fn test_03_create_vba_udf_handler_function_evaluation() {
    let source = "
    Function CalculateTax(amt As Double, rate As Double) As Double
        CalculateTax = amt * rate
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let handler = create_vba_udf_handler(program, 10_000);

    let args = vec![CellValue::Number(200.0), CellValue::Number(0.15)];
    let res = handler("CalculateTax", &args).unwrap();
    assert_eq!(res, CellValue::Number(30.0));
}

#[test]
fn test_04_create_vba_udf_handler_string_manipulation() {
    let source = "
    Function FormatCurrency(symbol As String, amt As Double) As String
        FormatCurrency = symbol & amt
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let handler = create_vba_udf_handler(program, 10_000);

    let args = vec![
        CellValue::InlineString("$".to_string()),
        CellValue::Number(500.0),
    ];
    let res = handler("FormatCurrency", &args).unwrap();
    assert_eq!(res, CellValue::InlineString("$500".to_string()));
}

#[test]
fn test_05_udf_handler_case_insensitive_dispatch() {
    let source = "
    Function DoubleVal(x As Double) As Double
        DoubleVal = x * 2
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let handler = create_vba_udf_handler(program, 10_000);

    let args = vec![CellValue::Number(21.0)];
    assert_eq!(
        handler("doubleval", &args).unwrap(),
        CellValue::Number(42.0)
    );
    assert_eq!(
        handler("DOUBLEVAL", &args).unwrap(),
        CellValue::Number(42.0)
    );
    assert_eq!(
        handler("DoubleVal", &args).unwrap(),
        CellValue::Number(42.0)
    );
}

#[test]
fn test_06_udf_handler_unknown_function_fallback() {
    let source = "
    Function KnownFunc() As Integer
        KnownFunc = 1
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let handler = create_vba_udf_handler(program, 10_000);
    let res = handler("NonExistentFunc", &[]).unwrap();
    assert_eq!(res, CellValue::Error(CellError::Value));
}

#[test]
fn test_07_udf_handler_respects_gas_limit() {
    let source = "
    Function InfiniteSpin() As Double
        Dim x As Double
        Do While True
            x = x + 1
        Loop
        InfiniteSpin = x
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();

    let handler = create_vba_udf_handler(program, 300);
    let res = handler("InfiniteSpin", &[]).unwrap();
    // Gas exhausted safely yields Value error
    assert_eq!(res, CellValue::Error(CellError::Value));
}

#[test]
fn test_08_excel_engine_eval_context_with_udf_handler() {
    let source = "
    Function AddFive(x As Double) As Double
        AddFive = x + 5
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let handler = create_vba_udf_handler(program, 10_000);

    let ctx = EvalContext::new(None, false).with_udf_handler(handler);
    assert!(ctx.udf_handler.is_some());
}

#[test]
fn test_09_excel_engine_dispatch_function_invokes_udf() {
    let source = "
    Function CalculateCommission(revenue As Double, rate As Double) As Double
        CalculateCommission = revenue * rate
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let handler = create_vba_udf_handler(program, 10_000);

    let ctx = EvalContext::new(None, false).with_udf_handler(handler);

    // Formulate call to CALCULATECOMMISSION(1000, 0.1)
    let args = vec![
        AstNode::Literal(Literal::Number(1000.0)),
        AstNode::Literal(Literal::Number(0.1)),
    ];

    let eval_fn = |node: &AstNode, _c: &EvalContext| -> ooxml_opc::error::Result<CellValue> {
        match node {
            AstNode::Literal(Literal::Number(n)) => Ok(CellValue::Number(*n)),
            _ => Ok(CellValue::Empty),
        }
    };

    let result = dispatch_function("CALCULATECOMMISSION", &args, &ctx, &eval_fn).unwrap();
    assert_eq!(result, CellValue::Number(100.0));
}

#[test]
fn test_10_excel_engine_dispatch_function_built_in_takes_precedence() {
    let source = "
    Function SUM(a As Double, b As Double) As Double
        SUM = 9999
    End Function
    ";

    let mut parser = VbaParser::from_source(source).unwrap();
    let program = parser.parse_program().unwrap();
    let handler = create_vba_udf_handler(program, 10_000);

    let ctx = EvalContext::new(None, false).with_udf_handler(handler);

    let args = vec![
        AstNode::Literal(Literal::Number(10.0)),
        AstNode::Literal(Literal::Number(20.0)),
    ];

    let eval_fn = |node: &AstNode, _c: &EvalContext| -> ooxml_opc::error::Result<CellValue> {
        match node {
            AstNode::Literal(Literal::Number(n)) => Ok(CellValue::Number(*n)),
            _ => Ok(CellValue::Empty),
        }
    };

    let result = dispatch_function("SUM", &args, &ctx, &eval_fn).unwrap();
    // Built-in SUM produces 30.0, NOT the overridden 9999
    assert_eq!(result, CellValue::Number(30.0));
}
