#![allow(non_camel_case_types)]

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    TK_RETURN,   // return
    TK_IF,       // if
    TK_ELSE,     // else
    TK_RESERVED, // 記号
    TK_IDENT,    // 変数名の識別子
    TK_NUM,      // 整数
    TK_EOF,      // 入力の終わり
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub val: Option<i32>, // TODO: この大きさでいいのか？
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

#[derive(PartialEq, Debug, Clone)]
pub enum NodeKind {
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
    ND_RETURN,
    ND_IF,
    ND_ELSE,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub val: Option<i32>,
    pub offset: Option<usize>,
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            kind,
            lhs: lhs,
            rhs: rhs,
            val: None,
            offset: None,
        })
    }

    pub fn new_node_num(val: i32) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_NUM,
            lhs: None,
            rhs: None,
            val: Some(val),
            offset: None,
        })
    }

    pub fn new_node_lvar(offset: usize) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_LVAR,
            lhs: None,
            rhs: None,
            val: None,
            offset: Some(offset),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LVar {
    pub name: String,
    pub len: usize,
    pub offset: usize,
}

impl LVar {
    pub fn new(name: String, len: usize, offset: usize) -> LVar {
        LVar {
            name,
            len,
            offset,
        }
    }
}