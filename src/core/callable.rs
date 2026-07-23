use crate::core::env::ScopeStack;
use crate::core::expr::{LiteralValue, StmtId, AST};
use crate::core::interpreter::Interpreter;
use crate::core::scanner::Token;
use crate::AmystError;

#[derive(Clone, PartialEq, Debug)]
pub enum AmystType {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Named(String),
    Option(Box<AmystType>),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Param<'a> {
    pub name: Token<'a>,
    pub is_mut: bool,
    pub type_annotation: Option<AmystType>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum AmystCallable<'a> {
    Native {
        arity: usize,
        name: &'a str,
        func: fn(&[LiteralValue<'a>]) -> Result<LiteralValue<'a>, AmystError>,
    },
    UserDefined {
        name: Token<'a>,
        params: Vec<Param<'a>>,
        body: StmtId,
        return_type: Option<AmystType>
    },
}

impl<'a> AmystCallable<'a> {
    pub(crate) fn call(&self, args: &[LiteralValue<'a>], interpreter: &mut Interpreter<'a>, ast: &AST<'a>) -> Result<LiteralValue<'a>, AmystError>{
        match self {
            AmystCallable::Native {func, ..} => {
                func(args)
            }
            AmystCallable::UserDefined { params, body, ..} => {
                interpreter.env.push_scope();
                for i in 0..params.len() {
                    interpreter.env.define(params[i].name.lexeme,args[i].clone())
                }
                let res =interpreter.execute(ast, *body);
                interpreter.env.pop_scope();
                res?;
                Ok(LiteralValue::Nil)
            }
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            AmystCallable::Native { arity, .. } => *arity,
            AmystCallable::UserDefined { params, .. } => params.len(),
        }
    }
}