use std::collections::{HashMap, HashSet};

use crate::types_ir::{ThreeAddressCode as TAC, VirtualReg, BinOp, Operand};

#[derive(Debug, PartialEq)]
pub struct Interval {
    vreg: VirtualReg,
    start: usize,
    end: usize,
    reg: Option<usize>,
}

impl Interval {
    fn new(virtual_reg: VirtualReg, start: usize, end: usize) -> Interval {
        Interval { vreg: virtual_reg, start: start, end, reg: None }
    }
}

/// 生存区間を記録する
/// - 先頭から見て初めて出現した位置がstart
/// - 後方から見て初めて出現した位置がend
pub fn scan_interval(codes: &Vec<TAC>) -> Vec<Interval> {
    let mut start = HashMap::new();
    let mut end = HashMap::new();

    // 先頭から見てレジスタ名と位置を記録
    for (i, tac) in codes.iter().enumerate() {
        // 命令から使用しているレジスタを取得
        for reg in tac.get_using_regs() {
            start.entry(reg).or_insert(i);
        }
    }

    for (i, tac) in codes.iter().enumerate().rev() {
        // 命令から使用しているレジスタを取得
        for reg in tac.get_using_regs() {
            end.entry(reg).or_insert(i);
        }
    }
    
    // 両方のHashMapを使ってインターバルを集める
    let mut intervals: Vec<Interval> = vec![];
    for (vreg, start_idx) in start {
        let end_idx = end.get(&vreg).unwrap().clone();
        let interval = Interval::new(vreg, start_idx, end_idx);
        intervals.push(interval);
    }
    intervals

}

/// 線形スキャンレジスタ割り当て
/// - 開始時刻でソート
/// - その時点でアクティブなレジスタを記録しておく
pub fn linear_reg_alloc(intervals: &mut Vec<Interval>) -> HashMap<VirtualReg, usize> {
    // 開始時間でソート
    intervals.sort_by_key(|i| i.start);
    
    let mut active: Vec<&mut Interval> = Vec::new();
    let mut vreg_to_reg = HashMap::new();
    
    for interval in intervals {
        // 生存しているものを残す
        active.retain(|a| a.end > interval.start);
        
        if active.len() <= 7 {
            // レジスタに空きがある場合
            // ここでpanicするならバグ
            let reg_idx = find_free_register(&active).unwrap();
            
            // TODO: ここで2つとも更新しないといけないのは良くない
            // HashMapの戻り値用
            vreg_to_reg.entry(interval.vreg).or_insert(reg_idx);
            // 計算用
            interval.reg = Some(reg_idx);
        } else {
            unimplemented!("no handle for spill");
        }
        active.push(interval);
        eprintln!("[DEBUG] active: {:?}", active);
    }
    
    vreg_to_reg
}

/// active配列を受け取って空いているレジスタを調べる
/// - activeで見るのは使用しているregisterのみ
/// - 使用していない最初のregisterの番号を返す
fn find_free_register(active: &Vec<&mut Interval>) -> Option<usize> {
    let mut used = HashSet::new();
    for interval in active {
        if let Some(reg) = interval.reg {
            used.insert(reg);
        }
    }
    for i in 0..7 {
        if !used.contains(&i) {
            return Some(i);
        }
    }
    None
}

pub struct Generator<'a> {
    regs: Vec<&'a str>,
    codes: Vec<TAC>,
}

impl<'a> Generator<'a> {
    pub fn new(codes: Vec<TAC>) -> Generator<'a> {
        Generator {
            regs: vec!["rbx", "rdi", "r10", "r11", "r12", "r13", "r14", "r15"],
            codes,
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
    
    pub fn gen_all(&self, vreg_to_reg: &HashMap<VirtualReg, usize>) {
        for instr in &self.codes {
            self.generate(&vreg_to_reg, instr);
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


#[test]
fn test_scan_imm_interval() {
    let ir = vec![
        TAC::LoadImm { dest: VirtualReg { id: 0 }, value: 1 },
        TAC::LoadImm { dest: VirtualReg { id: 1 }, value: 2 },
    ];

    let mut intervals = scan_interval(&ir);
    intervals.sort_by_key(|i| i.start);

    let expected = vec![
        Interval { vreg: VirtualReg { id: 0 }, start: 0, end: 0, reg: None }, 
        Interval { vreg: VirtualReg { id: 1 }, start: 1, end: 1, reg: None }
    ];
    assert_eq!(intervals, expected);
}

#[test]
fn test_scan_adding_interval() {
    let ir = vec![
        TAC::LoadImm { dest: VirtualReg { id: 0 }, value: 1 },
        TAC::LoadImm { dest: VirtualReg { id: 1 }, value: 1 },
        TAC::BinOpCode { dest: VirtualReg { id: 2 }, 
                         left: crate::types_ir::Operand::Reg(VirtualReg { id: 0 }), 
                         op: crate::types_ir::BinOp::Add, 
                         right: crate::types_ir::Operand::Reg(VirtualReg { id: 1 })
                       }
    ];
    
    let mut intervals = scan_interval(&ir);
    intervals.sort_by_key(|i| i.start);
    
    let expected = vec![
        Interval { vreg: VirtualReg { id: 0 }, start: 0, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 1 }, start: 1, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 2 }, start: 2, end: 2, reg: None }
    ];
    
    assert_eq!(intervals, expected);
}

#[test]
fn test_alloc_adding() {
    let mut intervals = vec![
        Interval { vreg: VirtualReg { id: 0 }, start: 0, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 1 }, start: 1, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 2 }, start: 2, end: 2, reg: None }
    ];

    let mut result: Vec<(VirtualReg, usize)> = linear_reg_alloc(&mut intervals).into_iter().collect();
    result.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    
    let expected = vec![
        (VirtualReg { id: 0 }, 0),
        (VirtualReg { id: 1 }, 1),
        (VirtualReg { id: 2 }, 0),
    ];
    
    assert_eq!(result, expected);
}