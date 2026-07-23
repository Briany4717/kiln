use crate::core::callable::{AmystType, Param};
use crate::core::expr::{AST, ExprId, ExprKind, LiteralValue, Stmt, StmtId};
use crate::core::scanner::{Token, TokenType};
use crate::{AmystError, report_error};
use std::borrow::Cow;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self, ast: &mut AST<'a>) -> Result<Vec<StmtId>, AmystError<'a>> {
        let mut root_stmt = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            let stmt = self.declaration(ast);
            match stmt {
                Ok(s) => {
                    let new_node = ast.add_stmt(s);
                    root_stmt.push(new_node);
                }
                Err(e) => {
                    errors.push(e);
                    self.synchronize()
                }
            }
        }
        if errors.is_empty() {
            Ok(root_stmt)
        } else {
            Err(AmystError::Multiple(errors))
        }
    }

    fn expression(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        self.assignment(ast)
    }

    fn declaration(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        if self.matches(&[TokenType::Fn]) {
            self.function(ast, "function")
        } else if self.matches(&[TokenType::Let]) {
            self.var_declaration(ast)
        } else {
            self.statement(ast)
        }
    }

    fn statement(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        if self.matches(&[TokenType::For]) {
            self.for_stmt(ast)
        } else if self.matches(&[TokenType::Print]) {
            self.print_stmt(ast)
        } else if self.matches(&[TokenType::Return]) {
            self.return_stmt(ast)
        } else if self.matches(&[TokenType::While]) {
            self.while_stmt(ast)
        } else {
            self.expression_stmt(ast)
        }
    }

    fn for_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let variable = self.consume_identifier("Expected an identifier.")?;
        self.consume(TokenType::In, "Expected 'in'.")?;

        let iterable = self.expression(ast)?;
        let body_stmt = self.statement(ast)?;
        let body = ast.add_stmt(body_stmt);
        Ok(Stmt::For {
            variable,
            iterable,
            body,
        })
    }

    fn if_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let condition = self.expression(ast)?;
        let then_branch = self.expression(ast)?;
        let mut else_branch = None;
        if self.matches(&[TokenType::Else]) {
            else_branch = Some(self.expression(ast)?);
        }
        Ok(Stmt::If(ast.add_node(ExprKind::If {
            condition,
            then_branch,
            else_branch,
        })))
    }

    fn print_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let val = self.expression(ast)?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(val))
    }

    fn return_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let keyword = self.previous();

        let value = if !self.check(&TokenType::Semicolon) {
            self.expression(ast)?
        } else {
            ast.add_node(ExprKind::Literal(LiteralValue::Unit))
        };
        self.consume(TokenType::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn var_declaration(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let name = self.consume_identifier("Expect variable name.")?;
        let mut initializer = None;
        if self.matches(&[TokenType::Equal]) {
            initializer = Some(self.expression(ast)?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn while_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let condition = self.expression(ast)?;
        let body_stmt = self.statement(ast)?;
        let body = ast.add_stmt(body_stmt);
        Ok(Stmt::While { condition, body })
    }

    fn expression_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, AmystError<'a>> {
        let val = self.expression(ast)?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(val))
    }

    fn function(&mut self, ast: &mut AST<'a>, kind: &'a str) -> Result<Stmt<'a>, AmystError<'a>> {
        let name = self.consume_identifier(&format!("Expect {} name.", kind))?;
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            let is_mut = self.matches(&[TokenType::Mut]);
            let name = self.consume_identifier("Expect parameter name.")?;
            let type_annotation = if self.matches(&[TokenType::Colon]) {
                Some(self.parse_type("Expected parameter type identifier.")?)
            } else {
                None
            };
            params.push(Param {
                name,
                is_mut,
                type_annotation,
            });

            while self.matches(&[TokenType::Comma]) {
                let is_mut = self.matches(&[TokenType::Mut]);
                let name = self.consume_identifier("Expect parameter name.")?;
                let type_annotation = if self.matches(&[TokenType::Colon]) {
                    Some(self.parse_type("Expected parameter type identifier.")?)
                } else {
                    None
                };
                params.push(Param {
                    name,
                    is_mut,
                    type_annotation,
                });
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        let return_type = if self.check(&TokenType::Arrow) {
            self.advance();
            Some(self.parse_type("Expected return type.")?)
        } else {
            None
        };

        let body = self.block(ast)?;

        Ok(Stmt::Function {
            name,
            params,
            body,
            return_type,
        })
    }

    fn parse_type(&mut self, message: &str) -> Result<AmystType, AmystError<'a>> {
        if self.matches(&[TokenType::LeftParen]) {
            self.consume(TokenType::RightParen, "Expected ')' to close unit type.")?;
            return Ok(AmystType::Unit);
        }

        let tok = self.consume_identifier(message)?;
        let ty = match tok.lexeme {
            "int" => AmystType::Int,
            "float" => AmystType::Float,
            "string" => AmystType::String,
            "bool" => AmystType::Bool,
            "unit" => AmystType::Unit,
            name => AmystType::Named(name.to_string()),
        };
        Ok(ty)
    }

    fn block(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        self.consume(TokenType::LeftBrace, "Expect '{' before block.")?;

        let mut stmts = Vec::new();
        let mut expr = None;

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.check(&TokenType::Let) || self.check(&TokenType::Fn) {
                let stmt = self.declaration(ast)?;
                stmts.push(ast.add_stmt(stmt));
            } else if self.check(&TokenType::For)
                || self.check(&TokenType::While)
                || self.check(&TokenType::Print)
                || self.check(&TokenType::Return)
            {
                let stmt = self.statement(ast)?;
                stmts.push(ast.add_stmt(stmt));
            } else {
                let expr_id = self.expression(ast)?;

                if self.matches(&[TokenType::Semicolon]) {
                    stmts.push(ast.add_stmt(Stmt::Expression(expr_id)));
                } else if self.check(&TokenType::RightBrace) {
                    expr = Some(expr_id);
                    break;
                } else {
                    if matches!(
                        ast.get_node(expr_id),
                        ExprKind::If { .. } | ExprKind::Block { .. }
                    ) {
                        stmts.push(ast.add_stmt(Stmt::Expression(expr_id)));
                    } else {
                        return Err(AmystError::Runtime {
                            message: report_error(
                                self.peek().line,
                                Some(&format!(" at '{}'", self.peek().lexeme)),
                                "Expect ';' after expression.",
                            ),
                        });
                    }
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(ast.add_node(ExprKind::Block { stmts, expr }))
    }

    fn assignment(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let expr = self.range(ast)?;

        if self.matches(&[TokenType::Equal]) {
            let _equals = self.previous();
            let value = self.assignment(ast)?;
            let exp_k = ast.get_node(expr);
            return match exp_k {
                ExprKind::Variable(tk) => Ok(ast.add_node(ExprKind::Assign {
                    name: tk.clone(),
                    value,
                })),
                _ => Err(AmystError::Runtime {
                    message: "Invalid assignment target.".to_string(),
                }),
            };
        }
        Ok(expr)
    }

    fn range(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.or(ast)?;

        if self.matches(&[TokenType::DotDot, TokenType::DotDotEqual]) {
            let op = self.previous().token_type.clone();
            let right = self.or(ast)?;
            let inclusive = matches!(op, TokenType::DotDotEqual);
            expr = ast.add_node(ExprKind::Range {
                start: expr,
                end: right,
                inclusive,
            });
        }

        Ok(expr)
    }

    fn or(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.and(ast)?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and(ast)?;
            expr = ast.add_node(ExprKind::Logical {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn and(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.equality(ast)?;

        while self.matches(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.equality(ast)?;
            expr = ast.add_node(ExprKind::Logical {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn equality(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.comparison(ast)?;
        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison(ast)?;
            expr = ast.add_node(ExprKind::Binary {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn comparison(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.term(ast)?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term(ast)?;
            expr = ast.add_node(ExprKind::Binary {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn term(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.factor(ast)?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor(ast)?;
            expr = ast.add_node(ExprKind::Binary {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn factor(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.unary(ast)?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary(ast)?;
            expr = ast.add_node(ExprKind::Binary {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn unary(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary(ast)?;
            return Ok(ast.add_node(ExprKind::Unary { operator, right }));
        }

        self.call(ast)
    }

    fn finish_call(&mut self, ast: &mut AST<'a>, callee: ExprId) -> Result<ExprId, AmystError<'a>> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            arguments.push(self.expression(ast)?);
            while self.matches(&[TokenType::Comma]) {
                // Design TODO: Implement argument count limit
                arguments.push(self.expression(ast)?);
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(ast.add_node(ExprKind::Call {
            callee,
            paren,
            arguments,
        }))
    }

    fn call(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        let mut expr = self.primary(ast)?;
        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(ast, expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self, ast: &mut AST<'a>) -> Result<ExprId, AmystError<'a>> {
        if self.is_at_end() {
            return Err(AmystError::Runtime {
                message: "Invalid token".to_string(),
            });
        }

        let tok = self.advance();
        let expr = match tok.token_type {
            TokenType::False => ExprKind::Literal(LiteralValue::Boolean(false)),
            TokenType::True => ExprKind::Literal(LiteralValue::Boolean(true)),
            TokenType::Unit => ExprKind::Literal(LiteralValue::Unit),
            TokenType::Number(n) => ExprKind::Literal(LiteralValue::Number(n)),
            TokenType::String(s) => ExprKind::Literal(LiteralValue::String(Cow::from(s))),
            TokenType::LeftParen => {
                let exp = self.expression(ast)?;
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                return Ok(ast.add_node(ExprKind::Grouping(exp)));
            }
            TokenType::LeftBrace => {
                self.current -= 1;
                return self.block(ast);
            }
            TokenType::If => {
                let condition = self.expression(ast)?;
                let then_branch = self.block(ast)?;
                let mut else_branch = None;

                if self.matches(&[TokenType::Else]) {
                    if self.check(&TokenType::If) {
                        self.advance();
                        else_branch = Some(self.primary(ast)?);
                    } else {
                        else_branch = Some(self.block(ast)?);
                    }
                }

                ExprKind::If {
                    condition,
                    then_branch,
                    else_branch,
                }
            }
            TokenType::Identifier(_) => ExprKind::Variable(self.previous()),
            _ => {
                return Err(AmystError::Runtime {
                    message: report_error(
                        tok.line,
                        Some(&format!(" at '{}'", tok.lexeme)),
                        "Expect expression.",
                    ),
                });
            }
        };

        Ok(ast.add_node(expr))
    }

    fn matches(&mut self, types: &[TokenType<'a>]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(
        &mut self,
        token_type: TokenType<'a>,
        message: &str,
    ) -> Result<Token<'a>, AmystError<'a>> {
        let tok = token_type.clone();
        if self.check(&token_type) {
            return Ok(self.advance());
        }
        match tok {
            TokenType::Eof => Err(AmystError::Runtime {
                message: report_error(self.peek().line, Some(" at end"), message),
            }),
            _ => Err(AmystError::Runtime {
                message: report_error(
                    self.peek().line,
                    Some(&format!(" at '{}'", self.peek().lexeme)),
                    message,
                ),
            }),
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<Token<'a>, AmystError<'a>> {
        if matches!(self.peek().token_type, TokenType::Identifier(_)) {
            Ok(self.advance())
        } else {
            Err(AmystError::Runtime {
                message: report_error(
                    self.peek().line,
                    Some(&format!(" at '{}'", self.peek().lexeme)),
                    message,
                ),
            })
        }
    }

    fn check_next(&self, t: &TokenType) -> bool {
        if self.peek_next().token_type == TokenType::Eof {
            false
        } else {
            self.peek_next().token_type == *t
        }
    }

    fn check(&self, t: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == *t
        }
    }

    fn advance(&mut self) -> Token<'a> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> Token<'a> {
        self.tokens[self.current].clone()
    }

    fn peek_next(&self) -> Token<'a> {
        self.tokens[self.current + 1].clone()
    }

    fn previous(&self) -> Token<'a> {
        self.tokens[self.current - 1].clone()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fn
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Let
                | TokenType::While => return,
                _ => {}
            }

            self.advance();
        }
    }
}

pub(crate) fn ensure_int<'a>(val: f64) -> Result<i32, AmystError<'a>> {
    if val.fract() != 0.0 {
        let message = String::from("Float has a fractional component and is not a whole integer");
        return Err(AmystError::Runtime { message });
    }

    if val < i32::MIN as f64 || val > i32::MAX as f64 || val.is_nan() {
        let message = String::from("Float is out of bounds for an i32 integer");
        return Err(AmystError::Runtime { message });
    }

    Ok(val as i32)
}

#[cfg(test)]
mod test {
    use crate::AmystError;
    use crate::core::Scanner;
    use crate::core::expr::{AST, format_ast};
    use crate::core::parser::Parser;

    #[test]
    fn parsing_test<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let scanner = Scanner::new("1 + 2 * 3");
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let id = parser.expression(&mut ast)?;
        assert_eq!(format_ast(&ast, id), "(+ 1 (* 2 3))");
        Ok(())
    }
}
