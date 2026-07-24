use crate::AmystError;
use crate::ast::{AST, AmystType, ExprId, Param, evaluate};
use crate::interpreter::{Interpreter, Value};
use crate::lexer::Token;

#[derive(Clone, Debug)]
pub enum AmystCallable<'a> {
    Native {
        arity: usize,
        name: &'a str,
        func: fn(&[Value<'a>]) -> Result<Value<'a>, AmystError<'a>>,
    },
    UserDefined {
        name: Token<'a>,
        params: Vec<Param<'a>>,
        body: ExprId,
        return_type: Option<AmystType>,
    },
}

impl<'a> AmystCallable<'a> {
    pub(crate) fn call(
        &self,
        args: &[Value<'a>],
        interpreter: &mut Interpreter<'a>,
        ast: &AST<'a>,
    ) -> Result<Value<'a>, AmystError<'a>> {
        match self {
            AmystCallable::Native { func, .. } => func(args),
            AmystCallable::UserDefined { params, body, .. } => {
                interpreter.env.push_scope();
                for i in 0..params.len() {
                    interpreter
                        .env
                        .define(params[i].name.lexeme, args[i].clone())
                }
                let res = evaluate(ast, interpreter, *body);
                interpreter.env.pop_scope();
                match res {
                    Ok(val) => Ok(val),
                    Err(AmystError::Return(val)) => Ok(val),
                    Err(err) => Err(err),
                }
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

impl<'a> PartialEq for AmystCallable<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Native { arity, name, .. },
                Self::Native { arity: o_arity, name: o_name, .. },
            ) => arity == o_arity && name == o_name,
            (
                Self::UserDefined { name, params, body, return_type },
                Self::UserDefined { name: o_name, params: o_params, body: o_body, return_type: o_return_type },
            ) => name == o_name && params == o_params && body == o_body && return_type == o_return_type,
            _ => false,
        }
    }
}