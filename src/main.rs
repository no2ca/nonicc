#![allow(non_camel_case_types)]

use clap::Parser as ClapParser;

use nonicc::lexer::{ Tokenizer, TokenStream };
use nonicc::parser::{ Parser };

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
    let input = args.input;
    
    // トークナイズ
    let mut tokenizer = Tokenizer::new(&input);
    let tok_vec = tokenizer.tokenize();

    if args.debug {
        eprintln!("[DEBUG] tokens: \n{:?}", tok_vec);
    }

    // 全ての文をパースする
    // TODO: 無限ループの可能性がある
    let mut parser = Parser::new(TokenStream::new(tok_vec, &input));
    let mut nodes = Vec::new();
    while parser.tokens.idx != parser.tokens.tok_vec.len() - 1  {
        nodes.push(parser.defun());
    }

    if args.debug {
        eprintln!("[DEBUG] node: \n{:?}", nodes);
    }


    // コード生成ここから
    println!(".intel_syntax noprefix");
    println!(".globl main");

    // 中間表現の生成
    use nonicc::ir::gen_ir::{ GenIrContext, stmt_to_ir };
    use nonicc::reg_alloc::{interval_analysis, register_allocation};
    use nonicc::gen_x64;
    // 各関数について中間表現を生成してレジスタ割り当て
    // caller-saved (呼び出し側が保存するレジスタ) だけを使用
    let regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    let reg_count = regs.len();
    for node in &nodes {
        let mut context = GenIrContext::new();
        stmt_to_ir(node, &mut context);
        let code = context.clone().code;
        let lvar_map = context.get_lvar_map();
        
        // デバッグ
        if args.debug {
            eprintln!("[DEBUG] IR:");
            for c in &code {
                eprintln!("{:?}", c);
            }
        }

        // レジスタ割り当て
        let mut intervals = interval_analysis::scan_interval(&code);
        let vreg_to_reg = register_allocation::linear_reg_alloc(&mut intervals, reg_count);

        // デバッグ
        if args.debug {
            eprintln!("[DEBUG] intervals");
            eprintln!("{:?}", intervals);
        }

        let generator = gen_x64::Generator::new(regs.clone(), code, lvar_map);
        generator.gen_fn(vreg_to_reg.clone());

        if args.debug {
            eprintln!("[DEBUG] vreg_to_offset: {:?}", generator.vreg_to_offset);
            eprintln!("[DEBUG] vreg_to_reg");
            eprintln!("{:?}", vreg_to_reg);
        }
    }

    
}
