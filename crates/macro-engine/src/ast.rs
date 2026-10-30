use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VbaProgram {
    pub routines: Vec<RoutineDef>,
    pub top_level_statements: Vec<VbaStatement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutineKind {
    Sub,
    Function,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VbaParam {
    pub name: String,
    pub param_type: Option<String>,
    pub is_by_val: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutineDef {
    pub kind: RoutineKind,
    pub name: String,
    pub params: Vec<VbaParam>,
    pub return_type: Option<String>,
    pub body: Vec<VbaStatement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitKind {
    Sub,
    Function,
    For,
    Do,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VbaStatement {
    Dim {
        name: String,
        var_type: Option<String>,
    },
    Assign {
        target: VbaExpr,
        value: VbaExpr,
    },
    Set {
        target: VbaExpr,
        value: VbaExpr,
    },
    Call {
        target: VbaExpr,
        args: Vec<VbaExpr>,
    },
    If {
        condition: VbaExpr,
        then_branch: Vec<VbaStatement>,
        else_ifs: Vec<(VbaExpr, Vec<VbaStatement>)>,
        else_branch: Option<Vec<VbaStatement>>,
    },
    For {
        counter: String,
        start: VbaExpr,
        end: VbaExpr,
        step: Option<VbaExpr>,
        body: Vec<VbaStatement>,
    },
    DoLoop {
        condition: Option<VbaExpr>,
        is_while: bool,
        is_post_test: bool,
        body: Vec<VbaStatement>,
    },
    WhileWend {
        condition: VbaExpr,
        body: Vec<VbaStatement>,
    },
    With {
        object: VbaExpr,
        body: Vec<VbaStatement>,
    },
    Exit(ExitKind),
    Expr(VbaExpr),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VbaLiteral {
    Empty,
    Null,
    Nothing,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    IntDiv, // \
    Pow,    // ^
    Concat, // &
    Mod,    // Mod
    Eq,     // =
    NotEq,  // <>
    Lt,     // <
    LtEq,   // <=
    Gt,     // >
    GtEq,   // >=
    And,    // And
    Or,     // Or
    Xor,    // Xor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg, // -
    Not, // Not
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VbaExpr {
    Literal(VbaLiteral),
    Variable(String),
    UnaryOp {
        op: UnaryOp,
        expr: Box<VbaExpr>,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<VbaExpr>,
        right: Box<VbaExpr>,
    },
    MemberAccess {
        object: Box<VbaExpr>,
        member: String,
    },
    IndexOrCall {
        callee: Box<VbaExpr>,
        args: Vec<VbaExpr>,
    },
    ImplicitMember(String), // e.g. `.Value` or `.Offset(1, 0)` inside With block
}
