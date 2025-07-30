#![allow(non_camel_case_types)]

use anyhow::anyhow;
use std::env;
use std::process::exit;

use no2cc::error_at;
use no2cc::lexer::{ Tokenizer, TokenStream };
use no2cc::parser::{ Parser };
use no2cc::codegen::{generate, CodegenContext};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: 引数の数が間違っています");
        exit(1);
    }

    let input = &args[1];
    let mut input_stream = Tokenizer::new(input);
    let tok_vec = input_stream.tokenize();

    eprintln!("[DEBUG] tokens: \n{:?}", &tok_vec);

    let mut tok = Parser::new(TokenStream::new(tok_vec, input));

    // TODO: 無限ループの可能性がある
    let mut nodes:Vec<Box<no2cc::types::Node>> = vec![];
    while tok.tokens.idx != tok.tokens.tok_vec.len() - 1  {
        nodes.push(tok.stmt());
    }

    eprintln!("[DEBUG] tokens.len: {}", tok.tokens.tok_vec.len());
    eprintln!("[DEBUG] idx: {}", tok.tokens.idx);
    
    // トークンを最後までパース出来たか調べる
    // EOFトークンがあるので -1 している
    if tok.tokens.idx != tok.tokens.tok_vec.len() - 1 {
        error_at(tok.tokens.input, tok.tokens.get_current_token().pos, anyhow!("余分なトークンがあります"));
    }

    eprintln!("[DEBUG] node: \n{:?}", nodes.clone());

    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, 208");

    let mut codegen_context = CodegenContext::new();
    for node in nodes {
        generate(&node, &mut codegen_context);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
