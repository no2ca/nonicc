#![allow(non_camel_case_types)]

use clap::Parser as ClapParser;

use no2cc::lexer::{ Tokenizer, TokenStream };
use no2cc::parser::{ Parser };

#[derive(ClapParser, Debug)]
struct Args {
    #[arg(index = 1)]
    input: String,

    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() {
    // 引数を解析する
    let args = Args::parse();
    let input = &args.input;
    
    // トークナイズ
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();

    if args.debug {
        eprintln!("[DEBUG] tokens: \n{:?}", &tok_vec);
    }

    // 全ての文をパースする
    // TODO: 無限ループの可能性がある
    let mut parser = Parser::new(TokenStream::new(tok_vec, input));
    let mut nodes:Vec<Box<no2cc::types::Node>> = vec![];
    while parser.tokens.idx != parser.tokens.tok_vec.len() - 1  {
        nodes.push(parser.stmt());
    }

    if args.debug {
        eprintln!("[DEBUG] node: \n{:?}", nodes.clone());
    }

    // スタックサイズの計算
    // 16ビットアラインメント
    // TODO: 変数サイズは常に8バイトとは限らなくなる
    let stack_size = (((parser.lvars.lvars_vec.len() - 1) * 8 + 15) / 16) * 16;

    // 中間表現の生成
    use no2cc::ir::gen_ir::{ GenIrContext, stmt_to_ir };
    use no2cc::reg_alloc::{interval_analysis, register_allocation};
    use no2cc::gen_x64;
    let mut codes = vec![];
    let mut context = GenIrContext::new();

    for node in &nodes {
        stmt_to_ir(node, &mut context);
        codes.append(&mut context.code);
    }
    
    if args.debug {
        eprintln!("[DEBUG] IR:");
        for code in &codes {
            println!("{:?}", code);
        }
    }

    // レジスタ割り当て
    // caller-saved (呼び出し側が保存するレジスタ) だけを使用
    let regs = vec!["rdi", "rsi", "rcx", "r8", "r9", "r10", "r11"];
    let reg_count = regs.len();

    let mut intervals = interval_analysis::scan_interval(&codes);
    
    let vreg_to_reg = register_allocation::linear_reg_alloc(&mut intervals, reg_count);
    
    if args.debug {
        eprintln!("[DEBUG] intervals");
        eprintln!("{:?}", intervals);
        eprintln!("[DEBUG] vreg_to_reg");
        eprintln!("{:?}", vreg_to_reg);
    }

    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    
    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", stack_size);

    let generator = gen_x64::Generator::new(regs, codes);
    generator.gen_all(&vreg_to_reg);

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
