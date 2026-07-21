pub(crate) use crate::core::expr::{AST, ExprKind, LiteralValue, NodeId};
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

    pub(crate) fn parse(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
        self.expression(ast)
    }

    fn expression(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
        self.equality(ast)
    }

    fn equality(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
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

    fn comparison(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
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

    fn term(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
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

    fn factor(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
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

    fn unary(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary(ast)?;
            return Ok(ast.add_node(ExprKind::Unary { operator, right }));
        }

        self.primary(ast)
    }

    fn primary(&mut self, ast: &mut AST<'a>) -> Result<NodeId, KilnError> {
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
                | TokenType::Var
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
