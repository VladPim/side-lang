use crate::ast::*;
use crate::token::Token;

pub struct Parser {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
    source: String,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, std::ops::Range<usize>)>, source: String) -> Self {
        Self { tokens, pos: 0, source }
    }

    fn location(&self, pos: usize) -> (usize, usize) {
        if pos >= self.tokens.len() {
            return (0, 0);
        }
        let range = &self.tokens[pos].1;
        let start = range.start;
        let source = &self.source;
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in source.chars().enumerate() {
            if i == start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    fn format_error(&self, pos: usize, msg: &str) -> String {
        let (line, col) = self.location(pos);
        format!("Error at {}:{}: {}", line, col, msg)
    }

    fn current_span(&self) -> Span {
        if self.pos >= self.tokens.len() {
            return Span::new(0, 0);
        }
        let range = &self.tokens[self.pos].1;
        Span::new(range.start, range.end)
    }

    fn span_from(&self, start: usize, end: usize) -> Span {
        Span::new(start, end)
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut imports = vec![];
        let mut structs = vec![];
        let mut constants = vec![];
        let mut functions = vec![];

        while self.peek().is_some() {
            match self.peek() {
                Some(Token::Import) => {
                    let path = self.parse_import()?;
                    imports.push(path);
                }
                Some(Token::Struct) => {
                    let s = self.parse_struct_def()?;
                    structs.push(s);
                }
                Some(Token::Fn) => {
                    let f = self.parse_function()?;
                    functions.push(f);
                }
                Some(Token::Let) => {
                    let c = self.parse_constant()?;
                    constants.push(c);
                }
                other => return Err(self.format_error(self.pos, &format!("Unexpected top-level token: {:?}", other))),
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
            Err(self.format_error(self.pos, "Expected string literal after 'import'"))
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
            Err(self.format_error(self.pos, &format!(
                "Syntax error: expected {:?}, got {:?}",
                expected,
                self.peek()
            )))
        }
    }

    // -------- Константы ----------
    fn parse_constant(&mut self) -> Result<Constant, String> {
        let start = self.pos;
        self.expect(Token::Let)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected constant name after 'let'"));
        };
        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Constant { name, value, span })
    }

    // -------- Структуры ----------
    fn parse_struct_def(&mut self) -> Result<StructDef, String> {
        let start = self.pos;
        self.expect(Token::Struct)?;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected struct name"));
        };
        self.expect(Token::LBrace)?;
        let mut fields = vec![];
        while self.peek() != Some(&Token::RBrace) {
            let field_span = self.current_span();
            let field_type = self.parse_type()?;
            let field_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                self.advance();
                n
            } else {
                return Err(self.format_error(self.pos, "Expected field name"));
            };
            fields.push(Field { name: field_name, field_type, span: field_span });
        }
        self.expect(Token::RBrace)?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(StructDef { name, fields, span })
    }

    // -------- Функции и методы ----------
    fn parse_function(&mut self) -> Result<Function, String> {
        let start = self.pos;
        self.expect(Token::Fn)?;

        let mut struct_name = None;
        let mut return_type = Type::Int;
        let mut name = String::new();

        if let Some(Token::Identifier(first)) = self.peek().cloned() {
            let save_pos = self.pos;
            self.advance();
            if self.peek() == Some(&Token::Dot) {
                self.advance();
                if let Some(Token::Identifier(method_name)) = self.peek().cloned() {
                    self.advance();
                    struct_name = Some(first);
                    name = method_name;
                } else {
                    return Err(self.format_error(self.pos, "Expected method name after '.'"));
                }
            } else {
                self.pos = save_pos;
                match self.peek() {
                    Some(Token::Int) | Some(Token::Double) | Some(Token::Str) => {
                        return_type = self.parse_type()?;
                    }
                    Some(Token::Identifier(_)) => {
                        let save_pos2 = self.pos;
                        self.advance();
                        if let Some(Token::Identifier(_)) = self.peek() {
                            self.pos = save_pos2;
                            return_type = self.parse_type()?;
                        } else {
                            self.pos = save_pos2;
                            return_type = Type::Int;
                        }
                    }
                    _ => {
                        return_type = Type::Int;
                    }
                }
                if let Some(Token::Identifier(func_name)) = self.peek().cloned() {
                    self.advance();
                    name = func_name;
                } else {
                    return Err(self.format_error(self.pos, "Expected function name"));
                }
            }
        } else {
            return Err(self.format_error(self.pos, "Expected function name or struct name"));
        }

        self.expect(Token::LParen)?;
        let mut params = vec![];
        if self.peek() != Some(&Token::RParen) {
            loop {
                let param_span = self.current_span();
                let param_type = self.parse_type()?;
                let param_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                    self.advance();
                    n
                } else {
                    return Err(self.format_error(self.pos, "Expected parameter name"));
                };
                params.push(Param { name: param_name, param_type, span: param_span });
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
        let end = self.pos - 1;
        let span = self.span_from(start, end);

        Ok(Function {
            name,
            struct_name,
            params,
            return_type,
            body,
            span,
        })
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
                    self.advance(); // съедаем '['
                    // Проверяем, есть ли число внутри
                    let size = if let Some(Token::Number(n)) = self.peek().cloned() {
                        self.advance();
                        Some(n as usize)
                    } else {
                        // Пустые скобки -> динамический массив
                        None
                    };
                    self.expect(Token::RBracket)?;
                    return Ok(Type::Array {
                        elem: Box::new(Type::Int),
                        size,
                    });
                }
                Ok(Type::Int)
            }
            Some(Token::Double) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.advance();
                    let size = if let Some(Token::Number(n)) = self.peek().cloned() {
                        self.advance();
                        Some(n as usize)
                    } else {
                        None
                    };
                    self.expect(Token::RBracket)?;
                    return Ok(Type::Array {
                        elem: Box::new(Type::Double),
                        size,
                    });
                }
                Ok(Type::Double)
            }
            Some(Token::Str) => {
                self.advance();
                Ok(Type::Str)
            }
            Some(Token::Identifier(name)) => {
                // Пока структуры не могут быть массивами, но в будущем можно расширить
                let n = name.clone();
                self.advance();
                // Проверяем, не идёт ли после идентификатора '[' (например, Point[10])
                if self.peek() == Some(&Token::LBracket) {
                    return Err(self.format_error(self.pos, "Static arrays of structs are not yet supported"));
                }
                Ok(Type::Struct(n))
            }
            _ => Ok(Type::Int),
        }
    }

    // -------- Остальные парсеры (без изменений) ----------
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) => self.parse_while(),
            Some(Token::For) => self.parse_for(),
            Some(Token::Return) => self.parse_return(),
            Some(Token::Break) => {
                let span = self.current_span();
                self.advance();
                Ok(Stmt::Break(span))
            }
            Some(Token::Continue) => {
                let span = self.current_span();
                self.advance();
                Ok(Stmt::Continue(span))
            }
            Some(Token::Io) => self.parse_io_print(),
            Some(Token::Identifier(_)) => self.parse_assign_or_call_stmt(),
            other => Err(self.format_error(self.pos, &format!("Unexpected token at statement start: {:?}", other))),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.expect(Token::Let)?;

        let first = self.peek().cloned();
        let var_type = match first {
            Some(Token::Int) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.advance();
                    let size = if let Some(Token::Number(n)) = self.peek().cloned() {
                        self.advance();
                        Some(n as usize)
                    } else {
                        None
                    };
                    self.expect(Token::RBracket)?;
                    Some(Type::Array { elem: Box::new(Type::Int), size })
                } else {
                    Some(Type::Int)
                }
            }
            Some(Token::Double) => {
                self.advance();
                if self.peek() == Some(&Token::LBracket) {
                    self.advance();
                    let size = if let Some(Token::Number(n)) = self.peek().cloned() {
                        self.advance();
                        Some(n as usize)
                    } else {
                        None
                    };
                    self.expect(Token::RBracket)?;
                    Some(Type::Array { elem: Box::new(Type::Double), size })
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
                        return Err(self.format_error(self.pos, "Expected variable name or type"));
                    }
                };
                if is_type {
                    let type_name = name1.clone();
                    let var_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
                        self.advance();
                        n
                    } else {
                        return Err(self.format_error(self.pos, "Expected variable name after type"));
                    };
                    self.expect(Token::Equals)?;
                    let value = self.parse_expr()?;
                    let end = self.pos - 1;
                    let span = self.span_from(start, end);
                    return Ok(Stmt::Let { name: var_name, var_type: Some(Type::Struct(type_name)), value, span });
                } else {
                    let var_name = name1.clone();
                    self.expect(Token::Equals)?;
                    let value = self.parse_expr()?;
                    let end = self.pos - 1;
                    let span = self.span_from(start, end);
                    return Ok(Stmt::Let { name: var_name, var_type: None, value, span });
                }
            }
            _ => None,
        };

        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected variable name after 'let'"));
        };

        self.expect(Token::Equals)?;
        let value = self.parse_expr()?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::Let { name, var_type, value, span })
    }

    fn parse_assign_or_call_stmt(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        let name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected identifier"));
        };

        if self.peek() == Some(&Token::LBracket) {
            self.expect(Token::LBracket)?;
            let index = self.parse_expr()?;
            self.expect(Token::RBracket)?;
            self.expect(Token::Equals)?;
            let value = self.parse_expr()?;
            let end = self.pos - 1;
            let span = self.span_from(start, end);
            return Ok(Stmt::AssignIndex { name, index, value, span });
        }

        if self.peek() == Some(&Token::LParen) {
            let args = self.parse_call_args()?;
            if self.peek() == Some(&Token::Equals) {
                self.expect(Token::Equals)?;
                let value = Expr::Call { name: name.clone(), args, span: self.current_span() };
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Stmt::Assign { name: name.clone(), value, span })
            } else {
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Stmt::CallStmt { name, args, span })
            }
        } else {
            let op = match self.peek() {
                Some(Token::PlusEquals) => Some(BinOp::Add),
                Some(Token::MinusEquals) => Some(BinOp::Sub),
                Some(Token::StarEquals) => Some(BinOp::Mul),
                Some(Token::SlashEquals) => Some(BinOp::Div),
                _ => None,
            };

            if let Some(bin_op) = op {
                self.advance();
                let rhs = self.parse_expr()?;
                let left = Expr::Variable(name.clone(), self.current_span());
                let right = rhs;
                let binary_span = left.span().merge(&right.span());
                let value = Expr::Binary {
                    left: Box::new(left),
                    op: bin_op,
                    right: Box::new(right),
                    span: binary_span,
                };
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Stmt::Assign { name, value, span })
            } else {
                self.expect(Token::Equals)?;
                let value = self.parse_expr()?;
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Stmt::Assign { name, value, span })
            }
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
        let start = self.pos;
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
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::If { condition, then_body, else_body, span })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.expect(Token::While)?;
        let condition = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::While { condition, body, span })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.expect(Token::For)?;
        let var_name = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected loop variable name in 'for'"));
        };
        self.expect(Token::Equals)?;
        let start_expr = self.parse_expr()?;
        self.expect(Token::Comma)?;
        let condition = self.parse_expr()?;
        self.expect(Token::Comma)?;
        let step_var = if let Some(Token::Identifier(n)) = self.peek().cloned() {
            self.advance();
            n
        } else {
            return Err(self.format_error(self.pos, "Expected step variable in for"));
        };
        self.expect(Token::Equals)?;
        let step_value = self.parse_expr()?;
        let _step = Stmt::Assign { name: step_var.clone(), value: step_value.clone(), span: self.current_span() };
        self.expect(Token::LBrace)?;
        let body = self.parse_block()?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::For { var_name, start: start_expr, condition, step: step_value, body, span })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
        self.expect(Token::Return)?;
        let expr = self.parse_expr()?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::Return(expr, span))
    }

    fn parse_io_print(&mut self) -> Result<Stmt, String> {
        let start = self.pos;
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
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Stmt::IoPrint(args, span))
    }

    // -------- Выражения ----------
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_or()?;
        if self.peek() == Some(&Token::Question) {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(Token::Colon)?;
            let else_expr = self.parse_ternary()?;
            let span = cond.span().merge(&else_expr.span());
            Ok(Expr::Ternary {
                condition: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
                span,
            })
        } else {
            Ok(cond)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            let span = left.span().merge(&right.span());
            left = Expr::Binary { left: Box::new(left), op: BinOp::Or, right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            let span = left.span().merge(&right.span());
            left = Expr::Binary { left: Box::new(left), op: BinOp::And, right: Box::new(right), span };
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
                Token::GreaterEq => BinOp::GreaterEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            let span = left.span().merge(&right.span());
            left = Expr::Binary { left: Box::new(left), op: binop, right: Box::new(right), span };
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
                    let span = left.span().merge(&right.span());
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Add, right: Box::new(right), span };
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication()?;
                    let span = left.span().merge(&right.span());
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Sub, right: Box::new(right), span };
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
                    let span = left.span().merge(&right.span());
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Mul, right: Box::new(right), span };
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    let span = left.span().merge(&right.span());
                    left = Expr::Binary { left: Box::new(left), op: BinOp::Div, right: Box::new(right), span };
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
            let span = self.current_span().merge(&expr.span());
            return Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr), span });
        }
        if self.peek() == Some(&Token::Minus) {
            self.advance();
            let expr = self.parse_unary()?;
            let span = self.current_span().merge(&expr.span());
            return Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr), span });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::LBracket) => {
                    let _start = self.pos;
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    let span = expr.span().merge(&index.span());
                    expr = Expr::Index { array: Box::new(expr), index: Box::new(index), span };
                }
                Some(Token::Dot) => {
                    let _start = self.pos;
                    self.advance();
                    let field = if let Some(Token::Identifier(f)) = self.peek().cloned() {
                        self.advance();
                        f
                    } else {
                        return Err(self.format_error(self.pos, "Expected field name after '.'"));
                    };
                    let span = expr.span().merge(&self.current_span());
                    if self.peek() == Some(&Token::LParen) {
                        let args = self.parse_call_args()?;
                        let call_span = span.merge(&self.current_span());
                        expr = Expr::MethodCall {
                            instance: Box::new(expr),
                            method: field,
                            args,
                            span: call_span,
                        };
                    } else {
                        expr = Expr::FieldAccess { expr: Box::new(expr), field, span };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_struct_literal(&mut self, name: String) -> Result<Expr, String> {
        let start = self.pos;
        self.expect(Token::LBrace)?;
        let mut fields = vec![];
        if self.peek() != Some(&Token::RBrace) {
            loop {
                fields.push(self.parse_expr()?);
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::RBrace)?;
        let end = self.pos - 1;
        let span = self.span_from(start, end);
        Ok(Expr::StructLiteral { name, fields, span })
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::Number(n, span))
            }
            Some(Token::DoubleLiteral(d)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::DoubleLiteral(d, span))
            }
            Some(Token::StringLiteral(s)) => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::StringLiteral(s, span))
            }
            Some(Token::Identifier(name)) => {
                let span = self.current_span();
                self.advance();
                if self.peek() == Some(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    let call_span = span.merge(&self.current_span());
                    Ok(Expr::Call { name, args, span: call_span })
                } else if self.peek() == Some(&Token::LBrace) {
                    self.parse_struct_literal(name)
                } else {
                    Ok(Expr::Variable(name, span))
                }
            }
            Some(Token::LBracket) => {
                let start = self.pos;
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
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Expr::ArrayLiteral(elements, span))
            }
            Some(Token::Io) => {
                let start = self.pos;
                self.advance();
                self.expect(Token::Dot)?;
                self.expect(Token::Input)?;
                self.expect(Token::LParen)?;
                let prompt = if let Some(Token::StringLiteral(s)) = self.peek().cloned() {
                    self.advance();
                    s
                } else {
                    return Err(self.format_error(self.pos, "Expected prompt string for io.input"));
                };
                self.expect(Token::RParen)?;
                let end = self.pos - 1;
                let span = self.span_from(start, end);
                Ok(Expr::Input(prompt, span))
            }
            Some(Token::Str) | Some(Token::Int) => {
                let name = if matches!(self.peek(), Some(Token::Str)) { "str".to_string() } else { "int".to_string() };
                let span = self.current_span();
                self.advance();
                if self.peek() == Some(&Token::LParen) {
                    let args = self.parse_call_args()?;
                    let call_span = span.merge(&self.current_span());
                    Ok(Expr::Call { name, args, span: call_span })
                } else {
                    Err(self.format_error(self.pos, &format!("Expected '(' after '{}'", name)))
                }
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            other => Err(self.format_error(self.pos, &format!("Unexpected token in expression: {:?}", other))),
        }
    }
}
