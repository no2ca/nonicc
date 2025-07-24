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
    _pos: usize,
}

impl Token {
    fn new(kind: TokenKind, str: String, len: usize, pos: usize) -> Token {
        Token {
            kind,
            str,
            val: None,
            len,
            _pos: pos,
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
    fn consume(&mut self, op: &str) -> bool {
        let tok = &self.tok_vec[self.idx];
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
        let tok = &self.tok_vec[self.idx];
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
        let tok = &self.tok_vec[self.idx];
        if tok.kind != TokenKind::TK_NUM {
            Err(anyhow!("Error: 数ではありません"))
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

    /* 
    fn at_eof(&self) -> bool {
        let tok = &self.tok_vec[self.idx];
        tok.kind == TokenKind::TK_EOF
    }
    */
    
    // expr = relational ( "==" relational | "!=" relational )*
    fn expr(&mut self) -> Box<Node> {
        let mut node = self.relational();
        
        loop {
            if self.consume("==") {
                node = Node::new(NodeKind::ND_EQ, node, self.relational());
            } else if self.consume("!=") {
                node = Node::new(NodeKind::ND_NE, node, self.relational());
            } else {
                return node;
            }
        }
    } 
    
    // relational = add ( "<" add | "<=" add | ">" add | ">=" add )*
    fn relational(&mut self) -> Box<Node> {
        let mut node = self.add();
        
        // 長いトークンから見ていく
        loop {
            if self.consume("<=") {
                node = Node::new(NodeKind::ND_LE, node, self.add());
            } else if self.consume("<") {
                node = Node::new(NodeKind::ND_LT, node, self.add());
            } else if self.consume(">=") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LE, self.add(), node);
            } else if self.consume(">") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LT, self.add(), node);
            } else {
                return node;
            }
        }
    }

    // add = mul ( "+" mul | "-" mul )*
    fn add(&mut self) -> Box<Node> {
        let mut node = self.mul();

        loop {
            if self.consume("+") {
                node = Node::new(NodeKind::ND_ADD, node, self.mul());
            } else if self.consume("-") {
                node = Node::new(NodeKind::ND_SUB, node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Node> {
        let mut node = self.unary();

        loop {
            if self.consume("*") {
                node = Node::new(NodeKind::ND_MUL, node, self.unary());
            } else if self.consume("/") {
                node = Node::new(NodeKind::ND_DIV, node, self.unary());
            } else {
                return node;
            }
        }
    }
    
    fn unary(&mut self) -> Box<Node> {
        if self.consume("+") {
            self.primary()
        } else if self.consume("-") {
            // 一時的に 0-primary() の形で負の数を表す
            Node::new(NodeKind::ND_SUB, Node::new_node_num(0), self.primary())
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Box<Node> {
        if self.consume("(") {
            let node = self.expr();
            match self.expect(")") {
                Ok(()) => (),
                Err(e) => error_at(&self.input, self.idx, e),
            };
            node
        } else {
            let mut num = None;
            match self.expect_number() {
                Ok(val) => num = Some(val),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    error_at(&self.input, self.idx, e);
                }
            };
            Node::new_node_num(num.unwrap())
        }
    }
}

fn starts_with_in(input: &str, patterns: &[&str]) -> Option<usize> {
    for i in 0..patterns.len() {
        if input.starts_with(patterns[i]) {
            return Some(i);
        }
    };
    None
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let head = Token::new(TokenKind::TK_RESERVED, "<HEAD>".to_string(), 6, 0);
    let mut tok_vec = vec![head];

    let mut pos = 0;
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            pos += 1;
            continue;
        }

        eprintln!("pos: {:?}", pos);
        let patterns = ["+", "-", "*", "/", "(", ")", "<=", "<", ">=", ">", "==", "!="];
        let next: anyhow::Result<Token> = 
        if let Some(i) = starts_with_in(input.get(pos..).unwrap(), &patterns) {
                eprintln!("as reserved {}", patterns[i]);
                let nxt = Token::new(TokenKind::TK_RESERVED, patterns[i].to_string(), patterns[i].len(), pos);
                pos += patterns[i].len();
                Ok(nxt)
        } else if c.is_ascii_digit() {
            eprintln!("as digit");
            // 数字を処理する
            let mut number = c.to_string();
            // peekで次の値の参照が得られる限り
            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    number.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            let mut nxt = Token::new(TokenKind::TK_NUM, number.clone(), number.len(), pos);
            pos += number.len();
            nxt.val = Some(number.parse::<i32>().unwrap());
            Ok(nxt)
        } else {
            Err(anyhow!("トークナイズできません: '{}'", c))
        };

        match next {
            Ok(next) => tok_vec.push(next),
            Err(e) => {
                eprintln!("Error While Tokenizing");
                error_at(input, pos, e);
            }
        }
    }
    let eof = Token::new(TokenKind::TK_EOF, String::from("EOF"), 1, pos);
    tok_vec.push(eof);
    tok_vec
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
    // 数以外は両側に何か持っているはずなので
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
        NodeKind::ND_ADD => println!("  add rax, rdi"),
        NodeKind::ND_SUB => println!("  sub rax, rdi"),
        NodeKind::ND_MUL => println!("  imul rax, rdi"),
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
