// ast.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<String>,
    pub structs: Vec<StructDef>,
    pub constants: Vec<Constant>,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub module_name: Option<String>,
    pub struct_name: Option<String>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Double,
    Str,
    Array { elem: Box<Type>, size: Option<usize> },
    Struct(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        var_type: Option<Type>,
        value: Expr,
        span: Span,
    },
    Assign {
        name: String,
        value: Expr,
        span: Span,
    },
    AssignIndex {
        name: String,
        index: Expr,
        value: Expr,
        span: Span,
    },
    IoPrint(Vec<Expr>, Span),
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        var_name: String,
        start: Expr,
        condition: Expr,
        step: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Return(Expr, Span),
    Break(Span),
    Continue(Span),
    CallStmt {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32, Span),
    DoubleLiteral(f64, Span),
    StringLiteral(String, Span),
    Variable(String, Span),
    Input(String, Span),
    Call {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    CallWithTarget {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    StructLiteral {
        name: String,
        fields: Vec<Expr>,
        span: Span,
    },
    ArrayLiteral(Vec<Expr>, Span),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
        span: Span,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, NotEq, Less, Greater, LessEq, GreaterEq,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number(_, span) => *span,
            Expr::DoubleLiteral(_, span) => *span,
            Expr::StringLiteral(_, span) => *span,
            Expr::Variable(_, span) => *span,
            Expr::Input(_, span) => *span,
            Expr::Call { span, .. } => *span,
            Expr::CallWithTarget { span, .. } => *span,
            Expr::StructLiteral { span, .. } => *span,
            Expr::ArrayLiteral(_, span) => *span,
            Expr::Index { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Ternary { span, .. } => *span,
        }
    }
}
