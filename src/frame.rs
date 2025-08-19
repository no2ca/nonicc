use std::collections::HashMap;

use crate::ir::types_ir::VirtualReg;

pub struct Frame {
    pub vreg_to_offset: HashMap<VirtualReg, usize>,
}

impl Frame {
    pub fn from_lvar_map(lvar_map: HashMap<String, VirtualReg>) -> Self {
        let mut vec = Vec::new();
        for x in lvar_map {
            vec.push(x);
        }

        // 昇順にする
        vec.sort_by_key(|(_, vreg)| vreg.id);
        
        // オフセットを計算
        let mut vreg_to_offset = HashMap::new();
        let mut offset = 8;
        for (_, vreg) in vec {
            vreg_to_offset.entry(vreg).or_insert(offset);
            offset += 8;
        }
        
        Frame { vreg_to_offset }
    }
}