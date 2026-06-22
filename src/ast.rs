#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug)]
pub enum Type {
    Int,
    Str,
}

#[derive(Debug)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Assign {
        name: String,
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
    Return(Expr),
    Break,
    Continue,
    CallStmt {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug)]
pub enum Expr {
    Number(i32),
    StringLiteral(String),
    Variable(String),
    Input(String),
    Call {
        name: String,
        args: Vec<Expr>,
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

#[derive(Debug)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, NotEq, Less, Greater, LessEq, GreaterEq,
    And, Or,
}

#[derive(Debug)]
pub enum UnaryOp {
    Not,
    Neg,
}
