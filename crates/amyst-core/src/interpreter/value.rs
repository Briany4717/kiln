use std::borrow::Cow;
use crate::interpreter::callable::AmystCallable;

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