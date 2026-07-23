//! `amyst-core`: el motor del lenguaje Amyst (lexer, parser, AST, interpreter).
//!
//! Este crate es una librería pura — no depende de `clap` ni de nada relacionado
//! con la CLI. Es lo que consumen `amyst-cli`, y en el futuro `amyst-lsp` y
//! `amyst-embed`.

mod diagnostics;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod interpreter;
mod engine;
pub mod prelude;

pub use diagnostics::{AmystError, report_error};
pub use engine::Engine;
