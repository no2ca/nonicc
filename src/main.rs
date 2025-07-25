#![allow(non_camel_case_types)]

use anyhow::anyhow;
use std::env;
use std::process::exit;

use rs9cc::{ error_at,starts_with_in };

#[derive(PartialEq, Debug, Clone)]
enum NodeKind {
    ND_ADD,
    ND_SUB,
    ND_MUL,
    ND_DIV,
    ND_NUM,
    ND_LE,
    ND_LT,
    ND_EQ,
    ND_NE,
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: Option<i32>,
}

impl Node {
    fn new(kind: NodeKind, lhs: Box<Node>, rhs: Box<Node>) -> Box<Node> {
        Box::new(Node {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
        })
    }

    fn new_node_num(val: i32) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_NUM,
            lhs: None,
            rhs: None,
            val: Some(val),
        })
    }
}

#[derive(PartialEq, Clone, Debug)]
enum TokenKind {
    TK_RESERVED,
    TK_NUM,
    TK_EOF,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    val: Option<i32>, // WARNING: この大きさでいいのか？
    str: String,
    len: usize,
    pos: usize,
}

impl Token {
    fn new(kind: TokenKind, str: String, len: usize, pos: usize) -> Token {
        Token {
            kind,
            str,
            val: None,
            len,
            pos,
        }
    }
}

#[derive(Debug)]
struct TokenStream<'a> {
    tok_vec: Vec<Token>,
    idx: usize,
    input: &'a str,
}

impl<'a> TokenStream<'a> {
    fn new(tok_vec: Vec<Token>, input:&str) -> TokenStream {
        TokenStream {
            tok_vec,
            idx: 0,
            input,
        }
    }

    fn consume(&mut self, op: &str) -> bool {
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

    fn expect(&mut self, op: &str) -> anyhow::Result<()> {
        let tok = self.tok_vec.get(self.idx).unwrap();
        let len = op.len();
        if tok.kind != TokenKind::TK_RESERVED || 
           tok.str.get(..len) != Some(op) || 
           tok.len != len {
            Err(anyhow!("'{}'を想定していたが、'{}'が入力されました", op, tok.str))
        } else {
            self.idx += 1;
            Ok(())
        }
    }

    fn expect_number(&mut self) -> anyhow::Result<i32> {
        let tok = self.tok_vec.get(self.idx).unwrap();
        if tok.kind != TokenKind::TK_NUM {
            Err(anyhow!("Error: ここは数字が必要です"))
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
    
}

struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    fn new(tokens: TokenStream) -> Parser {
        Parser {
            tokens,
        }
    }

    /// `expr = relational ( "==" relational | "!=" relational )*`
    fn expr(&mut self) -> Box<Node> {
        let mut node = self.relational();
        
        loop {
            if self.tokens.consume("==") {
                node = Node::new(NodeKind::ND_EQ, node, self.relational());
            } else if self.tokens.consume("!=") {
                node = Node::new(NodeKind::ND_NE, node, self.relational());
            } else {
                return node;
            }
        }
    } 
    
    /// `relational = add ( "<" add | "<=" add | ">" add | ">=" add )*`
    fn relational(&mut self) -> Box<Node> {
        let mut node = self.add();
        
        // 長いトークンから見ていく
        loop {
            if self.tokens.consume("<=") {
                node = Node::new(NodeKind::ND_LE, node, self.add());
            } else if self.tokens.consume("<") {
                node = Node::new(NodeKind::ND_LT, node, self.add());
            } else if self.tokens.consume(">=") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LE, self.add(), node);
            } else if self.tokens.consume(">") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LT, self.add(), node);
            } else {
                return node;
            }
        }
    }

    /// `add = mul ( "+" mul | "-" mul )*`
    fn add(&mut self) -> Box<Node> {
        let mut node = self.mul();

        loop {
            if self.tokens.consume("+") {
                node = Node::new(NodeKind::ND_ADD, node, self.mul());
            } else if self.tokens.consume("-") {
                node = Node::new(NodeKind::ND_SUB, node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Node> {
        let mut node = self.unary();

        loop {
            if self.tokens.consume("*") {
                node = Node::new(NodeKind::ND_MUL, node, self.unary());
            } else if self.tokens.consume("/") {
                node = Node::new(NodeKind::ND_DIV, node, self.unary());
            } else {
                return node;
            }
        }
    }
    
    fn unary(&mut self) -> Box<Node> {
        if self.tokens.consume("+") {
            self.primary()
        } else if self.tokens.consume("-") {
            // 一時的に 0-primary() の形で負の数を表す
            Node::new(NodeKind::ND_SUB, Node::new_node_num(0), self.primary())
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Box<Node> {
        if self.tokens.consume("(") {
            let node = self.expr();
            match self.tokens.expect(")") {
                Ok(()) => (),
                Err(e) => error_at(&self.tokens.input, self.tokens.idx, e),
            };
            node
        } else {
            let mut num = None;
            match self.tokens.expect_number() {
                Ok(val) => num = Some(val),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    let idx = self.tokens.idx;
                    eprintln!("[DEBUG] idx: {}", idx);
                    error_at(&self.tokens.input, self.tokens.tok_vec[idx].pos, e);
                }
            };
            Node::new_node_num(num.unwrap())
        }
    }
    
}

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
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

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tok_vec = vec![];

        while let Some(c) = self.peek() {
            // 判定にcを使用
            // 使うときはnext()

            if c.is_whitespace() {
                self.next();
                continue;
            }

            let patterns = ["+", "-", "*", "/", "(", ")", "<=", "<", ">=", ">", "==", "!="];

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

                let mut nxt = Token::new(TokenKind::TK_NUM, number.clone(), number.len(), head_pos);
                nxt.val = Some(number.parse::<i32>().unwrap());
                Ok(nxt)

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

fn generate(node: &Node) {
    // eprintln!("[DEBUG]: node before parsing number {:?}", node);
    if node.kind == NodeKind::ND_NUM {
        match node.val {
            Some(val) => println!("  push {}", &val),
            None => panic!("gen() error: missing node.val — received None instead"),
        }
        return;
    }

    // eprintln!("[DEBUG]: node after parsing number {:?}", node);
    // 数以外は両側に何か持っているはず
    match &node.lhs {
        Some(lhs) => generate(lhs),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    match &node.rhs {
        Some(rhs) => generate(rhs),
        None => panic!("gen() error: missing node.rhs — received None instead"),
    }

    println!("  pop rdi"); // 左側の項の値
    println!("  pop rax"); // 右側の項の値

    match node.kind {
        NodeKind::ND_NUM => (),
        NodeKind::ND_ADD => {
            println!("  add rax, rdi");
        }
        NodeKind::ND_SUB => {
            println!("  sub rax, rdi");
        }
        NodeKind::ND_MUL => {
            println!("  imul rax, rdi");
        }
        NodeKind::ND_DIV => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::ND_LE => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_LT => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_EQ => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_NE => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
    }

    println!("  push rax");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }

    let input = &args[1];
    let mut input_stream = Tokenizer::new(input);
    let tok_vec = input_stream.tokenize();

    eprintln!("[DEBUG] tokens: \n{:?}", &tok_vec);

    let mut tok = Parser::new(TokenStream::new(tok_vec, input));

    let node = tok.expr();

    eprintln!("[DEBUG] tokens.len: {}", tok.tokens.tok_vec.len());
    eprintln!("[DEBUG] idx: {}", tok.tokens.idx);
    
    // トークンを最後までパース出来たか調べる
    // EOFトークンがあるので -1 している
    if tok.tokens.idx != tok.tokens.tok_vec.len() - 1 {
        error_at(tok.tokens.input, tok.tokens.tok_vec[tok.tokens.idx].pos, anyhow!("余分なトークンがあります"));
    }

    eprintln!("[DEBUG] node: \n{:?}", node.clone());

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    generate(&node);

    println!("  pop rax");
    println!("  ret");
}
