use anyhow::anyhow;

use crate::types::{ Token, LVar };
use crate::types::{ BinOp, Expr, Stmt };
use crate::types::TokenKind::{ TK_RETURN, TK_IF, TK_ELSE, TK_WHILE, TK_FOR };
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
    defined_fn: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: TokenStream) -> Parser {
        Parser {
            tokens,
            lvars: Lvars::new(),
            defined_fn: Vec::new(),
        }
    }
    
    /// `params = "(" ident, .. ")"`
    fn params(&mut self) -> Vec<Expr> {
        self.tokens.expect("(").unwrap_or_else( |e|{
            eprintln!("Error While Parsing");
            error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
        });
        let mut params = Vec::new();
        if !self.tokens.consume(")") {
            loop {
                let param = match self.tokens.consume_ident() {
                    Some(t) => Expr::Var(t.str),
                    None => {
                        eprintln!("Error While Parsing");
                        let e = anyhow!("引数は識別子である必要があります");
                        error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                    }
                };
                params.push(param);
                if self.tokens.consume(",") {
                    continue;
                } else {
                    match self.tokens.expect(")") {
                        Ok(()) => break,
                        Err(e) => {
                            error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                        }
                    }
                }
            }
        }
        params
    }
    
    /// `args = expr, .. ")"`
    fn args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        if !self.tokens.consume(")") {
            loop {
                let arg = self.expr();
                args.push(arg);
                if self.tokens.consume(",") {
                    continue;
                } else {
                    match self.tokens.expect(")") {
                        Ok(()) => break,
                        Err(e) => {
                            error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                        }
                    }
                }
            }
        }
        args
    }
    
    /// defun = ident "(" params ")" "{" stmt* "}"
    pub fn defun(&mut self) -> Stmt {
        // 関数名を読む
        let fn_name: String = match self.tokens.consume_ident() {
            Some(ident) => ident.str,
            None => {
                eprintln!("Error While Parsing");
                let e = anyhow!("関数名が見つかりません");
                error_at(self.tokens.input, self.tokens.get_current_token().pos, e);
            }
        };

        // 関数名の重複を調べる
        if self.defined_fn.contains(&fn_name) {
            eprintln!("Error While Parsing");
            let e = anyhow!("関数が重複して定義されています");
            error_at(self.tokens.input, self.tokens.get_current_token().pos, e);
        } else {
            self.defined_fn.push(fn_name.clone());
        }

        let params = self.params();

        self.tokens.expect("{").unwrap_or_else( |e|{
            eprintln!("Error While Parsing");
            error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
        });
        
        let mut body = Vec::new();
        while !self.tokens.consume("}") {
            body.push(self.stmt());
        }
        
        Stmt::Fn { fn_name, params, body }
    }
    
    /// stmt = expr ";" | 
    ///        "while" "(" expr ")" stmt |
    ///        "if"  "(" expr ")" stmt ("else" stmt)? |
    ///        "for" "(" expr? ";" expr? ";" expr? ")" stmt |
    ///        "{" stmt* "}"
    ///        "return" expr ";" |
    fn stmt(&mut self) -> Stmt {
        if self.tokens.consume_keyword(TK_WHILE) {
            // while文をパース
            self.tokens.expect("(").unwrap_or_else( |e|{
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            });

            let cond = self.expr();

            self.tokens.expect(")").unwrap_or_else( |e|{
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            });

            let body = self.stmt();
            
            Stmt::While { cond, body: Box::new(body) }
        } else if self.tokens.consume_keyword(TK_FOR) {
            // for文をパース
            self.tokens.expect("(").unwrap_or_else( |e|{
                eprintln!("Error While Parsing");
                error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
            });
            
            let init = match self.tokens.consume(";") {
                true => {
                    None
                }
                false => {
                    let _init = self.expr();
                    self.tokens.expect(";").unwrap_or_else( |e|{
                        eprintln!("Error While Parsing");
                        error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                    });
                    Some(Box::new(_init))
                }
            };
            let cond = match self.tokens.consume(";") {
                true => {
                    None
                }
                false => {
                    let _cond = self.expr();
                    self.tokens.expect(";").unwrap_or_else( |e|{
                        eprintln!("Error While Parsing");
                        error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                    });
                    Some(Box::new(_cond))
                }
            };
            let update = match self.tokens.consume(")") {
                true => {
                    None
                }
                false => {
                    let _update = self.expr();
                    self.tokens.expect(")").unwrap_or_else( |e|{
                        eprintln!("Error While Parsing");
                        error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                    });
                    Some(Box::new(_update))
                }
            };
            let body = self.stmt();
            Stmt::For { init, cond, update, body: Box::new(body) }
        } else if self.tokens.consume_keyword(TK_IF) {
            // if文をパース
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
                Some(Box::new(self.stmt()))
            } else {
                None
            };
            Stmt::If { 
                cond, 
                then: Box::new(then), 
                els,
            }
        } else if self.tokens.consume("{") {
            // ブロックをパース
            let mut block_stmt = vec![];
            while !self.tokens.consume("}") {
                block_stmt.push(self.stmt());
            }
            Stmt::Block(block_stmt)
        } else {
            let node;
            
            if self.tokens.consume_keyword(TK_RETURN) {
                // return文の場合
                // 木は左から埋めていく
                node = Stmt::Return(self.expr());
            } else { 
                // それ以外は式 (expr)
                node = Stmt::ExprStmt(self.expr());
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
    fn expr(&mut self) -> Expr {
        self.assign()
    }
    
    /// `assign = equiality ("=" equiality)?`
    fn assign(&mut self) -> Expr {
        let pos = self.tokens.get_current_token().pos;
        let node = self.equiality();
        
        if self.tokens.consume("=") {
            let rhs = self.equiality();
            match node {
                Expr::Var(_) => (),
                Expr::Deref(_) => (),
                _ => {
                    let e = anyhow!("left value is not assignable");
                    error_at(self.tokens.input, pos, e)
                }
            }
            Expr::Assign { 
                lhs: Box::new(node), 
                rhs: Box::new(rhs) 
            }
        } else {
            return node;
        }
    }

    /// `equiality = relational ( "==" relational | "!=" relational )*`
    fn equiality(&mut self) -> Expr {
        let mut node = self.relational();
        
        loop {
            if self.tokens.consume("==") {
                node = Expr::Binary { 
                    op: BinOp::Eq, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.relational()) 
                };
            } else if self.tokens.consume("!=") {
                node = Expr::Binary { 
                    op: BinOp::Ne, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.relational()) 
                };
            } else {
                return node;
            }
        }
    }
    
    /// `relational = add ( "<" add | "<=" add | ">" add | ">=" add )*`
    fn relational(&mut self) -> Expr {
        let mut node = self.add();
        
        // 長いトークンから見ていく
        loop {
            if self.tokens.consume("<=") {
                node = Expr::Binary { 
                    op: BinOp::Le, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.add()) 
                };
            } else if self.tokens.consume("<") {
                node = Expr::Binary { 
                    op: BinOp::Lt, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.add()) 
                };
            } else if self.tokens.consume(">=") {
                // 逆にするだけ
                node = Expr::Binary { 
                    op: BinOp::Le, 
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node), 
                };
            } else if self.tokens.consume(">") {
                // 逆にするだけ
                node = Expr::Binary { 
                    op: BinOp::Lt, 
                    lhs: Box::new(self.add()),
                    rhs: Box::new(node), 
                };
            } else {
                return node;
            }
        }
    }

    /// `add = mul ( "+" mul | "-" mul )*`
    fn add(&mut self) -> Expr {
        let mut node = self.mul();

        loop {
            if self.tokens.consume("+") {
                node = Expr::Binary { 
                    op: BinOp::Add, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.mul()) 
                };
            } else if self.tokens.consume("-") {
                node = Expr::Binary { 
                    op: BinOp::Sub, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.mul()) 
                };
            } else {
                return node;
            }
        }
    }

    /// `mul = unary ( "*" unary | "/" unary )*`
    fn mul(&mut self) -> Expr {
        let mut node = self.unary();

        loop {
            if self.tokens.consume("*") {
                node = Expr::Binary { 
                    op: BinOp::Mul, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.unary()) 
                };
            } else if self.tokens.consume("/") {
                node = Expr::Binary { 
                    op: BinOp::Div, 
                    lhs: Box::new(node), 
                    rhs: Box::new(self.unary()) 
                };
            } else {
                return node;
            }
        }
    }
    
    /// unary = "+" primary | 
    ///         "-" primary |
    ///         "&" unary |
    ///         "*" unary
    fn unary(&mut self) -> Expr {
        if self.tokens.consume("+") {
            self.primary()
        } else if self.tokens.consume("-") {
            // 一時的に 0-primary() の形で負の数を表す
            Expr::Binary { 
                op: BinOp::Sub,
                lhs: Box::new(Expr::Num(0)),
                rhs: Box::new(self.primary()),
            }
        } else if self.tokens.consume("&") {
            let pos = self.tokens.get_current_token().pos;
            let var = self.unary();
            match var {
                Expr::Var(_) => Expr::Addr(Box::new(var)),
                _ => {
                    let e = anyhow!("this cannot be refecenced");
                    error_at(self.tokens.input, pos, e)
                }
            }
        } else if self.tokens.consume("*") {
            let pos = self.tokens.get_current_token().pos;
            let addr = self.unary();
            // 参照外し可能か検証
            match addr {
                Expr::Deref(_) | Expr::Var(_) => Expr::Deref(Box::new(addr)),
                _ => {
                    let e = anyhow!("the value cannot be dereferenced");
                    error_at(self.tokens.input, pos, e)
                }
            }
        } else {
            self.primary()
        }
    }

    /// primary = num |
    ///            ident ( "(" params ")" )? |
    ///            "(" expr ")" 
    fn primary(&mut self) -> Expr {
        // "(" expr ")"
        if self.tokens.consume("(") {
            let expr = self.expr();
            match self.tokens.expect(")") {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Error While Parsing");
                    error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                }
            };
            return expr;
        }

        // ident ( args )?
        if let Some(ident) = self.tokens.consume_ident() {
            // 関数かどうか調べる
            let args;
            if self.tokens.consume("(") {
                // 定義済みか調べる
                if !self.defined_fn.contains(&ident.str) {
                    let e = anyhow!("定義されていない関数を呼び出しています");
                    error_at(self.tokens.input, self.tokens.get_current_token().pos, e);
                }
                args = self.args();
                return Expr::Call { fn_name: ident.str, args };
            }

            // ローカル変数が既にあるか調べる
            if let Some(lvar) = self.lvars.find_lvar(&ident) {
                return Expr::Var(lvar.name);
            } else {
                // ない場合は手前のに8を足して使う
                // TokenStreamの初期化時に先頭があるため
                // TODO: LVarでオフセットを扱う必要はない
                let offset = self.lvars.lvars_vec.last().unwrap().offset + 8;
                let lvar = LVar::new(ident.str, ident.len, offset);
                self.lvars.lvars_vec.push(lvar.clone());
                return Expr::Var(lvar.name);
            }
        }

        let num = match self.tokens.expect_number() {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error While Parsing");
                let e_unmatch = anyhow!("Error: unmatched `}}`");
                if "}" == self.tokens.get_current_token().str {
                    error_at(&self.tokens.input, self.tokens.get_current_token().pos, e_unmatch);
                } else {
                    error_at(&self.tokens.input, self.tokens.get_current_token().pos, e);
                }
            }
        };
        Expr::Num(num)
    }
    
}