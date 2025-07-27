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
    
    /// 変数名ならその変数名を返す
    fn consume_ident(&mut self) -> Option<char> {
        let tok = self.tok_vec.get(self.idx).unwrap();
        let ident = tok.str.chars().nth(0);
        if tok.kind != TokenKind::TK_IDENT || 
            tok.str.chars().nth(0) !=  ident ||
            ident == None {
            // TODO: ↑ちょっと危なげ、、1文字限定。
            None
        } else {
            self.idx += 1;
            ident
        }
    }

    fn expect(&mut self, op: &str) -> anyhow::Result<()> {
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
    
    /// 現在のトークンを取得する
    fn get_current_token(&self) -> Token {
        let current_idx = self.idx;
        self.tok_vec[current_idx].clone()
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
                error_at(self.tokens.input, self.tokens.get_current_token().pos, e);
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
                    error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                }
            };
            node
        } else if let Some(ident) = self.tokens.consume_ident() {
            let mut next = Node::new(NodeKind::ND_LVAR, None, None);

            // ここは一時的に文字コードにおけるオフセットを設定
            next.offset = Some((ident as u32 - 'a' as u32 + 1) * 8);
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

/// 代入先のアドレスをスタックに積む
fn generate_lval(node: &Node) {
    if node.kind != NodeKind::ND_LVAR {
        eprintln!("代入の左辺値が変数ではありません");
        exit(1);
    }
    
    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.expect("node.offsetがNoneです"));
    println!("  push rax");
    return;
}

fn generate(node: &Node) {

    match node.kind { 
        NodeKind::ND_NUM => {
            match node.val {
                Some(val) => println!("  push {}", &val),
                None => panic!("gen() error: missing node.val — received None instead"),
            }
            return;
        }

        // ここは右辺値として変数を扱う時
        // つまり変数の評価をするときに呼び出される
        NodeKind::ND_LVAR => {
            generate_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        
        NodeKind::ND_ASSIGN => {
            match &node.lhs {
                Some(lhs) => generate_lval(lhs),
                None => panic!("gen() error: missing node.lhs — received None instead"),
            }

            match &node.rhs {
                Some(rhs) => generate(rhs),
                None => panic!("gen() error: missing node.rhs — received None instead"),
            }
            
            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
        }

        _ => ()

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

    // TODO: 無限ループの可能性がある
    let mut nodes: Vec<Box<Node>> = vec![];
    while tok.tokens.idx != tok.tokens.tok_vec.len() - 1  {
        nodes.push(tok.stmt());
    }

    eprintln!("[DEBUG] tokens.len: {}", tok.tokens.tok_vec.len());
    eprintln!("[DEBUG] idx: {}", tok.tokens.idx);
    
    // トークンを最後までパース出来たか調べる
    // EOFトークンがあるので -1 している
    if tok.tokens.idx != tok.tokens.tok_vec.len() - 1 {
        error_at(tok.tokens.input, tok.tokens.get_current_token().pos, anyhow!("余分なトークンがあります"));
    }

    eprintln!("[DEBUG] node: \n{:?}", nodes.clone());

    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    for node in nodes {
        generate(&node);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
