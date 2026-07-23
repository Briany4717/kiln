use std::time::{SystemTime, UNIX_EPOCH};
use crate::AmystError;
use crate::core::env::ScopeStack;
use crate::core::expr::{LiteralValue, Stmt, StmtId, evaluate, AST};
use crate::core::callable::AmystCallable;
pub struct Interpreter<'a> {
    pub env: ScopeStack<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        let mut env = ScopeStack::new();

        env.define_global("clock", LiteralValue::Callable(AmystCallable::Native {
            arity: 0,
            name: "clock",
            func: |_args|{Ok(LiteralValue::Number(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as f64))},
        }));

        Self {
            env
        }
    }

    pub(crate) fn interpret(
        &mut self,
        ast: &AST<'a>,
        statements: &[StmtId],
    ) -> Result<(), AmystError> {
        for stmt_id in statements {
            self.execute(ast, *stmt_id)?;
        }
        Ok(())
    }

    pub(crate) fn execute(&mut self, ast: &AST<'a>, stmt_id: StmtId) -> Result<(), AmystError> {
        let env = &mut self.env;
        match ast.get_stmt(stmt_id) {
            Stmt::Print(id) => {
                let val = evaluate(ast,self, *id)?;
                println!("{}", self.stringify(val))
            }
            Stmt::Expression(id) => {
                evaluate(ast, self, *id)?;
            }
            Stmt::Var { name, initializer } => {
                if let Some(id) = initializer {
                    let val = evaluate(ast, self, *id)?;
                    self.env.define(name.lexeme, val)
                } else {
                    self.env.define(name.lexeme, LiteralValue::Nil)
                }
            }
            Stmt::Block(stmts) => {
                env.push_scope();
                for stmt in stmts {
                    self.execute(ast, *stmt)?;
                }
                self.env.pop_scope()
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&evaluate(ast, self, *condition)?)? {
                    self.execute(ast, *then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(ast, *else_branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&evaluate(ast, self, *condition)?)? {
                    self.execute(ast, *body)?;
                }
            }
            Stmt::For {
                variable,
                iterable,
                body,
            } => {
                let iterator = evaluate(ast, self, *iterable)?;
                match iterator {
                    LiteralValue::Range {
                        start,
                        end,
                        inclusive,
                    } => {
                        let iter: Box<dyn Iterator<Item = i32>> = if inclusive {
                            Box::new(start..=end)
                        } else {
                            Box::new(start..end)
                        };

                        for i in iter {
                            self.env.push_scope();
                            self.env
                                .define(variable.lexeme, LiteralValue::Number(i as f64));
                            let result = self.execute(ast, *body);
                            self.env.pop_scope();

                            result?;
                        }
                    }
                    _ => {
                        return Err(AmystError::Runtime {
                            message: "Expected an iterable object.".to_string(),
                        });
                    }
                }
            }
            Stmt::Function {name,params,body} => {
                env.define(name.lexeme,LiteralValue::Callable(AmystCallable::UserDefined {
                    name: (*name).clone(),
                    params: (*params).clone(),
                    body: *body
                }))
            }
        }
        Ok(())
    }

    fn stringify(&self, val: LiteralValue) -> String {
        match val {
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::String(s) => s.to_string(),
            LiteralValue::Boolean(b) => String::from(if b { "true" } else { "false" }),
            LiteralValue::Range {
                start,
                end,
                inclusive,
            } => {
                if inclusive {
                    format!("{start}..={end}")
                } else {
                    format!("{start}..{end}")
                }
            }
            LiteralValue::Nil => String::from("nil"),
            LiteralValue::Callable(func) => match func {
                AmystCallable::Native { name, .. } => format!("<native fn {name}>"),
                AmystCallable::UserDefined { name, .. } => format!("<fn {}>", name.lexeme),
            },
        }
    }
}

pub(crate) fn is_truthy(val: &LiteralValue) -> Result<bool, AmystError> {
    match val {
        LiteralValue::Boolean(b) => Ok(*b),
        _ => {
            let message = String::from("Expected boolean expression");
            Err(AmystError::Runtime { message })
        }
    }
}
