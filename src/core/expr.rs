use crate::core::callable::{AmystCallable, AmystType, Param};
use crate::core::env::ScopeStack;
use crate::core::interpreter::{Interpreter, is_truthy};
use crate::core::parser::ensure_int;
use crate::core::scanner::{Token, TokenType};
use crate::{AmystError, report_error};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LiteralValue<'a> {
    Number(f64),
    String(Cow<'a, str>),
    Boolean(bool),
    Range {
        start: i32,
        end: i32,
        inclusive: bool,
    },
    Callable(AmystCallable<'a>),
    Unit,
}

pub type ExprId = usize;
pub type StmtId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'a> {
    Binary {
        left: ExprId,
        operator: Token<'a>,
        right: ExprId,
    },
    Call {
        callee: ExprId,
        paren: Token<'a>,
        arguments: Vec<ExprId>,
    },
    Range {
        start: ExprId,
        end: ExprId,
        inclusive: bool,
    },
    Assign {
        name: Token<'a>,
        value: ExprId,
    },
    Logical {
        left: ExprId,
        operator: Token<'a>,
        right: ExprId,
    },
    Unary {
        operator: Token<'a>,
        right: ExprId,
    },
    Grouping(ExprId),
    Literal(LiteralValue<'a>),
    Variable(Token<'a>),
}

pub enum Stmt<'a> {
    Block(Vec<StmtId>),
    Expression(ExprId),
    Function {
        name: Token<'a>,
        params: Vec<Param<'a>>,
        body: StmtId,
        return_type: Option<AmystType>,
    },
    If {
        condition: ExprId,
        then_branch: StmtId,
        else_branch: Option<StmtId>,
    },
    Print(ExprId),
    Return {
        keyword: Token<'a>,
        value: ExprId,
    },
    For {
        variable: Token<'a>,
        iterable: ExprId,
        body: StmtId,
    },
    While {
        condition: ExprId,
        body: StmtId,
    },
    Var {
        name: Token<'a>,
        initializer: Option<ExprId>,
    },
}

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
) -> Result<LiteralValue<'a>, AmystError<'a>> {
    let node = ast.get_node(id);

    match node {
        ExprKind::Literal(val) => Ok(val.clone()),
        ExprKind::Grouping(child_id) => evaluate(ast, interpreter, *child_id),
        ExprKind::Unary { operator, right } => {
            let right_val = evaluate(ast, interpreter, *right)?;
            match &operator.token_type {
                TokenType::Minus => match right_val {
                    LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                    _ => Err(AmystError::Runtime {
                        message: format!("Invalid - operand for {:?} literal", right_val),
                    }),
                },
                TokenType::Bang => match right_val {
                    LiteralValue::Boolean(b) => Ok(LiteralValue::Boolean(!b)),
                    _ => Err(AmystError::Runtime {
                        message: format!("Invalid ! operand for {:?} literal", right_val),
                    }),
                },
                _ => Ok(LiteralValue::Unit),
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
                (LiteralValue::Number(n), LiteralValue::Number(m)) => match operation {
                    TokenType::Plus => Ok(LiteralValue::Number(n + m)),
                    TokenType::Minus => Ok(LiteralValue::Number(n - m)),
                    TokenType::Star => Ok(LiteralValue::Number(n * m)),
                    TokenType::Slash => Ok(LiteralValue::Number(n / m)),
                    TokenType::EqualEqual => Ok(LiteralValue::Boolean(n == m)),
                    TokenType::BangEqual => Ok(LiteralValue::Boolean(n != m)),
                    TokenType::LessEqual => Ok(LiteralValue::Boolean(n <= m)),
                    TokenType::Less => Ok(LiteralValue::Boolean(n < m)),
                    TokenType::GreaterEqual => Ok(LiteralValue::Boolean(n >= m)),
                    TokenType::Greater => Ok(LiteralValue::Boolean(n > m)),
                    _ => {
                        let message =
                            format!("Invalid {:?} token for number binary operation", operation);
                        Err(AmystError::Runtime { message })
                    }
                },
                (LiteralValue::String(s), LiteralValue::String(t)) => match operation {
                    TokenType::Plus => {
                        let concatenated = format!("{s}{t}");
                        Ok(LiteralValue::String(Cow::Owned(concatenated)))
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
                (LiteralValue::Number(s), LiteralValue::Number(e)) => {
                    let start_int = ensure_int(s)?;
                    let end_int = ensure_int(e)?;
                    Ok(LiteralValue::Range {
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
                LiteralValue::Callable(func) => func,
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
    }
}

pub fn format_ast(ast: &AST, id: ExprId) -> String {
    let node = ast.get_node(id);
    match node {
        ExprKind::Literal(lit) => match lit {
            LiteralValue::String(s) => format!("{}", s),
            LiteralValue::Number(n) => format!("{}", n),
            LiteralValue::Boolean(b) => format!("{}", b),
            LiteralValue::Range {
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
            LiteralValue::Unit => "()".to_string(),
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
    use crate::core::env::ScopeStack;
    use crate::core::expr::{AST, ExprKind, LiteralValue, evaluate};
    use crate::core::interpreter::Interpreter;
    use crate::core::scanner::{Token, TokenType};
    use std::borrow::Cow;

    #[test]
    fn literal_value_expression_has_expected_result<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let id = ast.add_node(ExprKind::Literal(LiteralValue::Number(32.0)));
        let id_ev = ast.add_node(ExprKind::Grouping(id));
        let mut env = Interpreter::new();
        assert_eq!(evaluate(&ast, &mut env, id_ev)?, LiteralValue::Number(32.0));
        Ok(())
    }

    #[test]
    fn unitary_expression_evaluation_has_expected_result<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let right = ast.add_node(ExprKind::Literal(LiteralValue::Number(32.0)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: "",
                line: 1,
            },
            right,
        });
        let mut env = Interpreter::new();
        assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Number(-32.0));
        let right = ast.add_node(ExprKind::Literal(LiteralValue::Boolean(false)));
        let id = ast.add_node(ExprKind::Unary {
            operator: Token {
                token_type: TokenType::Bang,
                lexeme: "",
                line: 1,
            },
            right,
        });
        assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(true));
        Ok(())
    }

    #[test]
    fn unitary_evaluation_errors_are_displayed<'a>() -> Result<(), AmystError<'a>> {
        let mut ast = AST::new();
        let mut env = Interpreter::new();
        let right = ast.add_node(ExprKind::Literal(LiteralValue::Number(32.0)));
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
                message: format!(
                    "Invalid ! operand for {:?} literal",
                    LiteralValue::Number(32.0)
                )
            })
        );
        let right = ast.add_node(ExprKind::Literal(LiteralValue::Boolean(false)));
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
                message: format!(
                    "Invalid - operand for {:?} literal",
                    LiteralValue::Boolean(false)
                )
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
        let left = ast.add_node(ExprKind::Literal(LiteralValue::Number(n)));
        let right = ast.add_node(ExprKind::Literal(LiteralValue::Number(m)));

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
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Number(n + m))
                }
                TokenType::Minus => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Number(n - m))
                }
                TokenType::Star => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Number(n * m))
                }
                TokenType::Slash => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Number(n / m))
                }
                TokenType::EqualEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n == m))
                }
                TokenType::BangEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n != m))
                }
                TokenType::LessEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n <= m))
                }
                TokenType::Less => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n < m))
                }
                TokenType::GreaterEqual => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n >= m))
                }
                TokenType::Greater => {
                    assert_eq!(evaluate(&ast, &mut env, id)?, LiteralValue::Boolean(n > m))
                }
                _ => {}
            }
            println!("{:?} operation Ok", op);
        }

        let left = ast.add_node(ExprKind::Literal(LiteralValue::String(Cow::from("Hola"))));
        let right = ast.add_node(ExprKind::Literal(LiteralValue::String(Cow::from(
            " Mundo!",
        ))));
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
            LiteralValue::String(Cow::from("Hola Mundo!"))
        );
        Ok(())
    }
}
