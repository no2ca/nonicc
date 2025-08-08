#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtualReg{
    pub id: usize,
}

impl VirtualReg {
    pub(super) fn new(id: usize) -> VirtualReg {
        VirtualReg {
            id,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Reg(VirtualReg),    // 仮想レジスタ名
    Imm(i32),           // 即値 (TODO: この桁数も要検討)
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Le, Lt, Eq, Ne,
}

#[derive(Debug, PartialEq)]
pub enum ThreeAddressCode {
    // Assign { dest: VirtualReg, src: Operand },
    LoadImm { dest: VirtualReg, value: i32 },
    BinOpCode { dest: VirtualReg, left: Operand, op: BinOp, right: Operand }
}

impl ThreeAddressCode {
    /// 命令が使用しているレジスタを列挙する
    pub(crate) fn get_using_regs(&self) -> Vec<VirtualReg> {
        match self {
            ThreeAddressCode::LoadImm { dest, .. } => {
                vec![dest.clone()]
            }
            ThreeAddressCode::BinOpCode { dest, left, right ,.. } => {
                let mut vregs = vec![dest.clone()];
                match left {
                    Operand::Reg(vreg) => vregs.push(vreg.clone()),
                    _ => ()
                }
                match right {
                    Operand::Reg(vreg) => vregs.push(vreg.clone()),
                    _ => ()
                }
                vregs
            }
            // _ => unimplemented!("{:?}", self)
        }
    }
}