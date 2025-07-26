#![allow(non_camel_case_types)]

use anyhow::anyhow;
use std::env;
use std::process::exit;

use rs9cc::{ error_at, TokenKind, Token, Tokenizer };

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
    ND_ASSIGN,
    ND_LVAR,    // 左辺値
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: Option<i32>,
    offset: Option<u32>,
}

impl Node {
    fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            kind,
            lhs: lhs,
            rhs: rhs,
            val: None,
            offset: None,
        })
    }

    fn new_node_num(val: i32) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_NUM,
            lhs: None,
            rhs: None,
            val: Some(val),
            offset: None,
        })
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
    
    fn consume_ident(&mut self) -> bool {
        let tok = self.tok_vec.get(self.idx).unwrap();
        if tok.kind != TokenKind::TK_IDENT || 
            // TODO: ちょっと危なげ
            tok.str.chars().nth(0) != tok.str.chars().nth(0) {
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
    
    /// `expr ";"`
    fn stmt(&mut self) -> Box<Node> {
        let node = self.expr();
        
        match self.tokens.expect(";") {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error While Parsing");
                error_at(self.tokens.input, self.tokens.idx, e);
            }
        }
        
        node

    }
    
    /// `expr = assign`
    fn expr(&mut self) -> Box<Node> {
        self.assign()
    }
    
    /// `assign = equiality ("=" equiality)?`
    fn assign(&mut self) -> Box<Node> {
        let node = self.equiality();
        
        if self.tokens.consume("=") {
            Node::new(NodeKind::ND_ASSIGN, Some(node), Some(self.equiality()))
        } else {
            return node;
        }
        
    }

    /// `equiality = relational ( "==" relational | "!=" relational )*`
    fn equiality(&mut self) -> Box<Node> {
        let mut node = self.relational();
        
        loop {
            if self.tokens.consume("==") {
                node = Node::new(NodeKind::ND_EQ, Some(node), Some(self.relational()));
            } else if self.tokens.consume("!=") {
                node = Node::new(NodeKind::ND_NE, Some(node), Some(self.relational()));
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
                node = Node::new(NodeKind::ND_LE, Some(node), Some(self.add()));
            } else if self.tokens.consume("<") {
                node = Node::new(NodeKind::ND_LT, Some(node), Some(self.add()));
            } else if self.tokens.consume(">=") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LE, Some(self.add()), Some(node));
            } else if self.tokens.consume(">") {
                // 逆にするだけ
                node = Node::new(NodeKind::ND_LT, Some(self.add()), Some(node));
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
                node = Node::new(NodeKind::ND_ADD, Some(node), Some(self.mul()));
            } else if self.tokens.consume("-") {
                node = Node::new(NodeKind::ND_SUB, Some(node), Some(self.mul()));
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Box<Node> {
        let mut node = self.unary();

        loop {
            if self.tokens.consume("*") {
                node = Node::new(NodeKind::ND_MUL, Some(node), Some(self.unary()));
            } else if self.tokens.consume("/") {
                node = Node::new(NodeKind::ND_DIV, Some(node), Some(self.unary()));
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
            Node::new(NodeKind::ND_SUB, Some(Node::new_node_num(0)), Some(self.primary()))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Box<Node> {
        if self.tokens.consume("(") {
            let node = self.expr();
            match self.tokens.expect(")") {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    error_at(&self.tokens.input, self.tokens.idx, e);
                }
            };
            node
        } else if self.tokens.consume_ident() {
            let mut next = Node::new(NodeKind::ND_LVAR, None, None);
            // TODO: これは読みづらい! 現在のトークンを取得するメソッドを作る
            next.offset = Some((self.tokens.tok_vec[self.tokens.idx-1].str.chars().nth(0).unwrap() as u32 - 'a' as u32 + 1) * 8);
            next
        } else {
            let mut num = None;
            match self.tokens.expect_number() {
                Ok(val) => num = Some(val),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    let idx = self.tokens.idx;
                    error_at(&self.tokens.input, self.tokens.tok_vec[idx].pos, e);
                }
            };
            Node::new_node_num(num.unwrap())
        }
    }
    
}

fn generate(node: &Node) {

    if node.kind == NodeKind::ND_NUM {
        match node.val {
            Some(val) => println!("  push {}", &val),
            None => panic!("gen() error: missing node.val — received None instead"),
        }
        return;
    }

    if node.kind == NodeKind::ND_LVAR {
        // TODO: 左辺値のアセンブリコードを実装する
        panic!("LVAR is not implemented!");
    }

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
        _ => {
            eprintln!("そんなトークン知らねぇ！！\nアセンブリ出力部分が未実装！！");
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

    let node = tok.stmt();

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
