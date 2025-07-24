#![allow(non_camel_case_types)]

use anyhow::anyhow;
use std::env;
use std::process::exit;

fn error_at(input: &str, pos: usize, e: anyhow::Error) {
    eprintln!("{}", input);
    eprint!("{}", " ".repeat(pos));
    eprint!("^ ");
    eprintln!("{}", e);
    exit(1);
}

#[derive(PartialEq, Debug, Clone)]
enum NodeKind {
    ND_ADD,
    ND_SUB,
    ND_MUL,
    ND_DIV,
    ND_NUM,
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
            kind: kind,
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
    pos: usize,
}

impl Token {
    fn create_next(kind: TokenKind, str: String, pos: usize) -> Token {
        Token {
            kind,
            str,
            val: None,
            pos,
        }
    }
}

#[derive(Debug)]
struct CurrentToken {
    tok_vec: Vec<Token>,
    idx: usize,
    input: String,
}

impl CurrentToken {
    fn consume(&mut self, op: char) -> bool {
        let tok = &self.tok_vec[self.idx];
        if tok.kind != TokenKind::TK_RESERVED || tok.str.chars().nth(0) != Some(op) {
            false
        } else {
            self.idx += 1;
            true
        }
    }

    fn expect(&mut self, op: char) -> anyhow::Result<()> {
        let tok = &self.tok_vec[self.idx];
        if tok.kind != TokenKind::TK_RESERVED || tok.str.chars().nth(0) != Some(op) {
            Err(anyhow!("'{}'ではありません", op))
        } else {
            self.idx += 1;
            Ok(())
        }
    }

    fn expect_number(&mut self) -> anyhow::Result<i32> {
        let tok = &self.tok_vec[self.idx];
        if tok.kind != TokenKind::TK_NUM {
            Err(anyhow!("Error: 数ではありません"))
        } else {
            let val = tok.val.unwrap();
            self.idx += 1;
            Ok(val)
        }
    }

    fn at_eof(&self) -> bool {
        let tok = &self.tok_vec[self.idx];
        tok.kind == TokenKind::TK_EOF
    }

    fn expr(&mut self) -> Box<Node> {
        let mut node = self.mul();

        loop {
            if self.consume('+') {
                node = Node::new(NodeKind::ND_ADD, node, self.mul());
            } else if self.consume('-') {
                node = Node::new(NodeKind::ND_SUB, node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Node> {
        let mut node = self.primary();

        loop {
            if self.consume('*') {
                node = Node::new(NodeKind::ND_MUL, node, self.primary());
            } else if self.consume('/') {
                node = Node::new(NodeKind::ND_DIV, node, self.primary());
            } else {
                return node;
            }
        }
    }

    fn primary(&mut self) -> Box<Node> {
        if self.consume('(') {
            let node = self.expr();
            match self.expect(')') {
                Ok(()) => (),
                Err(e) => error_at(&self.input, self.idx, e),
            };
            node
        } else {
            let mut num = None;
            match self.expect_number() {
                Ok(val) => num = Some(val),
                Err(e) => {
                    error_at(&self.input, self.idx, e);
                }
            };
            Node::new_node_num(num.unwrap())
        }
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let head = Token {
        kind: TokenKind::TK_RESERVED,
        val: None,
        str: String::from("<HEAD>"),
        pos: 0,
    };

    let mut tok_vec = vec![head];

    let mut pos = 0;
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            pos += 1;
            continue;
        }

        let next: anyhow::Result<Token> = if "+-*/()".contains(c) {
            let nxt = Token::create_next(TokenKind::TK_RESERVED, c.to_string(), pos);
            Ok(nxt)
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
            let mut nxt = Token::create_next(TokenKind::TK_NUM, number.clone(), pos);
            nxt.val = Some(number.parse::<i32>().unwrap());
            Ok(nxt)
        } else {
            Err(anyhow!("トークナイズできません: '{}'", c))
        };

        match next {
            Ok(next) => tok_vec.push(next),
            Err(e) => {
                error_at(input, pos, e);
            }
        }
        pos += 1;
    }
    let eof = Token::create_next(TokenKind::TK_EOF, String::from("EOF"), pos);
    tok_vec.push(eof);
    tok_vec
}

fn generate(node: &Box<Node>) {
    // eprintln!("[DEBUG]: node before parsing number {:?}", node);
    if node.kind == NodeKind::ND_NUM {
        match node.val {
            Some(val) => println!("  push {}", &val),
            None => panic!("gen() error: missing node.val — received None instead"),
        }
        return;
    }

    // eprintln!("[DEBUG]: node after parsing number {:?}", node);

    match &node.lhs {
        Some(lhs) => generate(&lhs),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    match &node.rhs {
        Some(rhs) => generate(&rhs),
        None => panic!("gen() error: missing node.rhs — received None instead"),
    }

    println!("  pop rdi"); // 左側の項の値
    println!("  pop rax"); // 右側の項の値

    match node.kind {
        NodeKind::ND_ADD => println!("  add rax, rdi"),
        NodeKind::ND_SUB => println!("  sub rax, rdi"),
        NodeKind::ND_MUL => println!("  imul rax, rdi"),
        NodeKind::ND_DIV => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::ND_NUM => (),
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
    let tok_vec = tokenize(input);
    let mut tok = CurrentToken {
        tok_vec,
        idx: 1,
        input: input.clone(),
    };

    let node = tok.expr();
    // eprintln!("[DEBUG] node: \n{:?}", node);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    generate(&node);

    println!("  pop rax");
    println!("  ret");
}
