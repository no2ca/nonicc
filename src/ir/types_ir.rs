#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualReg{
    pub id: usize,
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Reg(VirtualReg),    // 仮想レジスタ名
    Imm(i32),           // 即値 (TODO: この桁数も要検討)
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
}


#[derive(Debug, PartialEq)]
pub enum ThreeAddressCode {
    Assign { dest: VirtualReg, src: Operand },
    LoadImm { dest: VirtualReg, value: i32 },
    BinOpCode { dest: VirtualReg, left: Operand, op: BinOp, right: Operand }
}

pub type IR = Vec<ThreeAddressCode>;