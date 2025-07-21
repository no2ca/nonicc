#![allow(non_camel_case_types)]

use std::{clone, env};
use std::process::exit;
use std::rc::Rc;

enum TokenKind{
    TK_RESERVED,
    TK_NUM,
    TK_EOF,
}

struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>,
    val: Option<i32>,   // この大きさでいいのか？
    str: String,
}

impl Token {
    // 現在のトークンに新しいトークンのポインタをつなげる
    // 新しいトークンを返す
    fn new (mut self, kind: TokenKind, str: String) -> Self {
        let tok = Box::new(
            Token {
                kind: kind,
                next: None,
                str: str,
                val: None,
            }
        );
        self.next = Some(tok);
        *self.next.unwrap()
    }
}

struct CurrentToken {
    token: Token,
}

impl CurrentToken {
    fn consume(&self, op: &str) {
        
    }

    fn expect(&self, op: &str) {
        
    }

    fn expect_number(&self) {

    }

    fn at_eof(&self) {
        
    }
}

fn tokenize() {

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: incorrect number of arguments");
        exit(1);
    }
}
