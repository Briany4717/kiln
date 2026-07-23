mod environment;
mod value;
mod callable;

pub use value::Value;
pub use callable::{AmystCallable};

use std::time::{SystemTime, UNIX_EPOCH};
use crate::{report_error, AmystError};
use crate::ast::{evaluate, Stmt, StmtId, AST};
use crate::interpreter::environment::ScopeStack;
pub struct Interpreter<'a> {
    pub env: ScopeStack<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        let mut env = ScopeStack::new();

        env.define_global(
            "clock",
            Value::Callable(AmystCallable::Native {
                arity: 0,
                name: "clock",
                func: |_args| {
                    Ok(Value::Number(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as f64,
                    ))
                },
            }),
        );

        Self { env }
    }

    pub fn interpret(
        &mut self,
        ast: &AST<'a>,
        statements: &[StmtId],
    ) -> Result<(), AmystError<'a>> {
        for stmt_id in statements {
            self.execute(ast, *stmt_id)?;
        }
        Ok(())
    }

    pub(crate) fn execute(&mut self, ast: &AST<'a>, stmt_id: StmtId) -> Result<(), AmystError<'a>> {
        let env = &mut self.env;
        match ast.get_stmt(stmt_id) {
            Stmt::Print(id) => {
                let val = evaluate(ast, self, *id)?;
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
                    return Err(AmystError::Runtime {
                        message: report_error(
                            name.line,
                            Some(&format!(" at '{}'", name.lexeme)),
                            "Variable should have an initial value.",
                        ),
                    });
                    self.env.define(name.lexeme, Value::Unit)
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
                    Value::Range {
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
                                .define(variable.lexeme, Value::Number(i as f64));
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
            Stmt::Function {
                name,
                params,
                body,
                return_type,
            } => env.define(
                name.lexeme,
                Value::Callable(AmystCallable::UserDefined {
                    name: (*name).clone(),
                    params: (*params).clone(),
                    body: *body,
                    return_type: (*return_type).clone(),
                }),
            ),
            Stmt::Return { value, .. } => {
                let val = evaluate(ast, self, *value)?;
                return Err(AmystError::Return(val));
            }
            Stmt::Block(id) => {
                evaluate(ast, self, *id)?;
            }
            Stmt::If(id) => {
                evaluate(ast, self, *id)?;
            }
        }
        Ok(())
    }

    fn stringify(&self, val: Value) -> String {
        match val {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.to_string(),
            Value::Boolean(b) => String::from(if b { "true" } else { "false" }),
            Value::Range {
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
            Value::Unit => String::from("()"),
            Value::Callable(func) => match func {
                AmystCallable::Native { name, .. } => format!("<native fn {name}>"),
                AmystCallable::UserDefined { name, .. } => format!("<fn {}>", name.lexeme),
            },
        }
    }
}

pub fn is_truthy<'a>(val: &Value) -> Result<bool, AmystError<'a>> {
    match val {
        Value::Boolean(b) => Ok(*b),
        _ => {
            let message = String::from("Expected boolean expression");
            Err(AmystError::Runtime { message })
        }
    }
}