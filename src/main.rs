#![allow(non_camel_case_types)]

use clap::Parser as ClapParser;
use anyhow::anyhow;

use nonicc::error_at;
use nonicc::frame::Frame;
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

    // パース
    let mut parser = Parser::new(TokenStream::new(tok_vec, &input));
    let mut nodes = Vec::new();
    let mut last_idx = parser.tokens.idx;
    while !parser.tokens.is_eof()  {

        nodes.push(parser.defun());

        if parser.tokens.idx == last_idx {
            // トークンが進まないときはエラーを出す
            // 無限ループを避けるため
            let e = anyhow!("Parser stuck: token index not advancing");
            error_at(&input, parser.tokens.idx, e)
        }
        last_idx = parser.tokens.idx;
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

    // caller-saved (呼び出し側が保存するレジスタ) だけを使用
    let regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    let regs_count = regs.len();

    // 各関数について中間表現を生成してレジスタ割り当て
    for node in &nodes {
        let mut context = GenIrContext::new();
        stmt_to_ir(node, &mut context);
        let code = context.get_ir_code();
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
        let vreg_to_reg = register_allocation::linear_reg_alloc(&mut intervals, regs_count);
        
        // スタックフレームの計算
        let frame = Frame::from_lvar_map(lvar_map);

        // コード生成
        let generator = gen_x64::Generator::new(regs.clone(), code, frame);
        generator.gen_fn(vreg_to_reg.clone());

        if args.debug {
            eprintln!("[DEBUG] vreg_to_offset: {:?}", generator.frame.vreg_to_offset);
            eprintln!("[DEBUG] vreg_to_reg");
            eprintln!("{:?}", vreg_to_reg);
        }
    }
}
