#![allow(non_camel_case_types)] 

use crate::types::{ Token, Node, NodeKind, LVar };
use crate::types::TokenKind::{ TK_RETURN, TK_IF, TK_ELSE };
use crate::lexer::TokenStream;
use crate::error_at;

pub struct Lvars {
    pub lvars_vec: Vec<LVar>,
}

impl Lvars {
    fn new() -> Self {
        // 先頭の識別子は衝突しない名前で
        let head_lvars = LVar::new("__dummy".to_string(), 12, 0);
        Lvars { lvars_vec: vec![head_lvars] }
    }

    /// 現在見ている変数名を検索する
    /// ローカル変数のオフセットを決めるのに使用する
    fn find_lvar(&self, cur: &Token) -> Option<LVar> {
        let lvars_vec = &self.lvars_vec;
        // 先頭を含めなければ衝突しない
        for i in 1..lvars_vec.len() {
            let lvar = &lvars_vec[i];
            if lvar.len == cur.len && lvar.name == cur.str {
                return Some(lvar.clone());
            }
        }
        None
    }
}

pub struct Parser<'a> {
    pub tokens: TokenStream<'a>,
    pub lvars: Lvars,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenStream) -> Parser {
        Parser {
            tokens,
            lvars: Lvars::new(),
        }
    }
    
    /// stmt = expr ";" | 
    ///        "return" expr ";" |
    ///        "if"  "(" expr ")" stmt ("else" stmt)? |
    ///        "{" stmt* "}"
    pub fn stmt(&mut self) -> Box<Node> {

        // if文をパース
        if self.tokens.consume_keyword(TK_IF) {

            // 条件のパース
            self.tokens.expect("(").unwrap_or_else( |e|{
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            });

            let cond = self.expr();
            
            self.tokens.expect(")").unwrap_or_else( |e|{
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            });
            
            // thenのパース
            let then = self.stmt();
            
            // elseの有無で分岐
            let els = if self.tokens.consume_keyword(TK_ELSE) {
                Some(self.stmt())
            } else {
                None
            };
            
            Node::new_node_if(cond, then, els)

        } else if self.tokens.consume("{") {
            // ブロックをパース
            let mut block_stmt: Vec<Node> = vec![];
            while !self.tokens.consume("}") {
                block_stmt.push(*self.stmt());
            }
            Node::new_node_block(block_stmt)
        } else {
            let node: Box<Node>;
            
            if self.tokens.consume_keyword(TK_RETURN) {
                // return文の場合
                // 木は左から埋めていく
                node = Node::new(NodeKind::ND_RETURN, Some(self.expr()), None);
            } else { 
                // それ以外は式 (expr)
                node = self.expr();
            }

            // セミコロンで文が閉じているか
            match self.tokens.expect(";") {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    error_at(self.tokens.input, self.tokens.get_current_token().pos, e);
                }
            }
            
            node
        }
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

    /// `primary = num | ident ("(" ")")? | "(" expr ")" `
    fn primary(&mut self) -> Box<Node> {
        // "(" expr ")"
        if self.tokens.consume("(") {
            let node = self.expr();
            match self.tokens.expect(")") {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                }
            };
            return node;
        }

        // ident ("(" ")")?
        if let Some(ident) = self.tokens.consume_ident() {
            // 関数かどうか調べる
            if self.tokens.consume("(") {
                match self.tokens.expect(")") {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("Error While Parsing");
                        error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                    }
                };
                return Node::new_node_fn(ident.str);
            }

            // ローカル変数が既にあるか調べる
            if let Some(lvar) = self.lvars.find_lvar(&ident) {
                // ある場合はそのオフセットを使う
                let offset = lvar.offset;
                return Node::new_node_lvar(offset);
            } else {
                // ない場合は手前のに8を足して使う
                // TokenStreamの初期化時に先頭があるため
                let offset = self.lvars.lvars_vec.last().unwrap().offset + 8;
                let lvar = LVar::new(ident.str, ident.len, offset);
                self.lvars.lvars_vec.push(lvar);
                return Node::new_node_lvar(offset);
            }
            
        }

        let mut num = None;
        match self.tokens.expect_number() {
            Ok(val) => num = Some(val),
            Err(e) => {
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            }
        };
        Node::new_node_num(num.unwrap())
    }
    
}