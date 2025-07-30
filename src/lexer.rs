use std::usize;

use anyhow::anyhow;

use crate::types::{ TokenKind, Token };
use crate::error_at;

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

    /// 文字列のリストを渡して一致したらその要素を返す
    fn starts_with_in(&self, patterns: &[&'a str]) -> Option<&'a str> {
        for pat in patterns {
            if self.input.get(self.pos..).unwrap().starts_with(pat) {
                return Some(pat);
            }
        };
        None
    }
    
    /// 入力のインデックスはトークンの構成文字か調べる
    fn is_alnum(&self, idx: usize) -> bool {
        let maybe_c = self.input.chars().nth(idx);
        match maybe_c {
            None => false,
            Some(c) => c.is_ascii_alphanumeric() || c == '_'
        }
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
            
            // returnをトークナイズする
            // 次の文字も調べる必要がある
            let len_return = "return".len();
            if self.input.get(self.pos..).unwrap().starts_with("return") && !self.is_alnum(self.pos + len_return) {
                let next = Token::new(TokenKind::TK_RETURN, "return".to_string(), len_return, self.pos);
                self.pos += len_return;
                
                tok_vec.push(next);

                continue;
            }
            
            // ifをトークナイズする
            // 次の文字も調べる必要がある
            let len_if = "if".len();
            if self.input.get(self.pos..).unwrap().starts_with("if") && !self.is_alnum(self.pos + len_if) {
                let next = Token::new(TokenKind::TK_IF, "if".to_string(), len_if, self.pos);
                self.pos += len_if;
                
                tok_vec.push(next);
                
                continue;
            }
            
            // elseをトークナイズする
            // 次の文字も調べる必要がある
            let len_else = "else".len();
            if self.input.get(self.pos..).unwrap().starts_with("else") && !self.is_alnum(self.pos + len_else) {
                let next = Token::new(TokenKind::TK_ELSE, "else".to_string(), len_else, self.pos);
                self.pos += len_else;
                
                tok_vec.push(next);
                
                continue;
            }

            // 2文字の予約語をトークナイズする
            let patterns_len_2 = ["<=", ">=", "==", "!="];
            if let Some(pat) = self.starts_with_in(&patterns_len_2) {
                // posは先頭を保存したいので先にTokenを作る
                let next = Token::new(TokenKind::TK_RESERVED, pat.to_string(), 2, self.pos);
                self.pos += 2;

                tok_vec.push(next);

                continue;
            }

            // 1文字の予約語をトークナイズする
            let patterns_1 = ["+", "-", "*", "/", "(", ")", ";", "<", ">", "="];
            if let Some(pat) = self.starts_with_in(&patterns_1) {
                // posは先頭を保存したいので先にTokenを作る
                let next = Token::new(TokenKind::TK_RESERVED, pat.to_string(), 1, self.pos);
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
                let head_pos = self.pos;
                let mut ident = self.next().unwrap().to_string();
                
                while let Some(s) = self.peek() {
                    if 'a' <= s && s <= 'z' {
                        ident.push(self.next().unwrap());
                    } else {
                        break;
                    }
                }

                let len = ident.len();
                let next = Token::new(TokenKind::TK_IDENT, ident, len, head_pos);
                
                tok_vec.push(next);
                
                continue;
            } 
            
            // それ以外はエラーを出す
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


#[derive(Debug)]
pub struct TokenStream<'a> {
    pub tok_vec: Vec<Token>,
    pub idx: usize,
    pub input: &'a str,
}

impl<'a> TokenStream<'a> {
    pub fn new(tok_vec: Vec<Token>, input:&str) -> TokenStream {
        TokenStream {
            tok_vec,
            idx: 0,
            input,
        }
    }

    pub fn consume(&mut self, op: &str) -> bool {
        let tok = self.tok_vec.get(self.idx).unwrap();
        let len = op.len();
        if tok.kind != TokenKind::TK_RESERVED || 
           tok.str.get(..len) != Some(op) || 
           tok.len != len {
            false
        } else {
            self.idx += 1;
            true
        }
    }
    
    pub fn consume_keyword(&mut self, kind: TokenKind) -> bool {
        let tok = self.tok_vec.get(self.idx).unwrap();
        if tok.kind != kind {
            false
        } else {
            self.idx += 1;
            true
        }
    }
    
    /// 変数名ならその変数名を返す
    pub fn consume_ident(&mut self) -> Option<Token> {
        // ここで呼び出しているメソッドはクローンを返すため
        let tok = self.get_current_token();
        if tok.kind != TokenKind::TK_IDENT {
            None
        } else {
            self.idx += 1;
            Some(tok)
        }
    }

    pub fn expect(&mut self, op: &str) -> anyhow::Result<()> {
        let tok = self.tok_vec.get(self.idx).unwrap();
        let len = op.len();
        if tok.kind != TokenKind::TK_RESERVED || 
           tok.str.get(..len) != Some(op) || 
           tok.len != len {
            if op == ";" {
                Err(anyhow!("';'が必要です"))
            } else {
                Err(anyhow!("'{}'を想定していたが、'{}'が入力されました", op, tok.str))
            }
        } else {
            self.idx += 1;
            Ok(())
        }
    }

    pub fn expect_number(&mut self) -> anyhow::Result<i32> {
        let tok = self.tok_vec.get(self.idx).unwrap();
        if tok.kind != TokenKind::TK_NUM {
            Err(anyhow!("Error: ここは直前に数字が必要です"))
        } else {
            match tok.val {
                Some(val) => {
                    self.idx += 1;
                    Ok(val)
                },
                None => Err(anyhow!("Error: 'Token'に数字が格納されていません"))
            }
        }
    }
    
    /// 現在のトークンを取得する
    pub fn get_current_token(&self) -> Token {
        let current_idx = self.idx;
        self.tok_vec[current_idx].clone()
    }
}