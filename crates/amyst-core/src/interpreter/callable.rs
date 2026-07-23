use crate::AmystError;
use crate::ast::{AST, ExprId, evaluate, Param, AmystType};
use crate::interpreter::{Interpreter, Value};
use crate::lexer::Token;

#[derive(Clone, PartialEq, Debug)]
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
