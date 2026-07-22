use crate::KilnError;
use crate::core::env::ScopeStack;
use crate::core::scanner::{Token, TokenType};
use std::borrow::Cow;
use crate::core::interpreter::is_truthy;

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue<'a> {
    Number(f64),
    String(Cow<'a, str>),
    Boolean(bool),
    Nil,
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
    Assign {
        name: Token<'a>,
        value: ExprId,
    },
    Logical {
        left: ExprId,
        operator: Token<'a>,
        right: ExprId
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
    If {
        condition: ExprId,
        then_branch: StmtId,
        else_branch: Option<StmtId>
    },
    Print(ExprId),
    While{
        condition: ExprId,
        body: StmtId
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
    env: &mut ScopeStack<'a>,
    id: ExprId,
) -> Result<LiteralValue<'a>, KilnError> {
    let node = ast.get_node(id);

    match node {
        ExprKind::Literal(val) => Ok(val.clone()),
        ExprKind::Grouping(child_id) => evaluate(ast, env, *child_id),
        ExprKind::Unary { operator, right } => {
            let right_val = evaluate(ast, env, *right)?;
            match &operator.token_type {
                TokenType::Minus => match right_val {
                    LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                    _ => Err(KilnError::Runtime {
                        message: format!("Invalid - operand for {:?} literal", right_val),
                    }),
                },
                TokenType::Bang => match right_val {
                    LiteralValue::Boolean(b) => Ok(LiteralValue::Boolean(!b)),
                    _ => Err(KilnError::Runtime {
                        message: format!("Invalid ! operand for {:?} literal", right_val),
                    }),
                },
                _ => Ok(LiteralValue::Nil),
            }
        }
        ExprKind::Binary {
            left,
            operator,
            right,
        } => {
            let left_val = evaluate(ast, env, *left)?;
            let right_val = evaluate(ast, env, *right)?;
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
                        Err(KilnError::Runtime { message })
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
                        Err(KilnError::Runtime { message })
                    }
                },
                _ => {
                    let message = "Incompatible operand types".to_string();
                    Err(KilnError::Runtime { message })
                }
            }
        }
        ExprKind::Variable(tk) => env.get(tk),
        ExprKind::Assign { name, value } => {
            let value = evaluate(ast, env, *value)?;
            env.assign(name, value)
        }
        ExprKind::Logical {left,operator,right} => {
            let left = evaluate(ast,env,*left)?;
            match operator.token_type {
                TokenType::Or => {
                    if is_truthy(&left)? { return Ok(left) };
                }
                _ => {
                    if !is_truthy(&left)? {return Ok(left)};
                }
            }
            
            Ok(evaluate(ast,env,*right)?)
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
            LiteralValue::Nil => "Nil".to_string(),
        },
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
        ExprKind::Logical { .. } => todo!(),
    }
}

#[cfg(test)]
mod test {
    use crate::KilnError;
    use crate::core::env::ScopeStack;
    use crate::core::expr::{AST, ExprKind, LiteralValue, evaluate};
    use crate::core::scanner::{Token, TokenType};
    use std::borrow::Cow;

    #[test]
    fn literal_value_expression_has_expected_result() -> Result<(), KilnError> {
        let mut ast = AST::new();
        let id = ast.add_node(ExprKind::Literal(LiteralValue::Number(32.0)));
        let id_ev = ast.add_node(ExprKind::Grouping(id));
        let mut env = ScopeStack::new();
        assert_eq!(evaluate(&ast, &mut env, id_ev)?, LiteralValue::Number(32.0));
        Ok(())
    }

    #[test]
    fn unitary_expression_evaluation_has_expected_result() -> Result<(), KilnError> {
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
        let mut env = ScopeStack::new();
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
    fn unitary_evaluation_errors_are_displayed() -> Result<(), KilnError> {
        let mut ast = AST::new();
        let mut env = ScopeStack::new();
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
            Some(KilnError::Runtime {
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
            Some(KilnError::Runtime {
                message: format!(
                    "Invalid - operand for {:?} literal",
                    LiteralValue::Boolean(false)
                )
            })
        );
        Ok(())
    }

    #[test]
    fn binary_expression_evaluation_has_expected_result() -> Result<(), KilnError> {
        let mut ast = AST::new();
        let mut env = ScopeStack::new();
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
