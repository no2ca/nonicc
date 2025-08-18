use std::collections::HashMap;
use crate::ir::types_ir::{BinOp, Label, ThreeAddressCode as TAC, VirtualReg};

pub struct Generator<'a> {
    regs: Vec<&'a str>,
    code: Vec<TAC>,
    pub vreg_to_offset: HashMap<VirtualReg, usize>,
}

impl<'a> Generator<'a> {
    /// ここで <変数名:仮想レジスタ> のHashMapを <仮想レジスタ:オフセット> のHashMapに昇順で変換
    pub fn new(regs: Vec<&'a str>, code: Vec<TAC>, lvar_map: HashMap<String, VirtualReg>) -> Generator<'a> {
        // 変数名:仮想レジスタのHashMapをVecに受け取る
        let mut vec = Vec::new();
        for x in lvar_map {
            vec.push(x);
        }

        // 昇順にする
        vec.sort_by_key(|(_, vreg)| vreg.id);
        
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
    pub fn gen_fn(&self, vreg_to_reg: HashMap<VirtualReg, usize>) {
        for instr in &self.code {
            self.generate(&vreg_to_reg, instr);
        }
    }
    
    fn label_to_string(&self, label: Label) -> String {
        match label {
            Label::Lelse(count) => {
                format!(".Lelse{count}")
            }
            Label::Lbegin(count) => {
                format!(".Lbegin{count}")
            }
            Label::Lend(count) => {
                format!(".Lend{count}")
            }
        }
    }
    
    /// 仮想レジスタを受け取って実際のレジスタ名をStringで返す
    fn vreg_to_string(&self, vreg: &VirtualReg, vreg_to_reg: &HashMap<VirtualReg, usize>) -> String {
        let msg = format!("Missing vreg key '{:?}' in 'vreg_to_reg'", vreg);
        let reg_idx = vreg_to_reg.get(vreg).expect(&msg);

        let msg = format!("vreg_to_reg returned '{:?}' which is out of range", reg_idx);
        self.regs.get(*reg_idx).expect(&msg).to_string()
    }
    
    fn get_offset(&self, vreg: &VirtualReg) -> usize {
        let msg = format!("Missing vreg key '{:?}' in 'vreg_to_offset'", vreg);
        let offset = self.vreg_to_offset.get(vreg).expect(&msg);
        *offset
    }
    
    fn generate(&self, vreg_to_reg: &HashMap<VirtualReg, usize>, instr: &TAC) {
        match instr {
            TAC::LoadImm { dest, value} => {
                let dest_reg_idx = vreg_to_reg.get(dest).unwrap();
                println!("  mov {}, {}", self.regs[*dest_reg_idx], value);            
            }
            TAC::BinOpCode { dest, left, op, right } => {
                let left_reg = self.vreg_to_string(left, vreg_to_reg);
                let right_reg = self.vreg_to_string(right, vreg_to_reg);
                let dest_reg = self.vreg_to_string(dest, vreg_to_reg);
                // 変数のときはレジスタに最新の値をロードする
                if let Some(offset) = self.vreg_to_offset.get(left) {
                    println!("  mov {}, [rbp - {}]", left_reg, offset);
                }
                if let Some(offset) = self.vreg_to_offset.get(right) {
                    println!("  mov {}, [rbp - {}]", right_reg, offset);
                }
                match op {
                    BinOp::Add => {
                        if dest_reg == right_reg {
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
                        // rdxの値を避難させる
                        // いつでも符号拡張で壊れる可能性があるため常に行う
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
                        
                        // rdxの値を復活させる
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
                let src_reg = self.vreg_to_string(src, vreg_to_reg);
                let offset = self.get_offset(dest);
                println!("  mov {}, {}", dest_reg, src_reg);
                println!("  mov [rbp - {}], {}", offset, src_reg);
            }
            TAC::EvalVar { .. } => {
                // let offset = self.get_offset(var);
                // let var_reg = self.vreg_to_string(var, vreg_to_reg);
                // println!("  mov {}, [rbp - {}]", var_reg, offset);
            }
            TAC::AddrOf { addr, var } => {
                // 参照
                let offset = self.get_offset(var);
                let addr_reg = self.vreg_to_string(addr, vreg_to_reg);
                println!("  lea {}, [rbp - {}]", addr_reg, offset);
            }
            TAC::LoadVar { dest, addr } => {
                // 参照外し
                let dest_reg = self.vreg_to_string(dest, vreg_to_reg);
                let addr_reg = self.vreg_to_string(addr, vreg_to_reg);
                // 変数のときは最新の値をロードしてから
                if let Some(offset) = self.vreg_to_offset.get(addr) {
                    println!("  mov {}, [rbp - {}]", addr_reg, offset);
                }
                println!("  mov {}, [{}]", dest_reg, addr_reg);
            }
            TAC::Store { addr, src } => {
                let addr_reg = self.vreg_to_string(addr, vreg_to_reg);
                let src_reg = self.vreg_to_string(src, vreg_to_reg);
                if let Some(offset) = self.vreg_to_offset.get(addr) {
                    println!("  mov {}, [rbp - {}]", addr_reg, offset);
                }
                println!("  mov [{}], {}", addr_reg, src_reg);
            }
            TAC::Return { src } => {
                let src_reg = self.vreg_to_string(src, vreg_to_reg);
                // 変数のときはレジスタに最新の値をロードする
                if let Some(offset) = self.vreg_to_offset.get(src) {
                    println!("  mov {}, [rbp - {}]", src_reg, offset);
                }
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

                // 現在のレジスタを待避
                let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for r in &regs {
                    println!("  push {}", r);
                }

                // 衝突防止のため一時レジスタに代入する
                let save = ["rbx", "r12", "r13", "r14", "r15"];
                for (i, arg) in args_reg.iter().enumerate() {
                    if let Some(s) = save.get(i) {
                        println!("  mov {}, {}", s, arg);
                    } else {
                        println!("  push {}", arg);
                    }
                }
                
                // 一時レジスタを介して引数に渡す
                for i in 0..args.len() {
                    let dest = regs.get(i).expect("too many args are given");
                    if let Some(s) = save.get(i) {
                        println!("  mov {}, {}", dest, s);
                    } else {
                        println!("  pop {}", dest);
                    }
                }

                println!("  call {}", fn_name);
                
                // レジスタを復活させる
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
                let recv_regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for i in 0..params.len() {
                    println!("  push {}", recv_regs.get(i).expect("too many args"));
                }
                for param in params.iter().rev() {
                    let dest = self.vreg_to_string(&param.dest, vreg_to_reg);
                    let offset = self.get_offset(&param.dest);
                    println!("  pop {}", dest);
                    println!("  mov [rbp - {}], {}", offset, dest);
                }
            }
            // ワイルドカードを使わない
        }
    }
}
