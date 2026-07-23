use crate::core::expr::LiteralValue;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum AmystError<'a> {
    FileNotFound,
    InvalidFormatFile,
    UnexpectedCharacter(usize),
    UnterminatedString,
    InvalidNumberFormat,
    Runtime { message: String },
    Return(LiteralValue<'a>),
    Multiple(Vec<AmystError<'a>>),
}

impl Display for AmystError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmystError::FileNotFound => write!(f, "File not found."),
            AmystError::InvalidFormatFile => write!(f, "Invalid format file."),
            AmystError::UnexpectedCharacter(line) => {
                write!(f, "{}", report_error(*line, None, "Unexpected character."))
            }
            AmystError::UnterminatedString => write!(f, "Unterminated string."),
            AmystError::InvalidNumberFormat => write!(f, "Invalid number format."),
            AmystError::Runtime { message } => write!(f, "{message}"),
            AmystError::Return(_) => write!(f, "Unexpected 'return' statement."),
            AmystError::Multiple(errors) => {
                for err in errors {
                    writeln!(f, "{err}")?;
                }
                write!(f, "Multiple errors found during execution.")
            }
        }
    }
}

impl std::error::Error for AmystError<'_> {}

pub fn report_error(line: usize, _where: Option<&str>, message: &str) -> String {
    if let Some(location) = _where {
        format!("[line {line}] Error {location}: {message}")
    } else {
        format!("[line {line}] Error: {message}")
    }
}
