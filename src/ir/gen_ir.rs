use std::collections::HashMap;

use crate::types::{ Node, NodeKind::* };
use crate::ir::types_ir::{ BinOp, ThreeAddressCode as TAC, VirtualReg, Label, Param };

#[derive(Clone)]
pub struct GenIrContext {
    pub code: Vec<TAC>,
    register_count: usize,
    label_count: usize,
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
    
    pub fn get_lvar_map(&self) -> HashMap<String, VirtualReg> {
        self.lvar_map.clone()
    }

    fn get_register_count(&mut self) -> usize {
        let id = self.register_count;
        self.register_count += 1;
        id
    }
    
    /// HashMapを使用して既にレジスタが割り当てられているか調べる
    /// 既存の割り当てが無かったら新たにレジスタを作る
    fn get_var_reg(&mut self, name: &str) -> VirtualReg {
        if let Some(&reg) = self.lvar_map.get(name) {
            reg
        } else {
            let id = self.get_register_count();
            let reg = VirtualReg { id };
            self.lvar_map.insert(name.to_string(), reg);
            reg
        }
    }
    
    fn get_label_count(&mut self) -> usize {
        let i = self.label_count;
        self.label_count += 1;
        i
    }
    
    fn emit(&mut self, instr: TAC) {
        self.code.push(instr);
    }
}

pub fn stmt_to_ir(node: &Node, context: &mut GenIrContext) {
    // stmt_to_irは文を生成するとき
    // expr_to_irは式を生成して値の入ったレジスタを受け取るとき
    match node.kind {
        ND_ASSIGN => {
            let lhs = node.lhs.as_ref().unwrap();
            let rhs = node.rhs.as_ref().unwrap();
            
            let left_vreg = expr_to_ir(&lhs, context);
            let right_vreg = expr_to_ir(&rhs, context);
            let dest_vreg = left_vreg;

            context.emit(TAC::Assign { 
                dest: dest_vreg, 
                src: right_vreg 
            });
        }
        ND_RETURN => {
            // lhsにexprが入っている
            let lhs = node.lhs.as_ref().unwrap();
            let src_vreg = expr_to_ir(lhs, context);
            context.emit(TAC::Return { src: src_vreg });
        }
        ND_IF => {
            let cond_node = node.cond.as_ref().unwrap();
            let cond = expr_to_ir(cond_node, context);

            // elseがあるとき
            if let Some(_els) = node.els.as_ref() {
                // 条件分岐
                let label_else = Label::Lelse(context.get_label_count());
                
                // if_false goto .Lelse
                context.emit(TAC::IfFalse { 
                    cond,
                    label: label_else.clone()
                });

                // then
                let then_node = node.then.as_ref().unwrap();                
                stmt_to_ir(then_node, context);
                
                // goto .Lend
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::GoTo { label: label_end.clone() });

                // else
                context.emit(TAC::Label { label: label_else });
                stmt_to_ir(_els, context);
                
                // .Lend
                context.emit(TAC::Label { label: label_end });
            } else {
                // elseが無いとき
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::IfFalse { 
                    cond,
                    label: label_end.clone()
                });
                let then_node = node.then.as_ref().unwrap();
                stmt_to_ir(then_node, context);
                context.emit(TAC::Label { label: label_end });
            }
        }
        ND_BLOCK => {
            let stmts = node.stmts.as_ref().unwrap();
            for stmt in stmts {
                stmt_to_ir(stmt, context);
            }
        }
        ND_DEFUN => {
            let fn_name = node.fn_name.clone().unwrap();
            
            let mut params = Vec::new();
            for param in node.params.as_ref().unwrap() {
                let name = param.ident_name.clone().unwrap();
                let dest = context.get_var_reg(&name);
                params.push(Param::new(dest, name));
            }

            context.emit(TAC::Fn { fn_name, params });
            
            let stmts = node.stmts.as_ref().unwrap();
            for stmt in stmts {
                stmt_to_ir(stmt, context);
            }
        }
        _ => {
            expr_to_ir(node, context);
        }
    }

}

pub fn expr_to_ir(node: &Node, context: &mut GenIrContext) -> VirtualReg {
    match node.kind {
        ND_NUM => {
            let val = node.val.unwrap();
            let id = context.get_register_count();
            let reg = VirtualReg::new(id);
            context.emit(TAC::LoadImm { dest: reg.clone(), value: val });
            reg
        }
        ND_ADD | ND_SUB | ND_MUL | ND_DIV | ND_LE | ND_LT | ND_EQ | ND_NE => {
            let lhs = node.lhs.as_ref().unwrap();
            let rhs = node.rhs.as_ref().unwrap();

            // ここは最適化しない
            // divとそれ以外で場合分けが発生して面倒なことになる
            // 単一責務
            let left_operand = expr_to_ir(&lhs, context);
            let right_operand = expr_to_ir(&rhs, context);

            let id = context.get_register_count();
            let dest_vreg = VirtualReg::new(id);
            let op = match node.kind {
                ND_ADD => BinOp::Add,
                ND_SUB => BinOp::Sub,
                ND_MUL => BinOp::Mul,
                ND_DIV => BinOp::Div,
                ND_LE => BinOp::Le,
                ND_LT => BinOp::Lt,
                ND_EQ => BinOp::Eq,
                ND_NE => BinOp::Ne,
                _ => unreachable!(),
            };
            context.emit(TAC::BinOpCode {
                dest: dest_vreg,
                left: left_operand,
                op,
                right: right_operand,
            });
            dest_vreg
        }
        ND_LVAR => {
            let name = node.ident_name.clone().unwrap();
            let dest_vreg = context.get_var_reg(&name);
            context.emit(TAC::EvalVar { 
                dest: dest_vreg, 
                name
            });
            dest_vreg
        }
        ND_CALL => {
            let fn_name = node.fn_name.clone().unwrap();
            let mut args = Vec::new();
            for arg in node.args.clone().unwrap() {
                args.push(expr_to_ir(&arg, context));
            }
            let ret_reg = VirtualReg::new(context.get_register_count());
            context.emit(TAC::Call { fn_name, args, ret_reg });
            ret_reg
        }
        _ => unreachable!("{:?}", node.kind),
    }
}
