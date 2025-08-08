use std::process::exit;

pub mod types;
pub mod parser;
pub mod lexer;
pub mod codegen;
pub mod ir;
pub mod gen_x64;
pub mod reg_alloc;

pub fn error_at(input: &str, pos: usize, e: anyhow::Error) {
    eprintln!("{}", input);
    eprint!("{}", " ".repeat(pos));
    eprint!("^ ");
    eprintln!("{}", e);
    exit(1);
}
