#![allow(non_camel_case_types)]

#[derive(Debug, PartialEq, Clone)]
pub enum TypeKind {
    Int,
}

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
    TK_TYPE(TypeKind), // 型
    TK_EOF,      // 入力の終わり
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub val: Option<i32>, // 32bitの符号付き整数型のみ
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

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Le, Lt, Eq, Ne,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Num(i32),
    Var(String),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Assign {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        fn_name: String,
        args: Vec<Expr>,
    },
    Addr (Box<Expr>),
    Deref (Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    ExprStmt(Expr),
    Return(Expr),
    If {
        cond: Expr,
        then: Box<Stmt>,
        els: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<Box<Expr>>,
        cond: Option<Box<Expr>>,
        update: Option<Box<Expr>>,
        body: Box<Stmt>,
    },
    Block(Vec<Stmt>),
    Fn {
        fn_name: String,
        params: Vec<Expr>,
        body: Vec<Stmt>,
    },
    VarDecl {
        name: String,
        ty: Type,
    },
}