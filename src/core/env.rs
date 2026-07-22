use crate::core::expr::LiteralValue;
use crate::core::scanner::Token;
use crate::{KilnError, report_error};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScopeStack<'a> {
    scopes: Vec<HashMap<&'a str, LiteralValue<'a>>>,
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

    pub fn define_global(&mut self, name: &'a str, val: LiteralValue<'a>) {
        if let Some(current_scope) = self.scopes.first_mut() {
            current_scope.insert(name, val);
        }
    }
    
    pub fn define(&mut self, name: &'a str, val: LiteralValue<'a>) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, val);
        }
    }

    pub(crate) fn get(&self, name: &Token<'a>) -> Result<LiteralValue<'a>, KilnError> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name.lexeme) {
                return Ok(val.clone());
            }
        }

        Err(KilnError::Runtime {
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
        val: LiteralValue<'a>,
    ) -> Result<LiteralValue<'a>, KilnError> {
        let name = tk.lexeme;

        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name, val.clone());
                return Ok(val);
            }
        }

        Err(KilnError::Runtime {
            message: report_error(
                tk.line,
                Some(&format!(" at '{}'", name)),
                &format!("Undefined variable '{}'.", name),
            ),
        })
    }
}
