use std::collections::HashMap;

use crate::types::{ BinOp, Expr, Stmt };
use crate::ir::types_ir::{ BinOp as IrBinOp, ThreeAddressCode as TAC, VirtualReg, Label, Param };

#[derive(Clone)]
pub struct GenIrContext {
    code: Vec<TAC>,
    register_count: usize,
    pub label_count: usize,
    lvar_map: HashMap<String, VirtualReg>,
}

impl GenIrContext {
    pub fn new() -> Self{
        GenIrContext {
            code: vec![],
            register_count: 0,
            label_count: 0,
            lvar_map: HashMap::new(),
        }
    }
    
    /// - 中間表現のコードの入った配列を取得する
    /// - mainで使用
    pub fn get_ir_code(&self) -> Vec<TAC> {
        self.code.clone()
    }
    
    /// - 変数名と仮想レジスタのHashMapを取得する
    /// - mainで使用
    pub fn get_lvar_map(&self) -> HashMap<String, VirtualReg> {
        self.lvar_map.clone()
    }

    /// 新しい仮想レジスタを作る
    fn get_new_register(&mut self) -> VirtualReg {
        let id = self.register_count;
        self.register_count += 1;
        VirtualReg { id }
    }
    
    /// - HashMapを使用して既にレジスタが割り当てられているか調べる
    /// - 既存の割り当てが無かったら新たにレジスタを作る
    fn get_var_reg(&mut self, name: &str) -> VirtualReg {
        if let Some(&reg) = self.lvar_map.get(name) {
            reg
        } else {
            let reg = self.get_new_register();
            self.lvar_map.insert(name.to_string(), reg);
            reg
        }
    }
    
    /// ラベルの番号を返す
    fn get_label_count(&mut self) -> usize {
        let i = self.label_count;
        self.label_count += 1;
        i
    }
    
    fn emit(&mut self, instr: TAC) {
        self.code.push(instr);
    }
}

pub fn stmt_to_ir(stmt: &Stmt, context: &mut GenIrContext) {
    // stmt_to_irは文を生成するとき
    // expr_to_irは式を生成して値の入ったレジスタを受け取るとき
    match stmt {
        Stmt::Return(expr) => {
            let src = expr_to_ir(expr, context);
            context.emit(TAC::Return { src });
        }
        Stmt::While { cond: _cond, body: _body } => {
            // begin:
            //   if (a == 0)
            //     goto end;
            //   <stmt>
            //   goto begin;
            // end:
            let begin = Label::Lbegin(context.get_label_count());
            context.emit(TAC::Label { label: begin.clone() });
            // cond(expr)に条件
            let cond = expr_to_ir(_cond, context);
            let end = Label::Lend(context.get_label_count());
            context.emit(TAC::IfFalse { cond, label: end.clone() });
            // body(stmt)に処理
            stmt_to_ir(_body, context);
            context.emit(TAC::GoTo { label: begin });
            // Lendラベル
            context.emit(TAC::Label { label: end });
        }
        Stmt::For { init: _init, cond: _cond, update: _update, body: _body } => {
            // // for (init; cond; update) body;
            // init;
            // begin:
            //     if (cond = 0)
            //         goto end;
            //     body;
            //     update;
            //     goto begin;
            // end:
            
            // init
            if let Some(init) = _init {
                expr_to_ir(init, context);
            }
            
            // begin
            let begin = Label::Lbegin(context.get_label_count());
            context.emit(TAC::Label { label: begin.clone() });
            
            // 終了判定
            let end = Label::Lend(context.get_label_count());
            if let Some(_cond) = _cond {
                let cond = expr_to_ir(_cond, context);
                context.emit(TAC::IfFalse { cond, label: end.clone() });
            }
            
            // body
            stmt_to_ir(_body, context);
            if let Some(update) = _update {
                expr_to_ir(update, context);
            }
            context.emit(TAC::GoTo { label: begin });
            context.emit(TAC::Label { label: end });
        }
        Stmt::If { cond: _cond, then, els: _els } => {
            //   if (cond == 0)
            //     goto Lelse;
            //   then;
            //   goto Lend;
            // Lelse:
            //     els;
            // Lend:
            let cond = expr_to_ir(_cond, context);
            // elseがあるとき
            if let Some(els) = _els {
                let label_else = Label::Lelse(context.get_label_count());
                context.emit(TAC::IfFalse { 
                    cond,
                    label: label_else.clone()
                });

                stmt_to_ir(then, context);
                
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::GoTo { label: label_end.clone() });

                context.emit(TAC::Label { label: label_else });
                stmt_to_ir(els, context);
                
                context.emit(TAC::Label { label: label_end });
            } else {
                // elseが無いとき
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::IfFalse { 
                    cond,
                    label: label_end.clone()
                });
                stmt_to_ir(then, context);
                context.emit(TAC::Label { label: label_end });
            }
        }
        Stmt::Block(stmts) => {
            for stmt in stmts {
                stmt_to_ir(stmt, context);
            }
        }
        Stmt::Fn { fn_name, params: _params, body } => {
            let mut params = Vec::new();
            for param in _params {
                let name = match param {
                    Expr::Var(name) => name.to_owned(),
                    _ => unreachable!("parameter should be identifier but got {:?}", param)
                };
                let dest = context.get_var_reg(&name);
                params.push(Param::new(dest, name));
            }

            context.emit(TAC::Fn { fn_name: fn_name.clone(), params });
            
            for stmt in body {
                stmt_to_ir(stmt, context);
            }
        }
        Stmt::ExprStmt(expr) => {
            expr_to_ir(expr, context);
        }
        Stmt::VarDecl { name, ..  } => {
            context.get_var_reg(name);
        }
    }

}

fn gen_lval_addr(expr: &Expr, context: &mut GenIrContext) -> VirtualReg {
    match expr {
        Expr::Deref(_var) => {
            let addr = gen_lval_addr(&_var, context);
            match &**_var {
                // 参照外しが続いているとき
                // **pp はまず *pp (LoadVar) をする
                Expr::Deref(_) => {
                    let value = context.get_new_register();
                    context.emit(TAC::LoadVar { value, addr });
                    value
                }
                _ => {
                    addr
                }
            }
            
        }
        Expr::Var(name) => {
            let dest = context.get_var_reg(&name);
            context.emit(TAC::EvalVar { 
                dest, 
                name: name.to_string()
            });
            dest
        }
        _ => unreachable!(),
    }
}

fn expr_to_ir(expr: &Expr, context: &mut GenIrContext) -> VirtualReg {
    match expr {
        Expr::Assign { lhs, rhs } => {
            let src = expr_to_ir(rhs, context);
            
            match &**lhs {
                Expr::Deref(_) => {
                    let addr = gen_lval_addr(&lhs, context);
                    context.emit(TAC::Store { addr, src });
                }
                Expr::Var(name) => {
                    let dest = context.get_var_reg(name);
                    context.emit(TAC::Assign { 
                        dest, 
                        src,
                    });
                }
                _ => unreachable!("left value got not assingnable node: {:?}", lhs),
            }
            return src;
        }
        Expr::Num(val) => {
            let reg = context.get_new_register();
            context.emit(TAC::LoadImm { dest: reg.clone(), value: *val });
            reg
        }
        Expr::Binary { op: _op, lhs, rhs } => {
            // ここは即値入れるなどの最適化しない
            // divとそれ以外で場合分けが発生して面倒なことになる
            // 単一責務
            let left_operand = expr_to_ir(&lhs, context);
            let right_operand = expr_to_ir(&rhs, context);

            let dest_vreg = context.get_new_register();
            let op = match _op {
                BinOp::Add => IrBinOp::Add,
                BinOp::Sub => IrBinOp::Sub,
                BinOp::Mul => IrBinOp::Mul,
                BinOp::Div => IrBinOp::Div,
                BinOp::Le => IrBinOp::Le,
                BinOp::Lt => IrBinOp::Lt,
                BinOp::Eq => IrBinOp::Eq,
                BinOp::Ne => IrBinOp::Ne,
            };
            context.emit(TAC::BinOpCode {
                dest: dest_vreg,
                left: left_operand,
                op,
                right: right_operand,
            });
            dest_vreg
        }
        Expr::Var(name) => {
            let dest = context.get_var_reg(&name);
            context.emit(TAC::EvalVar { 
                dest, 
                name: name.clone()
            });
            dest
        }
        Expr::Addr(_name) => {
            // TODO: 型検証を導入するまではpanicする
            // nameフィールドを埋めているのが変数名であること
            let name = match &**_name {
                Expr::Var(n) => n,
                _ => unreachable!("Addr has value that is not able to referenced (it should be a bug in parser!)")
            };
            let var = context.get_var_reg(&name);
            let addr = context.get_new_register();
            context.emit(TAC::AddrOf { addr, var });
            addr
        }
        Expr::Deref(deref) => {
            let dest = context.get_new_register();
            let addr = expr_to_ir(&deref, context);
            context.emit(TAC::LoadVar { value: dest, addr });
            dest
        }
        Expr::Call { fn_name, args: _args } => {
            let mut args = Vec::new();
            for arg in _args {
                args.push(expr_to_ir(&arg, context));
            }
            let ret_reg = context.get_new_register();
            context.emit(TAC::Call { fn_name: fn_name.clone(), args, ret_reg });
            ret_reg
        }
    }
}
