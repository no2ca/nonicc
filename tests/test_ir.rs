use no2cc::ir::gen_ir::{stmt_to_ir, GenIrContext};
use no2cc::ir::types_ir::{ VirtualReg, Operand::*, BinOp::*, ThreeAddressCode::* };
use no2cc::parser::Parser;
use no2cc::lexer::{ Tokenizer, TokenStream };

// 足し算のテスト
#[test]
fn ir_add() {
    let input = " main() {1 + 1;} ";
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();
    let tokens = TokenStream::new(tok_vec, input);

    let mut parser = Parser::new(tokens);
    let node = parser.defun();
    let mut context = GenIrContext::new();
    stmt_to_ir(&node, &mut context);

    let output_ir = context.code;
    let expected = vec![
        LoadImm { dest: VirtualReg { id: 0 }, value: 1 },
        LoadImm { dest: VirtualReg { id: 1 }, value: 1 },
        BinOpCode { dest: VirtualReg { id: 2 }, left: Reg(VirtualReg { id: 0 }), op: Add, right: Reg(VirtualReg { id: 1 }) }
    ];

    assert_eq!(output_ir, expected);
}

// 四則演算のテスト
#[test]
fn ir_basic_op() {
    let input = " main() {1 + 2 - 3 * 4 / 5;} ";
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();
    let tokens = TokenStream::new(tok_vec, input);

    let mut parser = Parser::new(tokens);
    let node = parser.defun();
    let mut context = GenIrContext::new();
    stmt_to_ir(&node, &mut context);

    let output_ir = context.code;
    let expected = vec![
        LoadImm { dest: VirtualReg { id: 0 }, value: 1 },
        LoadImm { dest: VirtualReg { id: 1 }, value: 2 },
        BinOpCode { dest: VirtualReg { id: 2 }, left: Reg(VirtualReg { id: 0 }), op: Add, right: Reg(VirtualReg { id: 1 }) },
        LoadImm { dest: VirtualReg { id: 3 }, value: 3 },
        LoadImm { dest: VirtualReg { id: 4 }, value: 4 },
        BinOpCode { dest: VirtualReg { id: 5 }, left: Reg(VirtualReg { id: 3 }), op: Mul, right: Reg(VirtualReg { id: 4 }) },
        LoadImm { dest: VirtualReg { id: 6 }, value: 5 },
        BinOpCode { dest: VirtualReg { id: 7 }, left: Reg(VirtualReg { id: 5 }), op: Div, right: Reg(VirtualReg { id: 6 }) },
        BinOpCode { dest: VirtualReg { id: 8 }, left: Reg(VirtualReg { id: 2 }), op: Sub, right: Reg(VirtualReg { id: 7 }) },
    ];

    assert_eq!(output_ir, expected);
}