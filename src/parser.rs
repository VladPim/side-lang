use crate::ast::*;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, std::ops::Range<usize>)>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = vec![];
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.peek() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Syntax error: expected {:?}, but found {:?}",
                expected,
                self.peek()
            ))
        }
    }

    // --- Разбор функции ---
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Fn)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected function name after 'fn'".to_string());
        };

        // Тело функции в фигурных скобках
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        // RBrace уже съедена в parse_block
        Ok(Function { name, body })
    }

    /// parse_block: считывает стейтменты пока не встретит '}'
    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    // --- Операторы ---
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Io) => self.parse_io_print(),
            other => Err(format!("Unexpected token: {:?}", other)),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected variable name after 'let'".to_string());
        };
        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let then_body = self.parse_block()?;
        // Проверяем, есть ли else
        let else_body = if self.peek() == Some(&Token::Else) {
            self.advance();
            self.expect(Token::LBrace)?;
            let else_stmts = self.parse_block()?;
            Some(else_stmts)
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_io_print(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Io)?;
        self.expect(Token::Dot)?;
        self.expect(Token::Print)?;
        self.expect(Token::LParen)?;
        let expr = self.parse_expr()?;
        self.expect(Token::RParen)?;
        Ok(Stmt::IoPrint(expr))
    }

    // --- Выражения (с добавлением операторов сравнения) ---
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_comparison()
    }

    // Уровень сравнения (ниже арифметики)
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        while let Some(op) = self.peek() {
            let binop = match op {
                Token::EqualEqual => BinOp::Eq,
                Token::NotEqual => BinOp::NotEq,
                Token::Less => BinOp::Less,
                Token::Greater => BinOp::Greater,
                Token::LessEqual => BinOp::LessEq,
                Token::GreaterEqual => BinOp::GreaterEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: binop,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    // Арифметика сложения
    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        while let Some(op) = self.peek() {
            match op {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Add,
                        right: Box::new(right),
                    };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Sub,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // Арифметика умножения
    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        while let Some(op) = self.peek() {
            match op {
                Token::Star => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Mul,
                        right: Box::new(right),
                    };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_primary()?;
                    left = Expr::Binary {
                        left: Box::new(left),
                        op: BinOp::Div,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Some(Token::StringLiteral(s)) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            Some(Token::Identifier(name)) => {
                self.advance();
                Ok(Expr::Variable(name))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            other => Err(format!("Unexpected token in expression: {:?}", other)),
        }
    }
}