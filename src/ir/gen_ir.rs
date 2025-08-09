use std::collections::HashMap;

use crate::types::{ Node, NodeKind::* };
use crate::ir::types_ir::{ BinOp, Operand, ThreeAddressCode as TAC, VirtualReg, Label };

pub struct GenIrContext {
    pub code: Vec<TAC>,
    register_count: usize,
    label_count: usize,
    var_map: HashMap<String, VirtualReg>,
}

impl GenIrContext {
    pub fn new() -> Self{
        GenIrContext {
            code: vec![],
            register_count: 0,
            label_count: 0,
            var_map: HashMap::new(),
        }
    }

    fn get_register_count(&mut self) -> usize {
        let id = self.register_count;
        self.register_count += 1;
        id
    }
    
    /// HashMapを使用して既にレジスタが割り当てられているか調べる
    fn get_var_reg(&mut self, name: &str) -> VirtualReg {
        if let Some(&reg) = self.var_map.get(name) {
            reg
        } else {
            let id = self.get_register_count();
            let r = VirtualReg { id };
            self.var_map.insert(name.to_string(), r);
            r
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

/// TODO: レジスタ返す必要はないのも返しているので要修正
pub fn node_to_ir(node: &Node, context: &mut GenIrContext) -> VirtualReg {
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
            let left_operand = Operand::Reg(node_to_ir(&lhs, context));
            let right_operand = Operand::Reg(node_to_ir(&rhs, context));

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
        ND_ASSIGN => {
            let lhs = node.lhs.as_ref().unwrap();
            let rhs = node.rhs.as_ref().unwrap();
            
            let left_vreg = node_to_ir(&lhs, context);

            // ここは即値を代入するだけなので大丈夫
            let right_operand = if rhs.kind == ND_NUM {
                Operand::Imm(rhs.val.unwrap())
            } else {
                Operand::Reg(node_to_ir(&rhs, context))
            };

            let dest_vreg = left_vreg;

            context.emit(TAC::Assign { 
                dest: dest_vreg, 
                src: right_operand 
            });
            
            dest_vreg
        }
        ND_LVAR => {
            let name = node.ident_name.clone().unwrap();
            let dest_vreg = context.get_var_reg(&name);
            context.emit(TAC::LoadVar { 
                dest: dest_vreg, 
                var: name
            });
            dest_vreg
        }
        ND_RETURN => {
            // lhsにexprが入っている
            let lhs = node.lhs.as_ref().unwrap();
            let src_vreg = node_to_ir(lhs, context);

            context.emit(TAC::Return { src: src_vreg });
            
            src_vreg
        }
        ND_IF => {
            let cond_node = node.cond.as_ref().unwrap();
            let cond = node_to_ir(cond_node, context);

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
                let _then = node_to_ir(then_node, context);
                
                // goto .Lend
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::GoTo { label: label_end.clone() });

                // else
                context.emit(TAC::Label { label: label_else });
                let _els = node_to_ir(_els, context);
                
                // .Lend
                context.emit(TAC::Label { label: label_end });
                
                _els

            } else {
                // elseが無いとき
                let label_end = Label::Lend(context.get_label_count());
                context.emit(TAC::IfFalse { 
                    cond,
                    label: label_end.clone()
                });
                let then_node = node.then.as_ref().unwrap();
                let _then = node_to_ir(then_node, context);
                context.emit(TAC::Label { label: label_end });
                _then
            }
        }
        _ => unimplemented!("{:?}", node.kind),
    }
}
