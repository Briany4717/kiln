use crate::AmystError;
use crate::ast::AST;
use crate::interpreter::{Interpreter, Value};
use crate::lexer::Scanner;
use crate::parser::Parser;

pub struct Engine<'a> {
    interpreter: Interpreter<'a>,
}

impl<'a> Engine<'a> {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    /// Ejecuta un script completo (statements).
    pub fn run(&mut self, source: &'a str) -> Result<(), AmystError<'a>> {
        let (ast, stmts) = self.compile(source)?;
        self.interpreter.interpret(&ast, &stmts)
    }

    /// Evalúa una única expresión y devuelve su valor.
    pub fn eval(&mut self, source: &'a str) -> Result<Value<'a>, AmystError<'a>> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let mut ast = AST::new();
        let expr_id = parser.expression(&mut ast)?;

        crate::ast::evaluate(&ast, &mut self.interpreter, expr_id)
    }

    /// Solo scanea + parsea, sin ejecutar.
    pub fn compile(
        &self,
        source: &'a str,
    ) -> Result<(AST<'a>, Vec<crate::ast::StmtId>), AmystError<'a>> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let mut ast = AST::new();
        let stmts = parser.parse(&mut ast)?;
        Ok((ast, stmts))
    }
}

impl<'a> Default for Engine<'a> {
    fn default() -> Self {
        Self::new()
    }
}
