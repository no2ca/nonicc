#![allow(non_camel_case_types)]

use std::env;
use std::process::exit;

#[derive(PartialEq, Clone, Debug)]
enum TokenKind{
    TK_RESERVED,
    TK_NUM,
    TK_EOF,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    val: Option<i32>,   // WARNING: この大きさでいいのか？
    str: String,
}

impl Token {
    // 新しいトークンを返す
    fn create_next (kind: TokenKind, str: String) -> Self {
        let tok = Box::new(
            Token {
                kind: kind,
                str: str,
                val: None,
            }
        );
        *tok 
    }
}

#[derive(Debug)]
struct CurrentToken {
    tok_vec: Vec<Box<Token>>,
    idx: usize,
}

impl CurrentToken {
    fn consume(&mut self, op: &char) -> bool {
        let tok = *self.tok_vec[self.idx].clone();
        if tok.kind != TokenKind::TK_RESERVED
            || tok.str.chars().nth(0) != Some(*op) {
                false
            } else {
                self.idx += 1;
                true
            }
    }

    fn expect(&mut self, op: &char) {
        let tok = *self.tok_vec[self.idx].clone();
        if tok.kind != TokenKind::TK_RESERVED
            || tok.str.chars().nth(0) != Some(*op) {
                eprintln!("'{}'ではありません", op)
            } else {
                self.idx += 1;
            }
    }

    fn expect_number(&mut self) -> i32 {
        let tok = *self.tok_vec[self.idx].clone();
        if tok.kind != TokenKind::TK_NUM {
            eprintln!("Error: 数ではありません");
            exit(1);    // WARNING: ここで止まるのでいいのか？
        } else {
            let val = tok.val.unwrap();
            self.idx += 1;
            val
        }
    }

    fn at_eof(&self) -> bool {
        let tok = *self.tok_vec[self.idx].clone();
        return tok.kind == TokenKind::TK_EOF;
    }
}

fn tokenize(input: &str) -> Vec<Box<Token>> {
    let mut chars = input.chars().peekable();
    let head = Box::new(
        Token {
            kind: TokenKind::TK_RESERVED,
            val: None,
            str: String::from("<HEAD>"),
        }
    );

    let mut tok_vec =  vec![head];

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        } 

        let next = if "+-".contains(c) {
            let nxt = Token::create_next(TokenKind::TK_RESERVED, c.to_string());
            Ok(Box::new(nxt))
        } else if c.is_ascii_digit() {
            let mut number = c.to_string();
            // peekで次の値の参照が得られる限り
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    number.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            let mut nxt = Token::create_next(TokenKind::TK_NUM, number.clone()); 
            nxt.val = Some(number.parse::<i32>().unwrap());
            Ok(Box::new(nxt))
        } else {
            Err("トークナイズできません")
        };
        match next {
            Ok(next) => tok_vec.push(next),
            Err(e) => eprintln!("Error: {}", e)
        }
    }
    let eof = Token::create_next(TokenKind::TK_EOF, String::from("EOF"));
    tok_vec.push(Box::new(eof));
    tok_vec
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }

    let tok_vec = tokenize(&args[1]);
    let mut tok = 
    CurrentToken {
        tok_vec: tok_vec,
        idx: 1
    };

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", &tok.expect_number());
    
    while !tok.at_eof() {
        if tok.consume(&'+') {
            println!("  add rax, {}", tok.expect_number());
            continue;
        }
        tok.expect(&'-');
        println!("  sub rax, {}", tok.expect_number());
    }
    println!("  ret");
}
