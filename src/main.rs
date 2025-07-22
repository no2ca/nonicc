#![allow(non_camel_case_types)]

use std::env;
use std::process::exit;

#[derive(PartialEq)]
enum TokenKind{
    TK_RESERVED,
    TK_NUM,
    TK_EOF,
}

struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>,
    val: Option<i32>,   // WARNING: この大きさでいいのか？
    str: String,
}

impl Token {
    // 現在のトークンに新しいトークンのポインタをつなげる
    // 新しいトークンを返す
    fn create_next (mut self, kind: TokenKind, str: String) -> Self {
        let tok = Box::new(
            Token {
                kind: kind,
                next: None,
                str: str,
                val: None,
            }
        );
        self.next = Some(tok);
        *self.next.unwrap() // TODO: 直前で作っているので危険ではないがいずれ修正する
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

fn tokenize() {

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }
}
