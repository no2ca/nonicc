#![allow(non_camel_case_types)]

use std::collections::HashMap;

use clap::Parser as ClapParser;

use no2cc::lexer::{ Tokenizer, TokenStream };
use no2cc::parser::{ Parser };
use no2cc::codegen::{ generate, CodegenContext };
use no2cc::types::NodeKind;


#[derive(ClapParser, Debug)]
struct Args {
    #[arg(index = 1)]
    input: String,

    #[arg(short = 'd', long = "debug")]
    debug: bool,
    
    #[arg(short = 'i', long = "ir")]
    ir: bool,
}

fn main() {
    // 引数を解析する
    let args = Args::parse();
    let input = &args.input;
    
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();

    if args.debug {
        eprintln!("[DEBUG] tokens: \n{:?}", &tok_vec);
    }

    let mut parser = Parser::new(TokenStream::new(tok_vec, input));

    // 全ての文をパースする
    // TODO: 無限ループの可能性がある
    let mut nodes:Vec<Box<no2cc::types::Node>> = vec![];
    while parser.tokens.idx != parser.tokens.tok_vec.len() - 1  {
        nodes.push(parser.stmt());
    }

    if args.debug {
        eprintln!("[DEBUG] node: \n{:?}", nodes.clone());
    }

    // 中間表現の生成
    use no2cc::ir::gen_ir::{ GenIrContext, node_to_ir };
    use no2cc::gen_x64;
    let mut codes = vec![];
    let mut vreg_to_reg= HashMap::new();
    if args.ir {
        let mut context = GenIrContext::new();
        eprintln!("[DEBUG] IR:");
        for node in &nodes {
            node_to_ir(node, &mut context);
            for x in &context.code {
                eprintln!("{:?}", x);
            }
            codes.append(&mut context.code);
        }
        let mut intervals = gen_x64::scan_interval(&codes);
        vreg_to_reg = gen_x64::linear_reg_alloc(&mut intervals);
    }

    // スタックサイズは16ビットアラインメント
    // TODO: 変数サイズは常に8バイトとは限らなくなる
    let stack_size = (((parser.lvars.lvars_vec.len() - 1) * 8 + 15) / 16) * 16;

    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", stack_size);

    if args.ir {
        let generator = gen_x64::Generator::new(codes);
        generator.gen_all(&vreg_to_reg);
    } else {
        let mut codegen_context = CodegenContext::new();
        for node in nodes {
            if node.kind == NodeKind::ND_BLOCK || node.kind == NodeKind::ND_RETURN || 
            node.kind == NodeKind::ND_IF || node.kind == NodeKind::ND_FN {
                generate(&node, &mut codegen_context);
            } else {
                generate(&node, &mut codegen_context);
                println!("  pop rax");
            }
        }
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
