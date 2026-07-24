use crate::ast::{AmystType, ExprId, Param};
use crate::lexer::Token;

pub type StmtId = usize;

#[derive(Debug)]
pub enum Stmt<'a> {
    Block(ExprId),
    Expression(ExprId),
    Function {
        name: Token<'a>,
        params: Vec<Param<'a>>,
        body: ExprId,
        return_type: Option<AmystType>,
    },
    If(ExprId),
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
        body: ExprId,
    },
    Var {
        name: Token<'a>,
        initializer: Option<ExprId>,
    },
}
