use crate::core::expr::{LiteralValue, StmtId};
use crate::core::scanner::Token;
use crate::KilnError;

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum KilnCallable<'a> {
    Native {
        arity: usize,
        name: &'a str,
        func: fn(&[LiteralValue<'a>]) -> Result<LiteralValue<'a>, KilnError>,
    },
    UserDefined {
        name: Token<'a>,
        params: Vec<Token<'a>>,
        body: StmtId,
    },
}

impl<'a> KilnCallable<'a> {
    pub(crate) fn call(&self, args: &[LiteralValue<'a>]) -> Result<LiteralValue<'a>, KilnError>{
        match self {
            KilnCallable::Native {func, ..} => {
                func(args)
            }
            KilnCallable::UserDefined {..} => {
                todo!()
            }
        }
    }
    pub fn arity(&self) -> usize {
        match self {
            KilnCallable::Native { arity, .. } => *arity,
            KilnCallable::UserDefined { params, .. } => params.len(),
        }
    }
}