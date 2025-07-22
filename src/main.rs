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
    token: Token,
}

impl CurrentToken {
    fn consume(&mut self, op: &char) -> bool {
        println!("token at 'consume': {:?}", self.token);
        if self.token.kind != TokenKind::TK_RESERVED
            || self.token.str.chars().nth(0) != Some(*op) {
                false
            } else {
                self.token = *self.token.next.clone().unwrap();
                true
            }
    }

    fn expect(&mut self, op: &char) {
        if self.token.kind != TokenKind::TK_RESERVED
            || self.token.str.chars().nth(0) != Some(*op) {
                eprintln!("'{}'ではありません", op)
            } else {
                self.token = *self.token.next.clone().unwrap();
            }
    }

    fn expect_number(&mut self) -> i32 {
        println!("token at 'expect_number': {:?}", self.token);
        if self.token.kind != TokenKind::TK_NUM {
            eprintln!("数ではありません");
            exit(1);    // WARNING: ここで止まるのでいいのか？
        } else {
            // FIXME: unwrapをなるべく使わない！
            let val = self.token.val.unwrap();
            self.token = *self.token.next.clone().unwrap();
            println!("token at 'expect_number': {:?}", self.token);
            val
        }
    }

    fn at_eof(&self) -> bool {
        return self.token.kind == TokenKind::TK_EOF;
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

    let mut cur =  vec![head];
    let mut next;

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        } 
        next = if "+-".contains(c) {
            Ok(Box::new(cur.last_mut().unwrap().create_next(TokenKind::TK_RESERVED, c.to_string())))
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
            let mut nxt = Box::new(cur.last_mut().unwrap().create_next(TokenKind::TK_NUM, tmp)); 
            nxt.val = Some(number.parse::<i32>().unwrap());
            Ok(nxt)
        } else {
            Err("トークナイズできません")
        };
        match next {
            Ok(next) => cur.push(next),
            Err(e) => eprintln!("Error: {}", e)
        }
    }
    let eof = cur.last_mut().unwrap().create_next(TokenKind::TK_EOF, String::from("EOF"));
    cur.push(Box::new(eof));
    for tok in &cur {
        println!("{:?}", tok);
    }
    cur
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }

    let tok_vec = tokenize(&args[1]);
    let mut tok = CurrentToken {token: *tok_vec[1].clone()};
    for i in &tok_vec {
        println!("[DEBUG] tok_vec: {:?}", i);
    }
    
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("  mov rax, {}", &tok.expect_number());
    println!("token at 'main': {:?}", &tok.token);
    
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
