use std::process::exit;

use crate::types::{ Node, NodeKind };

/// 代入先のアドレスをスタックに積んで戻る
fn generate_lval(node: &Node) {
    if node.kind != NodeKind::ND_LVAR {
        eprintln!("代入の左辺値が変数ではありません");
        exit(1);
    }
    
    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.expect("node.offsetがNoneです"));
    println!("  push rax");
    return;
}

pub fn generate(node: &Node) {

    match node.kind { 
        NodeKind::ND_NUM => {
            match node.val {
                Some(val) => println!("  push {}", &val),
                None => panic!("gen() error: missing node.val — received None instead"),
            }
            return;
        }

        // ここは右辺値として変数を扱う時
        // つまり変数の評価をするときに呼び出される
        NodeKind::ND_LVAR => {
            generate_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        
        NodeKind::ND_ASSIGN => {
            match &node.lhs {
                Some(lhs) => generate_lval(lhs),
                None => panic!("gen() error: missing node.lhs — received None instead"),
            }

            match &node.rhs {
                Some(rhs) => generate(rhs),
                None => panic!("gen() error: missing node.rhs — received None instead"),
            }
            
            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
        }

        _ => ()

    }

    // 数以外は両側に何か持っているはず
    match &node.lhs {
        Some(lhs) => generate(lhs),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    match &node.rhs {
        Some(rhs) => generate(rhs),
        None => panic!("gen() error: missing node.rhs — received None instead"),
    }

    println!("  pop rdi"); // 左側の項の値
    println!("  pop rax"); // 右側の項の値

    match node.kind {
        NodeKind::ND_NUM => (),
        NodeKind::ND_ADD => {
            println!("  add rax, rdi");
        }
        NodeKind::ND_SUB => {
            println!("  sub rax, rdi");
        }
        NodeKind::ND_MUL => {
            println!("  imul rax, rdi");
        }
        NodeKind::ND_DIV => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::ND_LE => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_LT => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_EQ => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::ND_NE => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        _ => {
            eprintln!("そんなトークン知らねぇ！！\nアセンブリ出力部分が未実装！！");
        }
    }

    println!("  push rax");
}