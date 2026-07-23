use amyst_core::AmystError;
use amyst_core::ast::AST;
use amyst_core::interpreter::Interpreter;
use amyst_core::lexer::Scanner;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs an Amyst script
    Run {
        /// Script to be executed
        script: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { script } => {
            if let Err(err_message) = run_file(script) {
                eprintln!("{err_message}");
                std::process::exit(70);
            }
        }
    }
}

fn run_file(file: &str) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|_| "File not found.".to_string())?;
    let file_text = String::from_utf8(bytes).map_err(|_| "Invalid format file.".to_string())?;

    run(&file_text).map_err(|e| e.to_string())
}

fn run(file: &'_ str) -> Result<(), AmystError<'_>> {
    let scanner = Scanner::new(file);
    let tokens = scanner.scan_tokens()?;
    let mut parser = amyst_core::parser::Parser::new(tokens);
    let mut ast = AST::new();
    let stmts = parser.parse(&mut ast)?;
    let mut interpreter = Interpreter::new();

    interpreter.interpret(&ast, &stmts)?;
    Ok(())
}
