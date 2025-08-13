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

#[derive(Debug, PartialEq, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Le, Lt, Eq, Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Label {
    Lelse(usize),
    Lend(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub dest: VirtualReg,
    pub name: String,
}

impl Param {
    pub fn new(dest: VirtualReg, name: String) -> Self {
        Param { dest, name }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ThreeAddressCode {
    LoadImm { dest: VirtualReg, value: i32 },
    BinOpCode { dest: VirtualReg, left: VirtualReg, op: BinOp, right: VirtualReg },
    Assign { dest: VirtualReg, src: VirtualReg },
    EvalVar { dest: VirtualReg, name: String }, // 生存期間の扱いを分かりやすく扱うために必要
    Return { src: VirtualReg },
    IfFalse { cond: VirtualReg, label: Label }, // condが0ならlabelに飛ぶ
    GoTo { label: Label },
    Label { label: Label },
    Call { fn_name: String, args: Vec<VirtualReg>, ret_reg: VirtualReg },
    Fn { fn_name: String, params: Vec<Param> },
}

impl ThreeAddressCode {
    /// 命令が使用しているレジスタを列挙して配列を返す
    pub(crate) fn get_using_regs(&self) -> Vec<VirtualReg> {
        match self {
            ThreeAddressCode::LoadImm { dest, .. } => {
                vec![dest.clone()]
            }
            ThreeAddressCode::BinOpCode { dest, left, right ,.. } => {
                vec![dest.clone(), left.clone(), right.clone()]
            }
            ThreeAddressCode::Assign { dest, src } => {
                vec![dest.clone(), src.clone()]
            }
            ThreeAddressCode::EvalVar { dest, .. } => {
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
            ThreeAddressCode::Call { ret_reg: ret_val , args, .. } => {
                let mut vregs = vec![ret_val.clone()];
                for r in args {
                    vregs.push(r.clone());
                }
                vregs
            }
            ThreeAddressCode::Fn { params, .. } => {
                let mut vregs = Vec::new();
                for param in params {
                    vregs.push(param.dest);
                }
                vregs
            }
            // 忘れてバグの原因になるためワイルドカードを使わない
        }
    }
}