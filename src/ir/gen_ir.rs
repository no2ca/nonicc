use crate::types::{ Node, NodeKind::* };
use crate::ir::types_ir::{ BinOp, Operand, ThreeAddressCode, VirtualReg };

pub struct GenIrContext {
    pub code: Vec<ThreeAddressCode>,
    register_count: usize,
}

impl GenIrContext {
    pub fn new() -> Self{
        GenIrContext {
            code: vec![],
            register_count: 0,
        }
    }
    fn get_register_count(&mut self) -> usize {
        let id = self.register_count;
        self.register_count += 1;
        id
    }
    
    fn emit(&mut self, instr: ThreeAddressCode) {
        self.code.push(instr);
    }
}

pub fn node_to_ir(node: &Node, context: &mut GenIrContext) -> VirtualReg {
    match node.kind {
        ND_NUM => {
            let val = node.val.unwrap();
            let id = context.get_register_count();
            let reg = VirtualReg::new(id);
            context.emit(ThreeAddressCode::LoadImm { dest: reg.clone(), value: val });
            reg
        }
        ND_ADD | ND_SUB | ND_MUL | ND_DIV => {
            let lhs  =node.lhs.as_ref().unwrap();
            let rhs = node.rhs.as_ref().unwrap();
            let left = node_to_ir(&lhs, context);
            let right = node_to_ir(&rhs, context);
            let id = context.get_register_count();
            let dest = VirtualReg::new(id);
            let op = match node.kind {
                ND_ADD => BinOp::Add,
                ND_SUB => BinOp::Sub,
                ND_MUL => BinOp::Mul,
                ND_DIV => BinOp::Div,
                _ => unreachable!(),
            };
            context.emit(ThreeAddressCode::BinOpCode {
                dest,
                left: Operand::Reg(left),
                op,
                right: Operand::Reg(right),
            });
            dest
        }
        _ => unimplemented!("{:?}", node.kind),
    }
}
