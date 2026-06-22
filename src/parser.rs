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
    let mut imports = vec![];
    let mut structs = vec![];
    let mut constants = vec![];
    let mut functions = vec![];

    while self.peek().is_some() {
        match self.peek() {
            Some(Token::Import) => imports.push(self.parse_import()?),
            Some(Token::Struct) => structs.push(self.parse_struct_def()?),
            Some(Token::Fn) => functions.push(self.parse_function()?),
            Some(Token::Let) => constants.push(self.parse_constant()?),
            other => return Err(format!("Unexpected top-level token: {:?}", other)),
        }
    }
    Ok(Program { imports, structs, constants, functions })
}

fn parse_import(&mut self) -> Result<String, String> {
    self.expect(Token::Import)?;
    if let Some(Token::StringLiteral(path)) = self.peek().cloned() {
        self.advance();
        Ok(path)
    } else {
        Err("Expected string literal after 'import'".to_string())
    }
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
                "Syntax error: expected {:?}, got {:?} at pos {}",
                expected,
                self.peek(),
                self.pos
            ))
        }
    }

    // -------- Константы ----------
    fn parse_constant(&mut self) -> Result<Constant, String> {
        self.expect(Token::Let)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected constant name after 'let'".to_string());
        };
        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        Ok(Constant { name, value })
    }

    // -------- Структуры ----------
    fn parse_struct_def(&mut self) -> Result<StructDef, String> {
        self.expect(Token::Struct)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected struct name".to_string());
        };
        self.expect(Token::LBrace)?;
        let mut fields = vec![];
        while self.peek() != Some(&Token::RBrace) {
            let field_type = self.parse_type()?;
            let field_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                self.advance();
                n
            } else {
                return Err("Expected field name".to_string());
            };
            fields.push(Field { name: field_name, field_type });
        }
        self.expect(Token::RBrace)?;
        Ok(StructDef { name, fields })
    }

    // -------- Функции ----------
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Fn)?;

        let mut return_type = match self.peek() {
            Some(Token::Int) | Some(Token::Double) | Some(Token::Str) => {
                self.parse_type()?
            }
            Some(Token::Identifier(_)) => {
                let save_pos = self.pos;
                let ident = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                    self.advance();
                    n
                } else {
                    return Err("Expected identifier after 'fn'".to_string());
                };
                if self.peek() == Some(&Token::LParen) {
                    self.pos = save_pos;
                    Type::Int
                } else {
                    if matches!(self.peek(), Some(Token::Identifier(_))) {
                        Type::Struct(ident)
                    } else {
                        return Err("Expected function name after return type".to_string());
                    }
                }
            }
            _ => Type::Int,
        };

        if return_type == Type::Array || return_type == Type::DoubleArray {
            return Err("Array return type not supported".to_string());
        }

        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected function name".to_string());
        };

        self.expect(Token::LParen)?;
        let mut params = vec![];
        if self.peek() != Some(&Token::RParen) {
            loop {
                let param_type = self.parse_type()?;
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

        if self.peek() == Some(&Token::Arrow) {
            self.advance();
            return_type = self.parse_type()?;
        }

        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        Ok(Function { name, params, return_type, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        while self.peek() != Some(&Token::RBrace) && self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    // -------- Типы ----------
    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek() {
            Some(Token::Int) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.expect(Token::LBracket)?;
                    self.expect(Token::RBracket)?;
                    return Ok(Type::Array);
                }
                Ok(Type::Int)
            }
            Some(Token::Double) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.expect(Token::LBracket)?;
                    self.expect(Token::RBracket)?;
                    return Ok(Type::DoubleArray);
                }
                Ok(Type::Double)
            }
            Some(Token::Str) => {
                self.advance();
                Ok(Type::Str)
            }
            Some(Token::Identifier(name)) => {
                let n = name.clone();
                self.advance();
                Ok(Type::Struct(n))
            }
            _ => Ok(Type::Int),
        }
    }

    // -------- Операторы ----------
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
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

        let first = self.peek().cloned();
        let var_type = match first {
            Some(Token::Int) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.expect(Token::LBracket)?;
                    self.expect(Token::RBracket)?;
                    Some(Type::Array)
                } else {
                    Some(Type::Int)
                }
            }
            Some(Token::Double) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.expect(Token::LBracket)?;
                    self.expect(Token::RBracket)?;
                    Some(Type::DoubleArray)
                } else {
                    Some(Type::Double)
                }
            }
            Some(Token::Str) => {
                self.advance();
                Some(Type::Str)
            }
            Some(Token::Identifier(ref name1)) => {
                let save_pos = self.pos;
                self.advance();
                let is_type = match self.peek() {
                    Some(Token::Identifier(_)) => true,
                    Some(Token::Equals) => false,
                    _ => {
                        self.pos = save_pos;
                        return Err("Expected variable name or type".to_string());
                    }
                };
                if is_type {
                    let type_name = name1.clone();
                    let var_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                        self.advance();
                        n
                    } else {
                        return Err("Expected variable name after type".to_string());
                    };
                    self.expect(Token::Equals)?;
                    let value = self.parse_expr()?;
                    return Ok(Stmt::Let { name: var_name, var_type: Some(Type::Struct(type_name)), value });
                } else {
                    let var_name = name1.clone();
                    self.expect(Token::Equals)?;
                    let value = self.parse_expr()?;
                    return Ok(Stmt::Let { name: var_name, var_type: None, value });
                }
            }
            _ => None,
        };

        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected variable name after 'let'".to_string());
        };

        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        Ok(Stmt::Let { name, var_type, value })
    }

    fn parse_assign_or_call_stmt(&mut self) -> Result<Stmt, String> {
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected identifier".to_string());
        };

        if self.peek() == Some(&Token::LBracket) {
            self.expect(Token::LBracket)?;
            let index = self.parse_expr()?;
            self.expect(Token::RBracket)?;
            self.expect(Token::Equals)?;
            let value = self.parse_expr()?;
            return Ok(Stmt::AssignIndex { name, index, value });
        }

        if self.peek() == Some(&Token::LParen) {
            let args = self.parse_call_args()?;
            if self.peek() == Some(&Token::Equals) {
                self.expect(Token::Equals)?;
                let value = Expr::Call { name: name.clone(), args };
                Ok(Stmt::Assign { name: name.clone(), value })
            } else {
                Ok(Stmt::CallStmt { name, args })
            }
        } else {
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

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(Token::For)?;
        let var_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected loop variable name in 'for'".to_string());
        };
        self.expect(Token::Equals)?;
        let start = self.parse_expr()?;
        self.expect(Token::Comma)?;
        let condition = self.parse_expr()?;
        self.expect(Token::Comma)?;
        // Шаг как присваивание
        let step_var = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err("Expected step variable in for".to_string());
        };
        self.expect(Token::Equals)?;
        let step_value = self.parse_expr()?;
        let _step = Stmt::Assign { name: step_var.clone(), value: step_value.clone() };
        self.expect(Token::LBrace)?;
        let mut body = self.parse_block()?;
        Ok(Stmt::For { var_name, start, condition, step: step_value, body })
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

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::LBracket) => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Index { array: Box::new(expr), index: Box::new(index) };
                }
                Some(Token::Dot) => {
                    self.advance();
                    let field = if let Some(Token::Identifier(f)) = self.peek().cloned() {
                        self.advance();
                        f
                    } else {
                        return Err("Expected field name after '.'".to_string());
                    };
                    expr = Expr::FieldAccess { expr: Box::new(expr), field };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

      fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => { self.advance(); Ok(Expr::Number(n)) }
            Some(Token::DoubleLiteral(d)) => { self.advance(); Ok(Expr::DoubleLiteral(d)) }
            Some(Token::StringLiteral(s)) => { self.advance(); Ok(Expr::StringLiteral(s)) }
            Some(Token::Identifier(name)) => {
                self.advance();
                if self.peek() == Some(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Some(Token::LBracket) => {
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
            // ----- ДОБАВЛЕННЫЙ БЛОК -----
            Some(Token::Str) | Some(Token::Int) => {
                let name = if matches!(self.peek(), Some(Token::Str)) { "str".to_string() } else { "int".to_string() };
                self.advance();
                if self.peek() == Some(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    Ok(Expr::Call { name, args })
                } else {
                    Err(format!("Expected '(' after '{}'", name))
                }
            }
            // ----------------------------
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
