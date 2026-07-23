use crate::lexer::{Token};
use crate::ast::{StmtId};
use crate::interpreter::Value;

pub type ExprId = usize;

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
    Literal(Value<'a>),
    Variable(Token<'a>),
    Block {
        stmts: Vec<StmtId>,
        expr: Option<ExprId>,
    },
    If {
        condition: ExprId,
        then_branch: ExprId,
        else_branch: Option<ExprId>,
    },
}
