pub(crate) use crate::core::expr::{AST, ExprKind, LiteralValue, ExprId};
use crate::core::expr::{Stmt, StmtId};
use crate::core::scanner::{Token, TokenType};
use crate::{KilnError, report_error};
use std::borrow::Cow;

pub struct Parser<'a> {
    tokens: &'a [Token<'a>],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self { tokens, current: 0 }
    }

    pub(crate) fn parse(&mut self, ast: &mut AST<'a>) -> Result<Vec<StmtId>, KilnError> {
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
            Err(KilnError::Multiple(errors))
        }
    }

    fn expression(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        self.assignment(ast)
    }

    fn declaration(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        if self.matches(&[TokenType::Let]) {
            self.var_declaration(ast)
        } else {
            self.statement(ast)
        }
    }

    fn statement(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        if self.matches(&[TokenType::If]) {
            self.if_stmt(ast)
        } else if self.matches(&[TokenType::Print]) {
            self.print_stmt(ast)
        } else if self.matches(&[TokenType::While]){
            self.while_stmt(ast)
        } else if self.matches(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block(ast)?))
        } else {
            self.expression_stmt(ast)
        }
    }

    fn if_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        let condition = self.expression(ast)?;
        let stmt = self.statement(ast)?;
        let then_branch = ast.add_stmt(stmt);
        let mut else_branch = None;
        if self.matches(&[TokenType::Else]){
            let stmt = self.statement(ast)?;
            else_branch = Some(ast.add_stmt(stmt));
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch
        })
    }

    fn print_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        let val = self.expression(ast)?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(val))
    }

    fn var_declaration(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
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

    fn while_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        let condition = self.expression(ast)?;
        let body_stmt = self.statement(ast)?;
        let body= ast.add_stmt(body_stmt);
        Ok(Stmt::While {condition, body})
    }

    fn expression_stmt(&mut self, ast: &mut AST<'a>) -> Result<Stmt<'a>, KilnError> {
        let val = self.expression(ast)?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(val))
    }

    fn block(&mut self, ast: &mut AST<'a>) -> Result<Vec<StmtId>, KilnError> {
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let stmt = self.declaration(ast)?;
            stmts.push(ast.add_stmt(stmt));
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn assignment(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        let expr = self.or(ast)?;
        if self.matches(&[TokenType::Equal]) {
            let _equals = self.previous();
            let value = self.assignment(ast)?;

            let exp_k = ast.get_node(expr);
            return match exp_k {
                ExprKind::Variable(tk) => Ok(ast.add_node(ExprKind::Assign {
                    name: tk.clone(),
                    value,
                })),
                _ => {
                    let message = "Invalid assignment target.".to_string();
                    Err(KilnError::Runtime { message })
                }
            };
        }

        Ok(expr)
    }

    fn or(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        let mut expr = self.and(ast)?;

        while self.matches(&[TokenType::Or]){
            let operator = self.previous();
            let right = self.and(ast)?;
            expr = ast.add_node(ExprKind::Logical {
                left: expr,
                operator,
                right
            })
        }

       Ok(expr)
    }

    fn and(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        let mut expr = self.equality(ast)?;

        while self.matches(&[TokenType::And]){
            let operator = self.previous();
            let right = self.equality(ast)?;
            expr = ast.add_node(ExprKind::Logical {
                left: expr,
                operator,
                right
            })
        }

        Ok(expr)
    }

    fn equality(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
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

    fn comparison(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
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

    fn term(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
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

    fn factor(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
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

    fn unary(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary(ast)?;
            return Ok(ast.add_node(ExprKind::Unary { operator, right }));
        }

        self.primary(ast)
    }

    fn primary(&mut self, ast: &mut AST<'a>) -> Result<ExprId, KilnError> {
        if self.is_at_end() {
            return Err(KilnError::Runtime {
                message: "Invalid token".to_string(),
            });
        }
        let expr;
        match self.advance().token_type {
            TokenType::False => expr = ExprKind::Literal(LiteralValue::Boolean(false)),
            TokenType::True => expr = ExprKind::Literal(LiteralValue::Boolean(true)),
            TokenType::Nil => expr = ExprKind::Literal(LiteralValue::Nil),
            TokenType::Number(n) => expr = ExprKind::Literal(LiteralValue::Number(n)),
            TokenType::String(s) => expr = ExprKind::Literal(LiteralValue::String(Cow::from(s))),
            TokenType::LeftParen => {
                let exp = self.expression(ast)?;
                self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
                return Ok(ast.add_node(ExprKind::Grouping(exp)));
            }
            TokenType::Identifier(_) => expr = ExprKind::Variable(self.previous()),
            _ => {
                return Err(KilnError::Runtime {
                    message: "Expect expression.".to_string(),
                });
            }
        }
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
    ) -> Result<Token<'a>, KilnError> {
        let tok = token_type.clone();
        if self.check(&token_type) {
            return Ok(self.advance());
        }
        match tok {
            TokenType::Eof => Err(KilnError::Runtime {
                message: report_error(self.peek().line, Some(" at end"), message),
            }),
            _ => Err(KilnError::Runtime {
                message: report_error(
                    self.peek().line,
                    Some(&format!(" at '{}'", self.peek().lexeme)),
                    message,
                ),
            }),
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<Token<'a>, KilnError> {
        if matches!(self.peek().token_type, TokenType::Identifier(_)) {
            Ok(self.advance())
        } else {
            Err(KilnError::Runtime {
                message: report_error(
                    self.peek().line,
                    Some(&format!(" at '{}'", self.peek().lexeme)),
                    message,
                ),
            })
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

#[cfg(test)]
mod test {
    use crate::KilnError;
    use crate::core::Scanner;
    use crate::core::expr::{AST, format_ast};
    use crate::core::parser::Parser;

    #[test]
    fn parsing_test() -> Result<(), KilnError> {
        let mut ast = AST::new();
        let scanner = Scanner::new("1 + 2 * 3");
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(&*tokens);
        let id = parser.expression(&mut ast)?;
        assert_eq!(format_ast(&ast, id), "(+ 1 (* 2 3))");
        Ok(())
    }
}
