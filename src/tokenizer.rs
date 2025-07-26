#![allow(non_camel_case_types)]

pub mod token {
    #[derive(PartialEq, Clone, Debug)]
    pub enum TokenKind {
        TK_RESERVED, // 記号
        TK_IDENT,    // 変数名の識別子
        TK_NUM,      // 整数
        TK_EOF,      // 入力の終わり
    }

    #[derive(Clone, Debug)]
    pub struct Token {
        pub kind: TokenKind,
        pub val: Option<i32>, // WARNING: この大きさでいいのか？
        pub str: String,
        pub len: usize,
        pub pos: usize,
    }

    impl Token {
        pub fn new(kind: TokenKind, str: String, len: usize, pos: usize) -> Token {
            Token {
                kind,
                str,
                val: None,
                len,
                pos,
            }
        }
    }
}

pub mod tokenizer {
    use crate::tokenizer::token::{ TokenKind, Token };
    use crate::error_at;
    use anyhow::anyhow;

    fn starts_with_in(input: &str, patterns: &[&str]) -> Option<usize> {
        for i in 0..patterns.len() {
            if input.starts_with(patterns[i]) {
                return Some(i);
            }
        };
        None
    }
    pub struct Tokenizer<'a> {
        input: &'a str,
        pos: usize,
    }

    impl<'a> Tokenizer<'a> {
        pub fn new(input: &str) -> Tokenizer {
            Tokenizer {
                input,
                pos: 0,
            }
        }
        
        /// posに文字があるか確認する
        fn peek(&self) -> Option<char> {
            self.input.chars().nth(self.pos)
        }
        
        /// 現在の要素を返してposを進める
        fn next(&mut self) -> Option<char> {
            let next = self.input.chars().nth(self.pos);
            self.pos += 1;
            next
        }

        pub fn tokenize(&mut self) -> Vec<Token> {
            let mut tok_vec = vec![];

            while let Some(c) = self.peek() {
                // 判定にcを使用
                // 使うときはnext()

                if c.is_whitespace() {
                    self.next();
                    continue;
                }

                let patterns = ["+", "-", "*", "/", "(", ")", ";", "<=", "<", ">=", ">", "==", "!="];

                let next: anyhow::Result<Token> = 
                if let Some(i) = starts_with_in(self.input.get(self.pos..).unwrap(), &patterns) {
                        // posは先頭を保存したいので先にTokenを作る
                        let nxt = Token::new(TokenKind::TK_RESERVED, patterns[i].to_string(), patterns[i].len(), self.pos);
                        self.pos += patterns[i].len();
                        Ok(nxt)
                } else if c.is_ascii_digit() {
                    // 数字を処理する
                    let mut number = self.next().unwrap().to_string();
                    
                    let head_pos = self.pos;

                    // peekで次の値の参照が得られる限り
                    while let Some(n) = self.peek() {
                        if n.is_ascii_digit() {
                            number.push(self.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    let mut next = Token::new(TokenKind::TK_NUM, number.clone(), number.len(), head_pos);
                    next.val = Some(number.parse::<i32>().unwrap());
                    Ok(next)

                } else if 'a' <= c && c <= 'z' {
                    let str = self.next().unwrap();
                    let len = 1;
                    Ok(Token::new(
                        TokenKind::TK_IDENT, 
                        str.to_string(), 
                        len, 
                        self.pos
                    ))

                } else {
                    Err(anyhow!("トークナイズできません: '{}'", c))
                };

                match next {
                    Ok(next) => tok_vec.push(next),
                    Err(e) => {
                        eprintln!("Error While Tokenizing");
                        error_at(self.input, self.pos, e);
                    }
                }
            }
            let eof = Token::new(TokenKind::TK_EOF, String::from("<EOF>"), 1, self.pos);
            tok_vec.push(eof);
            tok_vec
        }
    }
}