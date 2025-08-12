use nonicc::ir::gen_ir::{stmt_to_ir, GenIrContext};
use nonicc::ir::types_ir::{ VirtualReg, BinOp::*, ThreeAddressCode::*, Param };
use nonicc::parser::Parser;
use nonicc::lexer::{ Tokenizer, TokenStream };

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
        Fn { fn_name: "main".to_string(), params: Vec::new() }, 
        LoadImm { dest: VirtualReg { id: 0 }, value: 1 }, 
        LoadImm { dest: VirtualReg { id: 1 }, value: 1 }, 
        BinOpCode { dest: VirtualReg { id: 2 }, left: VirtualReg { id: 0 }, op: Add, right: VirtualReg { id: 1 } }
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
        Fn { fn_name: "main".to_string(), params: Vec::new() }, 
        LoadImm { dest: VirtualReg { id: 0 }, value: 1 }, 
        LoadImm { dest: VirtualReg { id: 1 }, value: 2 }, 
        BinOpCode { dest: VirtualReg { id: 2 }, left: VirtualReg { id: 0 }, op: Add, right: VirtualReg { id: 1 } }, 
        LoadImm { dest: VirtualReg { id: 3 }, value: 3 }, 
        LoadImm { dest: VirtualReg { id: 4 }, value: 4 }, 
        BinOpCode { dest: VirtualReg { id: 5 }, left: VirtualReg { id: 3 }, op: Mul, right: VirtualReg { id: 4 } }, 
        LoadImm { dest: VirtualReg { id: 6 }, value: 5 }, 
        BinOpCode { dest: VirtualReg { id: 7 }, left: VirtualReg { id: 5 }, op: Div, right: VirtualReg { id: 6 } }, 
        BinOpCode { dest: VirtualReg { id: 8 }, left: VirtualReg { id: 2 }, op: Sub, right: VirtualReg { id: 7 } }
    ];

    assert_eq!(output_ir, expected);
}

// 引数のある関数定義のテスト
#[test]
fn ir_function_with_params() {
    let input = " foo(a, b) { a; b; return 42; } ";
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();
    let tokens = TokenStream::new(tok_vec, input);

    let mut parser = Parser::new(tokens);
    let node = parser.defun();
    let mut context = GenIrContext::new();
    stmt_to_ir(&node, &mut context);

    let output_ir = context.code;
    let expected = vec![
        Fn { fn_name: "foo".to_string(), params: vec![Param { dest: VirtualReg { id: 0 }, name: "a".to_string() }, Param { dest: VirtualReg { id: 1 }, name: "b".to_string() }] },
        EvalVar { dest: VirtualReg { id: 0 }, name: "a".to_string() },
        EvalVar { dest: VirtualReg { id: 1 }, name: "b".to_string() },
        LoadImm { dest: VirtualReg { id: 2 }, value: 42 },
        Return { src: VirtualReg { id: 2 } },
    ];

    assert_eq!(output_ir, expected);
}