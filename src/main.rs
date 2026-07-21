pub mod core;

use clap::{Error, Parser};
use core::Scanner;
use std::fmt::{Display, write};
use crate::core::AST;
use crate::core::expr::format_ast;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "Script to run")]
    script: String,
}

#[derive(Debug, PartialEq)]
enum KilnError {
    FileNotFound,
    InvalidFormatFile,
    UnexpectedCharacter(usize),
    UnterminatedString,
    InvalidNumberFormat,
    Runtime { message: String },
}

impl Display for KilnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KilnError::FileNotFound => write!(f, "File not found."),
            KilnError::InvalidFormatFile => write!(f, "Invalid format file."),
            KilnError::UnexpectedCharacter(line) => write!(
                f,
                "{}",
                report_error(*line, None, "Unexpected character.").as_str()
            ),
            KilnError::UnterminatedString => write!(f, "Unterminated string."),
            KilnError::InvalidNumberFormat => write!(f, "Invalid number format."),
            KilnError::Runtime { message } => write!(f, "{}", message),
        }
    }
}

fn main() -> Result<(), Box<KilnError>> {
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

fn run_file(file: &str) -> Result<(), KilnError> {
    let bytes = std::fs::read(file).map_err(|_| KilnError::FileNotFound)?;
    let file_text = &String::from_utf8(bytes).map_err(|_| KilnError::InvalidFormatFile)?;
    run(file_text)
}

fn run(file: &str) -> Result<(), KilnError> {
    let scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens().map_err(|e| KilnError::from(e))?;
    let mut parser = crate::core::Parser::new(&*tokens);
    let mut ast = AST::new();
    let root = parser.parse(&mut ast)?;

    println!("{}",format_ast(&ast, root));
    Ok(())
}

fn report_error(line: usize, _where: Option<&str>, message: &str) -> String {
    if let Some(location) = _where {
        format!("[line {}] Error {}: {}", line, location, message).to_string()
    } else {
        format!("[line {}] Error: {}", line, message).to_string()
    }
}
