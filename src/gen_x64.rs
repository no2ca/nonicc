use std::collections::HashMap;
use crate::ir::types_ir::{BinOp, Label, Operand, ThreeAddressCode as TAC, VirtualReg};

pub struct Generator<'a> {
    regs: Vec<&'a str>,
    code: Vec<TAC>,
    pub vreg_to_offset: HashMap<VirtualReg, usize>,
}

impl<'a> Generator<'a> {
    /// ここで <変数名:仮想レジスタ> のHashMapを <仮想レジスタ:オフセット> のHashMapに昇順で変換
    /// TODO: 責務を分離して単体テストやる
    pub fn new(regs: Vec<&'a str>, code: Vec<TAC>, lvar_map: HashMap<String, VirtualReg>) -> Generator<'a> {
        // 変数名:仮想レジスタのHashMapを受け取る
        let mut vec = Vec::new();
        for x in lvar_map {
            vec.push(x);
        }

        // 昇順にする
        vec.sort_by_key(|i| i.1.id);
        
        // オフセットを計算
        let mut map = HashMap::new();
        let mut offset = 0;
        for (_, vreg) in vec {
            map.entry(vreg).or_insert(offset + 8);
            offset += 8;
        }

        Generator {
            regs,
            code,
            vreg_to_offset: map,
        }
    }
    
    /// アセンブリ生成はここから
    pub fn gen_all(&self, vreg_to_reg: &HashMap<VirtualReg, usize>) {
        for instr in &self.code {
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
                let left_reg = self.vreg_to_string(left, vreg_to_reg);
                let right_reg = self.vreg_to_string(right, vreg_to_reg);
                let dest_reg = self.vreg_to_string(dest, vreg_to_reg);
                match op {
                    BinOp::Add => {
                        if dest_reg == right_reg {
                            // TODO: これは壊れないんですか？
                            let tmp = "rbx";
                            println!("  mov {}, {}", tmp, left_reg);
                            println!("  add {}, {}", tmp, right_reg);
                            println!("  mov {}, {}", dest_reg, tmp);
                        } else {
                            println!("  mov {}, {}", dest_reg, left_reg);
                            println!("  add {}, {}", dest_reg, right_reg);
                        }
                    }
                    BinOp::Sub => {
                        if dest_reg == right_reg {
                            // TODO: これは壊れないんですか？
                            let tmp = "rbx";
                            println!("  mov {}, {}", tmp, left_reg);
                            println!("  sub {}, {}", tmp, right_reg);
                            println!("  mov {}, {}", dest_reg, tmp);
                        } else {
                            println!("  mov {}, {}", dest_reg, left_reg);
                            println!("  sub {}, {}", dest_reg, right_reg);
                        }
                    }
                    BinOp::Mul => {
                        if dest_reg == right_reg {
                            // TODO: これは壊れないんですか？
                            let tmp = "rbx";
                            println!("  mov {}, {}", tmp, left_reg);
                            println!("  imul {}, {}", tmp, right_reg);
                            println!("  mov {}, {}", dest_reg, tmp);
                        } else {
                            println!("  mov {}, {}", dest_reg, left_reg);
                            println!("  imul {}, {}", dest_reg, right_reg);
                        }
                    }
                    BinOp::Div => {
                        let tmp = "rbx";
                        println!("  mov {}, rdx", tmp);

                        // raxの値が割られる数
                        println!("  mov rax, {}", left_reg);
                        // raxを128bitに拡張してこれだけ使う
                        println!("  cqo");
                        if right_reg == "rdx".to_string() {
                            println!("  idiv {}", tmp);
                        } else {
                            println!("  idiv {}", right_reg);
                        }
                        // raxの値が商になる
                        println!("  mov {}, rax", dest_reg);
                        println!("  mov rdx, {}", tmp);
                    }
                    BinOp::Le => {
                        println!("  cmp {}, {}", left_reg, right_reg);
                        println!("  setle al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Lt => {
                        println!("  cmp {}, {}", left_reg, right_reg);
                        println!("  setl al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Eq => {
                        println!("  cmp {}, {}", left_reg, right_reg);
                        println!("  sete al");
                        println!("  movzb {}, al", dest_reg);
                    }
                    BinOp::Ne => {
                        println!("  cmp {}, {}", left_reg, right_reg);
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
            // EvalVarは仮想レジスタの生存期間を明示的に扱うためなので何も生成しない
            TAC::EvalVar { .. } => (),
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
            TAC::Call { fn_name, args, ret_reg } => {
                let mut args_reg = Vec::new();
                for arg in args {
                    args_reg.push(self.vreg_to_string(arg, vreg_to_reg));
                }

                /*
                for arg in &args_reg {
                    println!("  push {}", arg);
                }
                */
                
                let save = vec!["rbx", "r12", "r13", "r14", "r15"];
                let regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for r in &regs {
                    println!("  push {}", r);
                }
                for (i, arg) in args_reg.iter().enumerate() {
                    let s = save.get(i).expect("too many args are given");
                    println!("  mov {}, {}", s, arg);
                }
                
                for i in 0..args.len() {
                    let dest = regs.get(i).expect("too many args are given");
                    let s = save.get(i).expect("too many args are given");
                    println!("  mov {}, {}", dest, s);
                }

                println!("  call {}", fn_name);
                
                for r in regs.iter().rev() {
                    println!("  pop {}", r);
                }

                let ret_val_reg = self.vreg_to_string(ret_reg, vreg_to_reg);
                println!("  mov {}, rax", ret_val_reg);
            }
            TAC::Fn { fn_name, params } => {
                // 最大オフセットを使用してスタックサイズを計算
                let mut offset_max: usize = 0; 
                for (_, offset) in self.vreg_to_offset.clone() {
                    if offset > offset_max {
                        offset_max = offset;
                    }
                }
                let stack_size = ((offset_max + 15) / 16) * 16;
                
                // 関数プロローグ
                println!("{}:", fn_name);
                println!("  push rbp");
                println!("  mov rbp, rsp");
                println!("  sub rsp, {}", stack_size);

                // 引数の受け渡し(Linux)
                // OSによってルールが異なることに注意
                // 代入前に値が壊れてしまうことがあるためスタックに一時保存
                // Maybe: callee-saveに一時保存する手がある
                // let save = vec!["rbx", "r12", "r13", "r14", "r15"];
                let param_regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for i in 0..params.len() {
                    println!("  push {}", param_regs.get(i).expect("too many args"));
                }
                for param in params.iter().rev() {
                    let dest = self.vreg_to_string(&param.dest, vreg_to_reg);
                    println!("  pop {}", dest);
                }
            }
            // ワイルドカードを使わない
        }
    }
}
