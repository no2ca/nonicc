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
    next: Option<Box<Token>>,
    val: Option<i32>,   // WARNING: この大きさでいいのか？
    str: String,
}

impl Token {
    // 現在のトークンに新しいトークンのポインタをつなげる
    // 新しいトークンを返す
    fn create_next (&mut self, kind: TokenKind, str: String) -> Self {
        let tok = Box::new(
            Token {
                kind: kind,
                next: None,
                str: str,
                val: None,
            }
        );
        self.next = Some(tok.clone());
        *tok // TODO: 直前で作っているので危険ではないがいずれ修正する
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
            eprintln!("数ではありません");
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
            next: None,
            val: None,
            str: String::from("HEAD"),
        }
    );

    let mut tok_vec =  vec![head];
    let mut next;

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        } 
        next = if "+-".contains(c) {
            Ok(Box::new(tok_vec.last_mut().unwrap().create_next(TokenKind::TK_RESERVED, c.to_string())))
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
            let tmp = number.clone();
            let mut nxt = Box::new(tok_vec.last_mut().unwrap().create_next(TokenKind::TK_NUM, tmp)); 
            nxt.val = Some(number.parse::<i32>().unwrap());
            Ok(nxt)
        } else {
            Err("トークナイズできません")
        };
        match next {
            Ok(next) => tok_vec.push(next),
            Err(e) => eprintln!("Error: {}", e)
        }
    }
    let eof = tok_vec.last_mut().unwrap().create_next(TokenKind::TK_EOF, String::from("EOF"));
    tok_vec.push(Box::new(eof));
    for tok in &tok_vec {
        println!("{:?}", tok);
    }
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

    for i in &tok.tok_vec {
        println!("[DEBUG] tok_vec: {:?}", i);
    }
    
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", &tok.expect_number());
    // println!("token at 'main': {:?}", &tok.tok_vec);
    
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
