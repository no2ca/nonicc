use std::collections::HashMap;
use crate::ir::types_ir::{ThreeAddressCode as TAC, VirtualReg, BinOp, Operand};

pub struct Generator<'a> {
    regs: Vec<&'a str>,
    codes: Vec<TAC>,
}

impl<'a> Generator<'a> {
    pub fn new(regs: Vec<&'a str>, codes: Vec<TAC>) -> Generator<'a> {
        Generator {
            regs,
            codes,
        }
    }
    
    pub fn gen_all(&self, vreg_to_reg: &HashMap<VirtualReg, usize>) {
        for instr in &self.codes {
            self.generate(&vreg_to_reg, instr);
        }
    }
    
    fn operand_to_string(&self, operand: &Operand, vreg_to_reg: &HashMap<VirtualReg, usize>) -> String {
        match operand {
            Operand::Imm(val) => format!("{}", val),
            Operand::Reg(vreg) => {
                let reg_id = vreg_to_reg.get(&vreg).unwrap().clone();
                format!("{}", self.regs[reg_id])
            }
        }
    }
    
    fn generate(&self, vreg_to_reg: &HashMap<VirtualReg, usize>, instr: &TAC) {
        match instr {
            TAC::LoadImm { dest, value} => {
                let dest_reg_idx = vreg_to_reg.get(dest).unwrap().clone();
                println!("  mov {}, {}", self.regs[dest_reg_idx], value);            
                println!("  mov rax, {}", self.regs[dest_reg_idx]);
            }
            TAC::BinOpCode { dest, left, op, right } => {
                let lft = self.operand_to_string(left, vreg_to_reg);
                let rgt = self.operand_to_string(right, vreg_to_reg);
                let dest_reg_idx = vreg_to_reg.get(dest).unwrap().clone();
                match op {
                    BinOp::Add => {
                        println!("  add {}, {}", lft, rgt);
                        println!("  mov {}, {}", self.regs[dest_reg_idx], lft);
                        println!("  mov rax, {}", self.regs[dest_reg_idx]);
                    }
                    BinOp::Sub => {
                        println!("  sub {}, {}", lft, rgt);
                        println!("  mov {}, {}", self.regs[dest_reg_idx], lft);
                        println!("  mov rax, {}", self.regs[dest_reg_idx]);
                    }
                    _ => unimplemented!("{:?}", op)
                }
            }
            // _ => unimplemented!("{:?}", instr)
        }
    }
}
