use crate::lexer::Token;

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
