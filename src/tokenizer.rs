#![allow(non_camel_case_types)]

pub mod token {
    #[derive(PartialEq, Clone, Debug)]
    pub enum TokenKind {
        TK_RESERVED, // 記号
        TK_IDENT,    // 変数名の識別子
        TK_NUM,      // 整数
        TK_EOF,      // 入力の終わり
    }

    #[derive(PartialEq, Clone, Debug)]
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
    use anyhow::{ anyhow };

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

        /// 文字列のリストを渡して一致したらそのインデックスを返す
        fn starts_with_in(&self, patterns: &[&str]) -> Option<usize> {
            for i in 0..patterns.len() {
                if self.input.get(self.pos..).unwrap().starts_with(patterns[i]) {
                    return Some(i);
                }
            };
            None
        }
        
        /// 次に文字があるか確認する
        fn peek(&self) -> Option<char> {
            self.input.chars().nth(self.pos)
        }
        
        /// 現在の要素を返して1文字を進める
        fn next(&mut self) -> Option<char> {
            let next = self.input.chars().nth(self.pos);
            self.pos += 1;
            next
        }
        
        pub fn tokenize(&mut self) -> Vec<Token> {
            let mut tok_vec = vec![];

            // 判定にcを使用
            // 使うときはnext()
            while let Some(c) = self.peek() {

                // 空白を飛ばす
                if c.is_whitespace() {
                    self.next();
                    continue;
                }

                // 2文字の予約語をトークナイズする
                let patterns_len_2 = ["<=", ">=", "==", "!="];
                if let Some(i) = self.starts_with_in(&patterns_len_2) {
                    // posは先頭を保存したいので先にTokenを作る
                    let word = patterns_len_2[i].to_string();
                    let next = Token::new(TokenKind::TK_RESERVED, word, 2, self.pos);
                    self.pos += 2;

                    tok_vec.push(next);

                    continue;
                }

                // 1文字の予約語をトークナイズする
                let patterns_1 = ["+", "-", "*", "/", "(", ")", ";", "<", ">", "="];
                if let Some(i) = self.starts_with_in(&patterns_1) {
                    // posは先頭を保存したいので先にTokenを作る
                    let word = patterns_1[i].to_string();
                    let next = Token::new(TokenKind::TK_RESERVED, word, 1, self.pos);
                    self.pos += 1;

                    tok_vec.push(next);

                    continue;
                }

                // 数字をトークナイズする
                if c.is_ascii_digit() {
                    let head_pos = self.pos;
                    let mut number = self.next().unwrap().to_string();

                    // peekで次の値の参照が得られる限り
                    while let Some(n) = self.peek() {
                        if n.is_ascii_digit() {
                            number.push(self.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    let mut next = Token::new(TokenKind::TK_NUM, number.clone(), number.len(), head_pos);
                    // 数字を設定する
                    next.val = Some(number.parse::<i32>().unwrap());

                    tok_vec.push(next);
                    
                    continue;
                }

                // 変数をトークナイズする
                if 'a' <= c && c <= 'z' {
                    let str = self.next().unwrap();
                    let len = 1;
                    let next = Token::new(TokenKind::TK_IDENT, str.to_string(), len, self.pos);
                    
                    tok_vec.push(next);
                    
                    continue;
                } 
                
                else {
                    eprintln!("Error While Tokenizing");
                    let e = anyhow!("トークナイズできません");
                    error_at(self.input, self.pos, e);
                };
            }

            let eof = Token::new(TokenKind::TK_EOF, String::from("<EOF>"), 1, self.pos);
            tok_vec.push(eof);

            tok_vec

        }
    }
}