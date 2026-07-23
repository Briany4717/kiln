use crate::interpreter::Value;
use crate::lexer::Token;
use crate::{AmystError, report_error};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScopeStack<'a> {
    scopes: Vec<HashMap<&'a str, Value<'a>>>,
}

impl<'a> ScopeStack<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::with_capacity(16)],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::with_capacity(8));
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define_global(&mut self, name: &'a str, val: Value<'a>) {
        if let Some(current_scope) = self.scopes.first_mut() {
            current_scope.insert(name, val);
        }
    }

    pub fn define(&mut self, name: &'a str, val: Value<'a>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, val);
        }
    }

    pub(crate) fn get(&self, name: &Token<'a>) -> Result<Value<'a>, AmystError<'a>> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name.lexeme) {
                return Ok(val.clone());
            }
        }

        Err(AmystError::Runtime {
            message: report_error(
                name.line,
                Some(&format!(" at '{}'", name.lexeme)),
                &format!("Undefined variable '{}'.", name.lexeme),
            ),
        })
    }

    pub(crate) fn assign(
        &mut self,
        tk: &Token<'a>,
        val: Value<'a>,
    ) -> Result<Value<'a>, AmystError<'a>> {
        let name = tk.lexeme;

        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name, val.clone());
                return Ok(val);
            }
        }

        Err(AmystError::Runtime {
            message: report_error(
                tk.line,
                Some(&format!(" at '{}'", name)),
                &format!("Undefined variable '{}'.", name),
            ),
        })
    }
}
