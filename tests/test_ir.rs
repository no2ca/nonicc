use no2cc::gen_ir::{ GenIrContext, node_to_ir };
use no2cc::types_ir::{ VirtualReg, Operand::*, BinOp::*, ThreeAddressCode::* };
use no2cc::parser::Parser;
use no2cc::lexer::{ Tokenizer, TokenStream };

// 足し算のテスト
#[test]
fn ir_add() {
    let input = " 1 + 1; ";
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();
    let tokens = TokenStream::new(tok_vec, input);

    let mut parser = Parser::new(tokens);
    let node = parser.stmt();
    let mut context = GenIrContext::new();
    node_to_ir(&node, &mut context);

    let output_ir = context.code;
    let expected = vec![
        LoadImm { dest: VirtualReg { id: 0 }, value: 1 },
        LoadImm { dest: VirtualReg { id: 1 }, value: 1 },
        BinOpCode { dest: VirtualReg { id: 2 }, left: Reg(VirtualReg { id: 0 }), op: Add, right: Reg(VirtualReg { id: 1 }) }
    ];

    assert_eq!(output_ir, expected);
}