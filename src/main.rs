pub mod core;

use crate::core::interpreter::Interpreter;
use clap::Parser;
use core::Scanner;
use std::fmt::Display;
use crate::core::expr::AST;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Script to run")]
    script: String,
}

#[derive(Debug, PartialEq)]
enum AmystError {
    FileNotFound,
    InvalidFormatFile,
    UnexpectedCharacter(usize),
    UnterminatedString,
    InvalidNumberFormat,
    Runtime { message: String },
    Multiple(Vec<AmystError>),
}

impl Display for AmystError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmystError::FileNotFound => write!(f, "File not found."),
            AmystError::InvalidFormatFile => write!(f, "Invalid format file."),
            AmystError::UnexpectedCharacter(line) => write!(
                f,
                "{}",
                report_error(*line, None, "Unexpected character.").as_str()
            ),
            AmystError::UnterminatedString => write!(f, "Unterminated string."),
            AmystError::InvalidNumberFormat => write!(f, "Invalid number format."),
            AmystError::Runtime { message } => write!(f, "{}", message),
            AmystError::Multiple(errors) => {
                for err in errors {
                    writeln!(f, "{err}")?;
                }
                write!(f, "Multiple errors found during execution.")
            }
        }
    }
}

fn main() -> Result<(), Box<AmystError>> {
    let args = Args::parse();

    if args.script == "" {
        println!("Usage: kiln [script]");
        return Ok(());
    } else {
        match run_file(&args.script) {
            Ok(()) => {}
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}

fn run_file(file: &str) -> Result<(), AmystError> {
    let bytes = std::fs::read(file).map_err(|_| AmystError::FileNotFound)?;
    let file_text = &String::from_utf8(bytes).map_err(|_| AmystError::InvalidFormatFile)?;
    run(file_text)
}

fn run(file: &str) -> Result<(), AmystError> {
    let scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens().map_err(|e| AmystError::from(e))?;
    let mut parser = crate::core::Parser::new(&*tokens);
    let mut ast = AST::new();
    let stmts = parser.parse(&mut ast)?;
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast, &*stmts)?;
    Ok(())
}

fn report_error(line: usize, _where: Option<&str>, message: &str) -> String {
    if let Some(location) = _where {
        format!("[line {}] Error {}: {}", line, location, message).to_string()
    } else {
        format!("[line {}] Error: {}", line, message).to_string()
    }
}
