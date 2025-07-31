#![allow(non_camel_case_types)]

use anyhow::anyhow;
use clap::Parser as ClapParser;

use no2cc::error_at;
use no2cc::lexer::{ Tokenizer, TokenStream };
use no2cc::parser::{ Parser };
use no2cc::codegen::{generate, CodegenContext};
use no2cc::types::NodeKind;


#[derive(ClapParser, Debug)]
struct Args {
    #[arg(index = 1)]
    input: String,

    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    let input = &args.input;

    let mut input_stream = Tokenizer::new(input);
    let tok_vec = input_stream.tokenize();

    if args.debug {
        eprintln!("[DEBUG] tokens: \n{:?}", &tok_vec);
    }

    let mut tok = Parser::new(TokenStream::new(tok_vec, input));

    // TODO: 無限ループの可能性がある
    let mut nodes:Vec<Box<no2cc::types::Node>> = vec![];
    while tok.tokens.idx != tok.tokens.tok_vec.len() - 1  {
        nodes.push(tok.stmt());
    }

    if args.debug {
        eprintln!("[DEBUG] tokens.len: {}", tok.tokens.tok_vec.len());
        eprintln!("[DEBUG] idx: {}", tok.tokens.idx);
    }
    
    // トークンを最後までパース出来たか調べる
    // EOFトークンがあるので -1 している
    if tok.tokens.idx != tok.tokens.tok_vec.len() - 1 {
        error_at(tok.tokens.input, tok.tokens.get_current_token().pos, anyhow!("余分なトークンがあります"));
    }

    if args.debug {
        eprintln!("[DEBUG] node: \n{:?}", nodes.clone());
    }

    // スタックサイズは16ビットアラインメント
    // TODO: 変数サイズは常に8バイトとは限らなくなる
    let stack_size = (((tok.lvars.lvars_vec.len() - 1) * 8 + 15) / 16) * 16;

    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", stack_size);

    let mut codegen_context = CodegenContext::new();
    for node in nodes {
        if node.kind == NodeKind::ND_BLOCK || node.kind == NodeKind::ND_RETURN || node.kind == NodeKind::ND_IF {
            generate(&node, &mut codegen_context);
        } else {
            generate(&node, &mut codegen_context);
            println!("  pop rax");
        }
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
