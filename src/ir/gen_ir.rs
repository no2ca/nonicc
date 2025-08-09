use std::collections::HashMap;

use crate::types::{ Node, NodeKind::* };
use crate::ir::types_ir::{ BinOp, Operand, ThreeAddressCode as TAC, VirtualReg };

pub struct GenIrContext {
    pub code: Vec<TAC>,
    register_count: usize,
    var_map: HashMap<String, VirtualReg>,
}

impl GenIrContext {
    pub fn new() -> Self{
        GenIrContext {
            code: vec![],
            register_count: 0,
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
    
    fn emit(&mut self, instr: TAC) {
        self.code.push(instr);
    }
}

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

            let left_vreg = node_to_ir(&lhs, context);
            let right_vreg = node_to_ir(&rhs, context);

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
                left: Operand::Reg(left_vreg),
                op,
                right: Operand::Reg(right_vreg),
            });
            dest_vreg
        }
        ND_ASSIGN => {
            let lhs = node.lhs.as_ref().unwrap();
            let rhs = node.rhs.as_ref().unwrap();
            
            let left_vreg = node_to_ir(&lhs, context);
            let right_vreg = node_to_ir(&rhs, context);

            let dest_vreg = left_vreg;

            context.emit(TAC::Assign { 
                dest: dest_vreg, 
                src: Operand::Reg(right_vreg) 
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
        _ => unimplemented!("{:?}", node.kind),
    }
}
