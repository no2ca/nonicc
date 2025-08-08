use std::collections::{HashMap, HashSet};
use crate::ir::types_ir::VirtualReg;
use crate::reg_alloc::interval_analysis::Interval;

/// 線形スキャンレジスタ割り当て
/// - 開始時刻でソート
/// - その時点でアクティブなレジスタを記録しておく
pub fn linear_reg_alloc(intervals: &mut Vec<Interval>, reg_count: usize) -> HashMap<VirtualReg, usize> {
    // 開始時間でソート
    intervals.sort_by_key(|i| i.start);
    
    let mut active: Vec<&mut Interval> = Vec::new();
    let mut vreg_to_reg = HashMap::new();
    
    for interval in intervals {
        // 生存しているものを残す
        active.retain(|a| a.end > interval.start);
        
        if active.len() < reg_count {
            // レジスタに空きがある場合
            // ここでpanicするならバグ
            let reg_idx = find_free_register(&active, reg_count).unwrap();
            
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
fn find_free_register(active: &Vec<&mut Interval>, reg_count: usize) -> Option<usize> {
    let mut used = HashSet::new();
    for interval in active {
        if let Some(reg) = interval.reg {
            used.insert(reg);
        }
    }
    for i in 0..reg_count {
        if !used.contains(&i) {
            return Some(i);
        }
    }
    None
}

#[test]
/// 例: 1 + 1;
fn test_alloc_binop() {
    let mut intervals = vec![
        Interval { vreg: VirtualReg { id: 0 }, start: 0, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 1 }, start: 1, end: 2, reg: None }, 
        Interval { vreg: VirtualReg { id: 2 }, start: 2, end: 2, reg: None }
    ];

    let mut result: Vec<(VirtualReg, usize)> = linear_reg_alloc(&mut intervals, 8).into_iter().collect();
    result.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    
    let expected = vec![
        (VirtualReg { id: 0 }, 0),
        (VirtualReg { id: 1 }, 1),
        (VirtualReg { id: 2 }, 0),
    ];
    
    assert_eq!(result, expected);
}

#[test]
/// 例: 1 + (2 + 3);
fn test_alloc_longer_op() {
    let mut intervals = vec![
        Interval { vreg: VirtualReg { id: 0 }, start: 0, end: 4, reg: None }, 
        Interval { vreg: VirtualReg { id: 1 }, start: 1, end: 3, reg: None }, 
        Interval { vreg: VirtualReg { id: 2 }, start: 2, end: 3, reg: None }, 
        Interval { vreg: VirtualReg { id: 3 }, start: 3, end: 4, reg: None },
        Interval { vreg: VirtualReg { id: 4 }, start: 4, end: 4, reg: None }, 
    ];

    let mut result: Vec<(VirtualReg, usize)> = linear_reg_alloc(&mut intervals, 8).into_iter().collect();
    result.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    
    let expected = vec![
        (VirtualReg { id: 0 }, 0), 
        (VirtualReg { id: 1 }, 1),
        (VirtualReg { id: 2 }, 2), 
        (VirtualReg { id: 3 }, 1),
        (VirtualReg { id: 4 }, 0), 
    ];
    
    assert_eq!(result, expected);
}