use std::collections::HashMap;
use crate::ir::types_ir::{BinOp, Label, Operand, ThreeAddressCode as TAC, VirtualReg};

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
                let msg = format!("Missing vreg key '{:?}' in 'vreg_to_reg'", vreg);
                let reg_idx = vreg_to_reg.get(vreg).expect(&msg).clone();

                let msg = format!("vreg_to_reg returned '{:?}' which is out of range", reg_idx);
                self.regs.get(reg_idx).expect(&msg).to_string()
            }
        }
    }
    
    fn label_to_string(&self, label: Label) -> String {
        match label {
            Label::Lelse(count) => {
                format!(".Lelse{count}")
            }
            Label::Lend(count) => {
                format!(".Lend{count}")
            }
        }
    }
    
    /// 仮想レジスタを受け取って実際のレジスタ名をStringで返す
    fn vreg_to_string(&self, vreg: &VirtualReg, vreg_to_reg: &HashMap<VirtualReg, usize>) -> String {
        let msg = format!("Missing vreg key '{:?}' in 'vreg_to_reg'", vreg);
        let reg_idx = vreg_to_reg.get(vreg).expect(&msg).clone();

        let msg = format!("vreg_to_reg returned '{:?}' which is out of range", reg_idx);
        self.regs.get(reg_idx).expect(&msg).to_string()
    }
    
    fn generate(&self, vreg_to_reg: &HashMap<VirtualReg, usize>, instr: &TAC) {
        match instr {
            TAC::LoadImm { dest, value} => {
                let dest_reg_idx = vreg_to_reg.get(dest).unwrap().clone();
                println!("  mov {}, {}", self.regs[dest_reg_idx], value);            
            }
            TAC::BinOpCode { dest, left, op, right } => {
                let left_operand = self.operand_to_string(left, vreg_to_reg);
                let right_operand = self.operand_to_string(right, vreg_to_reg);
                let dest_reg = self.vreg_to_string(dest, vreg_to_reg);
                match op {
                    BinOp::Add => {
                        println!("  add {}, {}", left_operand, right_operand);
                        if dest_reg != left_operand {
                            println!("  mov {}, {}", dest_reg, left_operand);
                        }
                    }
                    BinOp::Sub => {
                        println!("  sub {}, {}", left_operand, right_operand);
                        if dest_reg != left_operand {
                            println!("  mov {}, {}", dest_reg, left_operand);
                        }
                    }
                    BinOp::Mul => {
                        println!("  imul {}, {}", left_operand, right_operand);
                        if dest_reg != left_operand {
                            println!("  mov {}, {}", dest_reg, left_operand);
                        }
                    }
                    BinOp::Div => {
                        // raxの値が割られる数
                        println!("  mov rax, {}", left_operand);
                        // raxを128bitに拡張してこれだけ使う
                        println!("  cqo");
                        println!("  idiv {}", right_operand);
                        // raxの値が商になる
                        println!("  mov {}, rax", dest_reg);
                    }
                    BinOp::Le => {
                        println!("  cmp {}, {}", left_operand, right_operand);
                        println!("  setle al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Lt => {
                        println!("  cmp {}, {}", left_operand, right_operand);
                        println!("  setl al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Eq => {
                        println!("  cmp {}, {}", left_operand, right_operand);
                        println!("  sete al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Ne => {
                        println!("  cmp {}, {}", left_operand, right_operand);
                        println!("  setne al");
                        println!("  movzb {}, al", dest_reg);
                    }
                }
            }
            TAC::Assign { dest, src } => {
                let dest_reg = self.vreg_to_string(dest, vreg_to_reg);
                let src_reg = self.operand_to_string(src, vreg_to_reg);
                println!("  mov {dest_reg}, {src_reg}");
            }
            TAC::LoadVar { .. } => (),
            TAC::Return { src } => {
                let src_reg = self.vreg_to_string(src, vreg_to_reg);
                println!("  mov rax, {}", src_reg);
                
                // 関数エピローグ
                println!("  mov rsp, rbp");
                println!("  pop rbp");
                println!("  ret");
            }
            TAC::IfFalse { cond, label } => {
                let cond_reg = self.vreg_to_string(cond, vreg_to_reg);
                let real_label = self.label_to_string(label.clone());
                println!("  cmp {}, 0", cond_reg);
                println!("  je {}", real_label);
            }
            TAC::GoTo { label } => {
                let real_label = self.label_to_string(label.clone());
                println!("  jmp {}", real_label);
            }
            TAC::Label { label } => {
                let real_label = self.label_to_string(label.clone());
                println!("{}:", real_label);
            }
            // ワイルドカードを使わない
        }
    }
}
