use crate::ir::types_ir::{VirtualReg, ThreeAddressCode as TAC};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Interval {
    pub vreg: VirtualReg,
    pub start: usize,
    pub end: usize,
    pub reg: Option<usize>,
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
                         left: VirtualReg { id: 0 }, 
                         op: crate::ir::types_ir::BinOp::Add, 
                         right: VirtualReg { id: 1 },
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