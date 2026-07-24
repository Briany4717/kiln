use crate::interpreter::callable::AmystCallable;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
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


impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f,"{s}"),
            Value::Boolean(b) => {
                let r = if *b {"true"} else {"false"};
                write!(f, "{r}" )
            },
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    write!(f,"{start}..={end}")
                } else {
                    write!(f,"{start}..{end}")
                }
            },
            Value::Unit => write!(f,"()"),
            Value::Callable(func) => match func {
                AmystCallable::Native { name, .. } => write!(f,"<native fn {name}>"),
                AmystCallable::UserDefined { name, .. } => write!(f,"<fn {}>", name.lexeme),
            },

        }
    }
}