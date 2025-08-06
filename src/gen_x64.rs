use crate::types_ir::ThreeAddressCode as TAC;

pub fn generate(inst: TAC) {
    let regs = vec!["rcx", "rdx"];
    match inst {
        TAC::LoadImm { dest, value} => {
            println!("  mov {}, {}", regs[dest.id], value);            
            println!("  mov rax, {}", regs[dest.id]);
        }
        // TAC::BinOpCode { dest, left, op, right } => (),
        // TAC::Assign { dest, src } => (),
        _ => unimplemented!("{:?}", inst),
    }
}