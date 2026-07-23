mod expr;
mod stmt;
mod types;

pub use expr::{ExprId, ExprKind};
pub use stmt::{Stmt, StmtId};
pub use types::{AmystType, Param};

use crate::interpreter::{Interpreter, Value, is_truthy};
use crate::lexer::TokenType;
use crate::parser::ensure_int;
use crate::{AmystError, report_error};
use std::borrow::Cow;

pub struct AST<'a> {
    expressions: Vec<ExprKind<'a>>,
    statements: Vec<Stmt<'a>>,
}

impl<'a> AST<'a> {
    pub fn new() -> Self {
        AST {
            expressions: Vec::with_capacity(256),
            statements: Vec::with_capacity(256),
        }
    }

    pub fn add_node(&mut self, kind: ExprKind<'a>) -> ExprId {
        let id = self.expressions.len();
        self.expressions.push(kind);
        id
    }

    pub fn add_stmt(&mut self, stmt: Stmt<'a>) -> StmtId {
        let id = self.statements.len();
        self.statements.push(stmt);
        id
    }

    pub fn get_node(&self, id: ExprId) -> &ExprKind<'a> {
        &self.expressions[id]
    }

    pub fn get_stmt(&self, id: StmtId) -> &Stmt<'a> {
        &self.statements[id]
    }
}

pub(crate) fn evaluate<'a>(
    ast: &AST<'a>,
    interpreter: &mut Interpreter<'a>,
    id: ExprId,
) -> Result<Value<'a>, AmystError<'a>> {
    let node = ast.get_node(id);

    match node {
        ExprKind::Literal(val) => Ok(val.clone()),
        ExprKind::Grouping(child_id) => evaluate(ast, interpreter, *child_id),
        ExprKind::Unary { operator, right } => {
            let right_val = evaluate(ast, interpreter, *right)?;
            match &operator.token_type {
                TokenType::Minus => match right_val {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(AmystError::Runtime {
                        message: format!("Invalid - operand for {:?} literal", right_val),
                    }),
                },
                TokenType::Bang => match right_val {
                    Value::Boolean(b) => Ok(Value::Boolean(!b)),
                    _ => Err(AmystError::Runtime {
                        message: format!("Invalid ! operand for {:?} literal", right_val),
                    }),
                },
                _ => Ok(Value::Unit),
            }
        }
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            let left_val = evaluate(ast, interpreter, *left)?;
            let right_val = evaluate(ast, interpreter, *right)?;
            let operation = &operator.token_type;
            match (left_val, right_val) {
                (Value::Number(n), Value::Number(m)) => match operation {
                    TokenType::Plus => Ok(Value::Number(n + m)),
                    TokenType::Minus => Ok(Value::Number(n - m)),
                    TokenType::Star => Ok(Value::Number(n * m)),
                    TokenType::Slash => Ok(Value::Number(n / m)),
                    TokenType::EqualEqual => Ok(Value::Boolean(n == m)),
                    TokenType::BangEqual => Ok(Value::Boolean(n != m)),
                    TokenType::LessEqual => Ok(Value::Boolean(n <= m)),
                    TokenType::Less => Ok(Value::Boolean(n < m)),
                    TokenType::GreaterEqual => Ok(Value::Boolean(n >= m)),
                    TokenType::Greater => Ok(Value::Boolean(n > m)),
                    _ => {
                        let message =
                            format!("Invalid {:?} token for number binary operation", operation);
                        Err(AmystError::Runtime { message })
                    }
                },
                (Value::String(s), Value::String(t)) => match operation {
                    TokenType::Plus => {
                        let concatenated = format!("{s}{t}");
                        Ok(Value::String(Cow::Owned(concatenated)))
                    }
                    _ => {
                        let message =
                            format!("Invalid {:?} token for string binary operation", operation);
                        Err(AmystError::Runtime { message })
                    }
                },
                _ => {
                    let message = "Incompatible operand types".to_string();
                    Err(AmystError::Runtime { message })
                }
            }
        }
        ExprKind::Variable(tk) => interpreter.env.get(tk),
        ExprKind::Assign { name, value } => {
            let value = evaluate(ast, interpreter, *value)?;
            interpreter.env.assign(name, value)
        }
        ExprKind::Logical {
            left,
            operator,
            right,
        } => {
            let left = evaluate(ast, interpreter, *left)?;
            match operator.token_type {
                TokenType::Or => {
                    if is_truthy(&left)? {
                        return Ok(left);
                    };
                }
                _ => {
                    if !is_truthy(&left)? {
                        return Ok(left);
                    };
                }
            }

            Ok(evaluate(ast, interpreter, *right)?)
        }
        ExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let start_val = evaluate(ast, interpreter, *start)?;
            let end_val = evaluate(ast, interpreter, *end)?;

            match (start_val, end_val) {
                (Value::Number(s), Value::Number(e)) => {
                    let start_int = ensure_int(s)?;
                    let end_int = ensure_int(e)?;
                    Ok(Value::Range {
                        start: start_int,
                        end: end_int,
                        inclusive: *inclusive,
                    })
                }
                _ => Err(AmystError::Runtime {
                    message: "Range operands must be numbers.".to_string(),
                }),
            }
        }
        ExprKind::Call {
            callee,
            paren,
            arguments: expr_args,
        } => {
            let callee_val = evaluate(ast, interpreter, *callee)?;
            let mut arguments = Vec::new();
            for arg in expr_args {
                arguments.push(evaluate(ast, interpreter, *arg)?);
            }

            let function = match callee_val {
                Value::Callable(func) => func,
                _ => {
                    return Err(AmystError::Runtime {
                        message: report_error(
                            paren.line,
                            Some(&format!(" at '{}'", paren.lexeme)),
                            "Can only call functions and classes.",
                        ),
                    });
                }
            };

            if arguments.len() != function.arity() {
                return Err(AmystError::Runtime {
                    message: report_error(
                        paren.line,
                        Some(&format!(" at '{}'", paren.lexeme)),
                        &format!(
                            "Expected {} arguments but got {} instead.",
                            function.arity(),
                            arguments.len()
                        ),
                    ),
                });
            }

            function.call(&arguments, interpreter, ast)
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            if is_truthy(&evaluate(ast, interpreter, *condition)?)? {
                evaluate(ast, interpreter, *then_branch)
            } else if let Some(else_branch) = else_branch {
                evaluate(ast, interpreter, *else_branch)
            } else {
                // Design TODO: if assign statements return value where no else clause
                Ok(Value::Unit)
            }
        }
        ExprKind::Block { stmts, expr } => {
            interpreter.env.push_scope();

            let mut eval_block = || {
                for stmt_id in stmts {
                    interpreter.execute(ast, *stmt_id)?;
                }

                if let Some(expr_id) = expr {
                    evaluate(ast, interpreter, *expr_id)
                } else {
                    Ok(Value::Unit)
                }
            };

            let result = eval_block();
            interpreter.env.pop_scope();

            result
        }
    }
}

pub fn format_ast(ast: &AST, id: ExprId) -> String {
    let node = ast.get_node(id);
    match node {
        ExprKind::Literal(lit) => match lit {
            Value::String(s) => format!("{}", s),
            Value::Number(n) => format!("{}", n),
            Value::Boolean(b) => format!("{}", b),
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    format!("{start}..={end}")
                } else {
                    format!("{start}..{end}")
                }
            }
            Value::Unit => "()".to_string(),
            _ => {
                todo!()
            }
        },
        ExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            if *inclusive {
                format!("{start}..={end}")
            } else {
                format!("{start}..{end}")
            }
        }
        ExprKind::Grouping(id) => {
            format!("(group ({}))", format_ast(ast, *id))
        }
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            format!(
                "({} {} {})",
                operator.lexeme,
                format_ast(ast, *left),
                format_ast(ast, *right)
            )
        }
        ExprKind::Unary { operator, right } => {
            format!("({}{})", operator.lexeme, format_ast(ast, *right))
        }
        ExprKind::Variable(tk) => format!("{}", tk.lexeme),
        ExprKind::Assign { .. } => String::from(""),
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use crate::AmystError;
    use crate::ast::{AST, ExprKind, evaluate};
    use crate::interpreter::{Interpreter, Value};
    use crate::lexer::{Token, TokenType};
    use std::borrow::Cow;

    #[test]
    fn literal_value_expression_has_expected_result<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let id = ast.add_node(ExprKind::Literal(Value::Number(32.0)));
        let id_ev = ast.add_node(ExprKind::Grouping(id));
        let mut env = Interpreter::new();
        assert_eq!(evaluate(&ast, &mut env, id_ev)?, Value::Number(32.0));
        Ok(())
    }

    #[test]
    fn unitary_expression_evaluation_has_expected_result<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let right = ast.add_node(ExprKind::Literal(Value::Number(32.0)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "",
                line: 1,
            },
            right,
        });
        let mut env = Interpreter::new();
        assert_eq!(evaluate(&ast, &mut env, id)?, Value::Number(-32.0));
        let right = ast.add_node(ExprKind::Literal(Value::Boolean(false)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Bang,
                lexeme: "",
                line: 1,
            },
            right,
        });
        assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(true));
        Ok(())
    }

    #[test]
    fn unitary_evaluation_errors_are_displayed<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let mut env = Interpreter::new();
        let right = ast.add_node(ExprKind::Literal(Value::Number(32.0)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Bang,
                lexeme: "",
                line: 1,
            },
            right,
        });
        assert_eq!(
            evaluate(&ast, &mut env, id).err(),
            Some(AmystError::Runtime {
                message: format!("Invalid ! operand for {:?} literal", Value::Number(32.0))
            })
        );
        let right = ast.add_node(ExprKind::Literal(Value::Boolean(false)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "",
                line: 1,
            },
            right,
        });
        assert_eq!(
            evaluate(&ast, &mut env, id).err(),
            Some(AmystError::Runtime {
                message: format!("Invalid - operand for {:?} literal", Value::Boolean(false))
            })
        );
        Ok(())
    }

    #[test]
    fn binary_expression_evaluation_has_expected_result<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let mut env = Interpreter::new();
        let operations = vec![
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::EqualEqual,
            TokenType::BangEqual,
            TokenType::LessEqual,
            TokenType::Less,
            TokenType::GreaterEqual,
            TokenType::Greater,
        ];
        let n = 34.5;
        let m = 67.0;
        let left = ast.add_node(ExprKind::Literal(Value::Number(n)));
        let right = ast.add_node(ExprKind::Literal(Value::Number(m)));

        for op in operations {
            let operator = Token {
                token_type: op.clone(),
                lexeme: "",
                line: 1,
            };
            let id = ast.add_node(ExprKind::Binary {
                left,
                operator,
                right,
            });

            println!("Running {:?} operation", op);

            match op {
                TokenType::Plus => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Number(n + m))
                }
                TokenType::Minus => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Number(n - m))
                }
                TokenType::Star => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Number(n * m))
                }
                TokenType::Slash => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Number(n / m))
                }
                TokenType::EqualEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n == m))
                }
                TokenType::BangEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n != m))
                }
                TokenType::LessEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n <= m))
                }
                TokenType::Less => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n < m))
                }
                TokenType::GreaterEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n >= m))
                }
                TokenType::Greater => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, Value::Boolean(n > m))
                }
                _ => {}
            }
            println!("{:?} operation Ok", op);
        }

        let left = ast.add_node(ExprKind::Literal(Value::String(Cow::from("Hola"))));
        let right = ast.add_node(ExprKind::Literal(Value::String(Cow::from(" Mundo!"))));
        let operator = Token {
            token_type: TokenType::Plus,
            lexeme: "",
            line: 1,
        };
        let id = ast.add_node(ExprKind::Binary {
            left,
            operator,
            right,
        });
        assert_eq!(
            evaluate(&ast, &mut env, id)?,
            Value::String(Cow::from("Hola Mundo!"))
        );
        Ok(())
    }
}
