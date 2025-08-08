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
    ND_BLOCK,
    ND_FN,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub cond: Option<Box<Node>>,    // if文の条件
    pub then: Option<Box<Node>>,    // if文のthen
    pub els: Option<Box<Node>>,     // if文のelse
    pub val: Option<i32>,           // ND_NUMのとき
    pub offset: Option<usize>,      // ND_IDENTのとき
    pub block_stmt: Option<Vec<Node>>, // ND_BLOCKのとき
    pub ident_name: Option<String>,
    pub fn_name: Option<String>,       // ND_FNのとき
}

impl Node {
    pub fn new(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            kind,
            lhs,
            rhs: rhs,
            cond: None,
            then: None,
            els: None,
            val: None,
            offset: None,
            block_stmt: None,
            ident_name: None,
            fn_name: None,
        })
    }

    pub fn new_node_num(val: i32) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_NUM,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            val: Some(val),
            offset: None,
            block_stmt: None,
            ident_name: None,
            fn_name: None,
        })
    }

    pub fn new_node_lvar(offset: usize, ident_name: String) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_LVAR,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            val: None,
            offset: Some(offset),
            block_stmt: None,
            ident_name: Some(ident_name),
            fn_name: None,
        })
    }
    
    pub fn new_node_if(cond:Box<Node>, then: Box<Node>, els: Option<Box<Node>>) -> Box<Node> {
        Box::new(Node { 
            kind: NodeKind::ND_IF, 
            lhs: None, 
            rhs: None, 
            cond: Some(cond), 
            then: Some(then), 
            els, 
            val: None, 
            offset: None, 
            block_stmt: None,
            ident_name: None,
            fn_name: None,
        })
    }
    
    pub fn new_node_block(block_stmt: Vec<Node>) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_BLOCK,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            val: None,
            offset: None,
            block_stmt: Some(block_stmt),
            ident_name: None,
            fn_name: None,
        })
    }
    
    pub fn new_node_fn(fn_name: String) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_FN,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            val: None,
            offset: None,
            block_stmt: None,
            ident_name: None,
            fn_name: Some(fn_name.to_string()),
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