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
    Lbegin(usize),
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
    EvalVar { var: VirtualReg, name: String }, // 生存期間の扱いを分かりやすく扱うために必要
    AddrOf { addr: VirtualReg, var: VirtualReg }, // 変数のアドレスを取る (&a)
    LoadVar { dest: VirtualReg, addr: VirtualReg }, // 参照外し (*p)
    Store { addr: VirtualReg, src: VirtualReg }, // 間接ストア (*p = v)
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
                vec![*dest]
            }
            ThreeAddressCode::BinOpCode { dest, left, right ,.. } => {
                vec![*dest, *left, *right]
            }
            ThreeAddressCode::Assign { dest, src } => {
                vec![*dest, *src]
            }
            ThreeAddressCode::EvalVar { var: dest, .. } => {
                vec![*dest]
            }
            ThreeAddressCode::AddrOf { addr, var } => {
                vec![*addr, *var]
            }
            ThreeAddressCode::LoadVar { dest, addr: src } => {
                vec![*dest, *src]
            }
            ThreeAddressCode::Store { addr, src } => {
                vec![*addr, *src]
            }
            ThreeAddressCode::Return { src } => {
                vec![*src]
            }
            ThreeAddressCode::IfFalse { cond, .. } => {
                vec![*cond]
            }
            ThreeAddressCode::GoTo { .. } => {
                Vec::new()
            }
            ThreeAddressCode::Label { .. } => {
                Vec::new()
            }
            ThreeAddressCode::Call { ret_reg: ret_val , args, .. } => {
                let mut vregs = vec![*ret_val];
                for r in args {
                    vregs.push(*r);
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