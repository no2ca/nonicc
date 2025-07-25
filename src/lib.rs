use std::process::exit;

pub fn error_at(input: &str, pos: usize, e: anyhow::Error) {
    eprintln!("{}", input);
    eprint!("{}", " ".repeat(pos));
    eprint!("^ ");
    eprintln!("{}", e);
    exit(1);
}

pub fn starts_with_in(input: &str, patterns: &[&str]) -> Option<usize> {
    for i in 0..patterns.len() {
        if input.starts_with(patterns[i]) {
            return Some(i);
        }
    };
    None
}