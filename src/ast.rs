/// Вся программа – набор функций
#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// Одна функция: имя и список операторов тела
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

/// Операторы
#[derive(Debug)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    IoPrint(Expr),
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
}

/// Выражения
#[derive(Debug)]
pub enum Expr {
    Number(i32),
    StringLiteral(String),
    Variable(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,          // ==
    NotEq,       // !=
    Less,
    Greater,
    LessEq,      // <=
    GreaterEq,   // >=
}