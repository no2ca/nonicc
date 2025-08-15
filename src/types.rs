#![allow(non_camel_case_types)]

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    TK_RETURN,   // return
    TK_WHILE,    // while
    TK_FOR,      // for
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
    ND_LVAR,
    ND_RETURN,
    ND_WHILE,
    ND_IF,
    ND_ELSE,
    ND_BLOCK,
    ND_CALL,
    ND_DEFUN,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub cond: Option<Box<Node>>,    // if文の条件
    pub then: Option<Box<Node>>,    // if文のthen
    pub els: Option<Box<Node>>,     // if文のelse
    pub body: Option<Box<Node>>,    // while文のbody
    pub val: Option<i32>,           // ND_NUMのとき
    pub offset: Option<usize>,      // ND_IDENTのとき
    pub stmts: Option<Vec<Node>>,   // ND_BLOCK, ND_DEFUNのとき
    pub ident_name: Option<String>, // ND_LVARのときの変数名
    pub fn_name: Option<String>,    // ND_CALL, ND_DEFUNのときの関数名
    pub params: Option<Vec<Node>>,  // ND_DEFUNのとき
    pub args: Option<Vec<Node>>,    // ND_CALLのとき
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
            body: None,
            val: None,
            offset: None,
            stmts: None,
            ident_name: None,
            fn_name: None,
            params: None,
            args: None,
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
            body: None,
            val: Some(val),
            offset: None,
            stmts: None,
            ident_name: None,
            fn_name: None,
            params: None,
            args: None,
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
            body: None,
            val: None,
            offset: Some(offset),
            stmts: None,
            ident_name: Some(ident_name),
            fn_name: None,
            params: None,
            args: None,
        })
    }
    
    pub fn new_node_while(cond:Box<Node>, body: Box<Node>) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_WHILE, 
            lhs: None, 
            rhs: None, 
            cond: Some(cond), 
            then: None, 
            els: None, 
            body: Some(body),
            val: None, 
            offset: None, 
            stmts: None,
            ident_name: None,
            fn_name: None,
            params: None,
            args: None,
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
            body: None,
            val: None, 
            offset: None, 
            stmts: None,
            ident_name: None,
            fn_name: None,
            params: None,
            args: None,
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
            body: None,
            val: None,
            offset: None,
            stmts: Some(block_stmt),
            ident_name: None,
            fn_name: None,
            params: None,
            args: None,
        })
    }
    
    pub fn new_node_call(fn_name: String, args: Vec<Node>) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_CALL,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            body: None,
            val: None,
            offset: None,
            stmts: None,
            ident_name: None,
            fn_name: Some(fn_name.to_string()),
            params: None,
            args: Some(args),
        })
    }
    
    pub fn new_node_defun(fn_name: String, stmts: Vec<Node>, params: Vec<Node>) -> Box<Node> {
        Box::new(Node {
            kind: NodeKind::ND_DEFUN,
            lhs: None,
            rhs: None,
            cond: None,
            then: None,
            els: None,
            body: None,
            val: None,
            offset: None,
            stmts: Some(stmts),
            ident_name: None,
            fn_name: Some(fn_name.to_string()),
            params: Some(params),
            args: None,
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