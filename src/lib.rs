mod tokenizer;

use std::process::exit;
pub use tokenizer::token::{ TokenKind, Token };
pub use tokenizer::tokenizer::{ Tokenizer };

pub fn error_at(input: &str, pos: usize, e: anyhow::Error) {
    eprintln!("{}", input);
    eprint!("{}", " ".repeat(pos));
    eprint!("^ ");
    eprintln!("{}", e);
    exit(1);
}
