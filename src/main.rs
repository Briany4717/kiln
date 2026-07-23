pub mod core;

use crate::core::expr::{AST, LiteralValue};
use crate::core::interpreter::Interpreter;
use clap::Parser;
use core::Scanner;
use std::fmt::Display;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Script to run")]
    script: String,
}

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

fn main() {
    let args = Args::parse();

    if args.script.is_empty() {
        println!("Usage: kiln [script]");
        return;
    }

    if let Err(err_message) = run_file(&args.script) {
        eprintln!("{err_message}");
        std::process::exit(70);
    }
}

fn run_file(file: &str) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|_| "File not found.".to_string())?;
    let file_text = String::from_utf8(bytes).map_err(|_| "Invalid format file.".to_string())?;

    run(&file_text).map_err(|e| e.to_string())
}

fn run(file: &str) -> Result<(), AmystError> {
    let scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens()?;
    let mut parser = crate::core::Parser::new(tokens);
    let mut ast = AST::new();
    let stmts = parser.parse(&mut ast)?;
    let mut interpreter = Interpreter::new();

    interpreter.interpret(&ast, &stmts)?;
    Ok(())
}

fn report_error(line: usize, _where: Option<&str>, message: &str) -> String {
    if let Some(location) = _where {
        format!("[line {line}] Error {location}: {message}")
    } else {
        format!("[line {line}] Error: {message}")
    }
}
