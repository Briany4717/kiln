use crate::KilnError;
use crate::core::AST;
use crate::core::env::Env;
use crate::core::expr::{LiteralValue, Stmt, StmtId, evaluate};

pub struct Interpreter<'a> {
    env: Env<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Self { env: Env::new() }
    }

    pub(crate) fn interpret(
        &mut self,
        ast: &'a AST,
        statements: &[StmtId],
    ) -> Result<(), KilnError> {
        for stmt_id in statements {
            self.execute(ast, *stmt_id)?;
        }
        Ok(())
    }

    fn execute(&mut self, ast: &'a AST, stmt_id: StmtId) -> Result<(), KilnError> {
        let env = &mut self.env;
        match ast.get_stmt(stmt_id) {
            Stmt::Print(id) => {
                let val = evaluate(ast, env, *id)?;
                println!("{}", self.stringify(val))
            }
            Stmt::Expression(id) => {
                evaluate(ast, env, *id)?;
            }
            Stmt::Var { name, initializer } => {
                if let Some(id) = initializer {
                    let val = evaluate(ast, env, *id)?;
                    self.env.define(name.lexeme, val)
                } else {
                    self.env.define(name.lexeme, LiteralValue::Nil)
                }
            }
        }
        Ok(())
    }

    fn stringify(&self, val: LiteralValue) -> String {
        match val {
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::String(s) => s.to_string(),
            LiteralValue::Boolean(b) => String::from(if b { "true" } else { "false" }),
            LiteralValue::Nil => String::from("nil"),
        }
    }
}
