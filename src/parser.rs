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
                "Syntax error: expected {:?}, got {:?}",
                expected,
                self.peek()
            ))
        }
    }

    // -------- Функции ----------
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Fn)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected function name after 'fn'".to_string());
        };
        self.expect(Token::LParen)?;
        let mut params = vec![];
        if self.peek() != Some(&Token::RParen) {
            loop {
                let param_type = match self.peek() {
                    Some(Token::Int) => { self.advance(); Type::Int }
                    Some(Token::Str) => { self.advance(); Type::Str }
                    _ => Type::Int,
                };
                let param_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                    self.advance();
                    n
                } else {
                    return Err("Expected parameter name".to_string());
                };
                params.push(Param { name: param_name, param_type });
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Function { name, params, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    // -------- Операторы ----------
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => { self.advance(); Ok(Stmt::Break) }
            Some(Token::Continue) => { self.advance(); Ok(Stmt::Continue) }
            Some(Token::Io) => self.parse_io_print(),
            Some(Token::Identifier(_)) => self.parse_assign_or_call_stmt(),
            other => Err(format!("Unexpected token at statement start: {:?}", other)),
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

    /// Разбор присваивания, присваивания с индексом или вызова функции
    fn parse_assign_or_call_stmt(&mut self) -> Result<Stmt, String> {
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected identifier".to_string());
        };

        // Проверяем индексацию: name[ ... ]
        if self.peek() == Some(&Token::LBracket) {
            self.expect(Token::LBracket)?;
            let index = self.parse_expr()?;
            self.expect(Token::RBracket)?;
            self.expect(Token::Equals)?;
            let value = self.parse_expr()?;
            return Ok(Stmt::AssignIndex { name, index, value });
        }

        // Вызов функции?
        if self.peek() == Some(&Token::LParen) {
            let args = self.parse_call_args()?;
            if self.peek() == Some(&Token::Equals) {
                // присваивание результата вызова: name() = expr
                self.expect(Token::Equals)?;
                let value = Expr::Call { name: name.clone(), args };
                Ok(Stmt::Assign { name: name.clone(), value })
            } else {
                // вызов как стейтмент
                Ok(Stmt::CallStmt { name, args })
            }
        } else {
            // Обычное присваивание
            self.expect(Token::Equals)?;
            let value = self.parse_expr()?;
            Ok(Stmt::Assign { name, value })
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, String> {
        self.expect(Token::LParen)?;
        let mut args = vec![];
        if self.peek() != Some(&Token::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        Ok(args)
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let then_body = self.parse_block()?;
        let else_body = if self.peek() == Some(&Token::Else) {
            self.advance();
            if self.peek() == Some(&Token::If) {
                let else_if_stmt = self.parse_if()?;
                Some(vec![else_if_stmt])
            } else {
                self.expect(Token::LBrace)?;
                let else_stmts = self.parse_block()?;
                Some(else_stmts)
            }
        } else {
            None
        };
        Ok(Stmt::If { condition, then_body, else_body })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(Token::While)?;
        let condition = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Return)?;
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(expr))
    }

    fn parse_io_print(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Io)?;
        self.expect(Token::Dot)?;
        self.expect(Token::Print)?;
        self.expect(Token::LParen)?;
        let mut args = vec![];
        if self.peek() != Some(&Token::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;
        Ok(Stmt::IoPrint(args))
    }

    // -------- Выражения ----------
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary { left: Box::new(left), op: BinOp::And, right: Box::new(right) };
        }
        Ok(left)
    }

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
            left = Expr::Binary { left: Box::new(left), op: binop, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        while let Some(op) = self.peek() {
            match op {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Add, right: Box::new(right) };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Sub, right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Some(op) = self.peek() {
            match op {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Mul, right: Box::new(right) };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Div, right: Box::new(right) };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.peek() == Some(&Token::Not) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr) });
        }
        if self.peek() == Some(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr) });
        }
        self.parse_postfix()
    }

    /// После первичного выражения обрабатываем постфиксные операции [индекс]
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        while self.peek() == Some(&Token::LBracket) {
            self.advance();
            let index = self.parse_expr()?;
            self.expect(Token::RBracket)?;
            expr = Expr::Index {
                array: Box::new(expr),
                index: Box::new(index),
            };
        }
        Ok(expr)
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
                // Вызов функции?
                if self.peek() == Some(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::LBracket) => {
                // Литерал массива
                self.advance();
                let mut elements = vec![];
                if self.peek() != Some(&Token::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            }
            Some(Token::Io) => {
                self.advance();
                self.expect(Token::Dot)?;
                self.expect(Token::Input)?;
                self.expect(Token::LParen)?;
                let prompt = if let Some(Token::StringLiteral(s)) = self.peek().cloned() {
                    self.advance();
                    s
                } else {
                    return Err("Expected prompt string for io.input".to_string());
                };
                self.expect(Token::RParen)?;
                Ok(Expr::Input(prompt))
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
