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
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Double,
    Str,
    Array,
    DoubleArray,
    Struct(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        var_type: Option<Type>,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    AssignIndex {
        name: String,
        index: Expr,
        value: Expr,
    },
    IoPrint(Vec<Expr>),
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var_name: String,
        start: Expr,
        condition: Expr,
        step: Expr,
        body: Vec<Stmt>,
    },
    Return(Expr),
    Break,
    Continue,
    CallStmt {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i32),
    DoubleLiteral(f64),
    StringLiteral(String),
    Variable(String),
    Input(String),
    Call {
        name: String,
        args: Vec<Expr>,
    },
    StructLiteral {
        name: String,
        fields: Vec<Expr>,
    },
    ArrayLiteral(Vec<Expr>),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
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
