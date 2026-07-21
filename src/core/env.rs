use crate::core::expr::LiteralValue;
use crate::core::scanner::Token;
use crate::{KilnError, report_error};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Env<'a> {
    values: HashMap<&'a str, LiteralValue<'a>>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &'a str, val: LiteralValue<'a>) {
        self.values.insert(name, val);
    }

    pub(crate) fn get(&self, name: &Token<'a>) -> Result<LiteralValue<'a>, KilnError> {
        if let Some(val) = self.values.get(name.lexeme) {
            Ok(val.clone())
        } else {
            Err(KilnError::Runtime {
                message: report_error(
                    name.line,
                    Some(&format!(" at '{}'", name.lexeme)),
                    &format!("Trying to use undefined variable '{}'.", name.lexeme),
                ),
            })
        }
    }

    pub(crate) fn assign(
        &mut self,
        tk: &Token<'a>,
        val: LiteralValue<'a>,
    ) -> Result<LiteralValue<'a>, KilnError> {
        let name = tk.lexeme;
        if self.values.contains_key(name) {
            self.values.insert(name, val.clone());
            Ok(val)
        } else {
            Err(KilnError::Runtime {
                message: report_error(
                    tk.line,
                    Some(&format!(" at '{}'", name)),
                    &format!("Undefined variable '{}'.", tk.lexeme),
                ),
            })
        }
    }
}
