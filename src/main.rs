#![allow(non_camel_case_types)]

use anyhow::anyhow;
use std::env;
use std::process::exit;

use rs9cc::error_at;
use rs9cc::lexer::{ Tokenizer, TokenStream };
use rs9cc::parser::{ Parser };
use rs9cc::codegen::generate;

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
    let mut nodes:Vec<Box<rs9cc::types::Node>> = vec![];
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

    for node in nodes {
        generate(&node);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
