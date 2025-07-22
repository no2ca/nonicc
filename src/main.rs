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

struct CurrentToken {
    token: Token,
}

impl CurrentToken {
    fn consume(mut self, op: &char) -> bool {
        if self.token.kind != TokenKind::TK_RESERVED
            || self.token.str.chars().nth(0) != Some(*op) {
                false
            } else {
                self.token = *self.token.next.unwrap();
                true
            }
    }

    fn expect(mut self, op: &char) {
        if self.token.kind != TokenKind::TK_RESERVED
            || self.token.str.chars().nth(0) != Some(*op) {
                eprintln!("'{}'ではありません", op)
            } else {
                self.token = *self.token.next.unwrap();
            }
    }

    fn expect_number(mut self) -> i32 {
        if self.token.kind != TokenKind::TK_NUM {
            eprintln!("数ではありません");
            exit(1);    // WARNING: ここで止まるのでいいのか？
        } else {
            // FIXME: unwrapをなるべく使わない！
            let val = self.token.val.unwrap();
            self.token = *self.token.next.unwrap();
            val
        }
    }

    fn at_eof(&self) -> bool {
        return self.token.kind == TokenKind::TK_EOF;
    }
}

fn tokenize(input: &str) -> Box<Token> {
    let mut chars = input.chars().peekable();
    let mut head = Box::new(
        Token {
            kind: TokenKind::TK_RESERVED,
            next: None,
            val: None,
            str: String::new(),
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
    cur[0].clone().next.unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }
    
    let _ = tokenize(&args[1]);
}
