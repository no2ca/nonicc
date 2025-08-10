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

#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    Lelse(usize),
    Lend(usize),
}

#[derive(Debug, PartialEq)]
pub enum ThreeAddressCode {
    LoadImm { dest: VirtualReg, value: i32 },
    BinOpCode { dest: VirtualReg, left: Operand, op: BinOp, right: Operand },
    Assign { dest: VirtualReg, src: Operand },
    LoadVar { dest: VirtualReg, var: String }, // TODO: これ必要なの
    Return { src: VirtualReg },
    IfFalse { cond: VirtualReg, label: Label }, // condが0ならlabelに飛ぶ
    GoTo { label: Label },
    Label { label: Label },
    Call { fn_name: String, ret_val: VirtualReg },
    Fn { fn_name: String },
}

impl ThreeAddressCode {
    /// 命令が使用しているレジスタを列挙して配列を返す
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
            ThreeAddressCode::Assign { dest, src } => {
                let mut vregs = vec![dest.clone()];
                match src {
                    Operand::Reg(vreg) => vregs.push(vreg.clone()),
                    _ => ()
                }
                vregs
            }
            ThreeAddressCode::LoadVar { dest, .. } => {
                vec![dest.clone()]
            }
            ThreeAddressCode::Return { src } => {
                vec![src.clone()]
            }
            ThreeAddressCode::IfFalse { cond, .. } => {
                vec![cond.clone()]
            }
            ThreeAddressCode::GoTo { .. } => {
                Vec::new()
            }
            ThreeAddressCode::Label { .. } => {
                Vec::new()
            }
            ThreeAddressCode::Call { ret_val , .. } => {
                vec![ret_val.clone()]
            }
            ThreeAddressCode::Fn { .. } => {
                Vec::new()
            }
            // 忘れてバグの原因になるためワイルドカードを使わない
        }
    }
}