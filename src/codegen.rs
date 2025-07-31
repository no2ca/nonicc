use std::process::exit;

use crate::types::{ Node, NodeKind };

pub struct CodegenContext {
    label_count: usize,
}

impl CodegenContext {
    pub fn new() -> CodegenContext {
        CodegenContext {
            label_count: 1,
        }
    }
}

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

pub fn generate(node: &Node, context: &mut CodegenContext) {

    // 両端にノードを持たない場合
    // もしくは片方だけに持っている場合
    match node.kind { 
        NodeKind::ND_NUM => {
            match node.val {
                Some(val) => println!("  push {}", &val),
                None => panic!("gen() error: missing node.val — received None instead"),
            }
            return;
        }

        // 右辺値として変数を扱う時
        NodeKind::ND_LVAR => {
            generate_lval(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        
        
        // return文
        NodeKind::ND_RETURN => {
            match &node.lhs {
                Some(lhs) => generate(&lhs, context),
                None => panic!("gen() error: missing node.lhs — received None instead"),
            }

            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");              // このあとmainに戻って余分に出力されるが実行されないので気にしない
            return;
        }

        _ => ()

    }
    
    // 代入文
    // generate_lvalを呼び出す
    if node.kind == NodeKind::ND_ASSIGN {
        match &node.lhs {
            Some(lhs) => generate_lval(lhs),
            None => panic!("gen() error: missing node.lhs — received None instead"),
        }

        match &node.rhs {
            Some(rhs) => generate(rhs, context),
            None => panic!("gen() error: missing node.rhs — received None instead"),
        }
        
        println!("  pop rdi");          // 計算結果を取り出す
        println!("  pop rax");          // 変数のアドレスを取り出す 
        println!("  mov [rax], rdi");
        println!("  push rdi");
        return;
    }
    
    // if文
    if node.kind == NodeKind::ND_IF {
        // TODO: ここで管理するのが良くなさすぎる
        let label_count = context.label_count;
        context.label_count += 1;

        // conditionの値を生成
        // スタックトップに積んで戻る
        match &node.cond {
            Some(cond) => generate(cond, context),
            None => panic!("gen() error: missing node.lhs — received None instead"),
        }

        // thenのノードを取得
        let then = match &node.then {
            Some(then) => then,
            None => panic!("gen() error: missing node.rhs — received None instead"),
        };
        
        println!("  pop rax");
        println!("  cmp rax, 0");

        // elseがある場合
        if let Some(els) = &node.els {
            println!("  je .Lelse{}", label_count);
            
            // thenの内容を生成
            generate(then, context);

            println!("  jmp .Lend{}", label_count);
            println!(".Lelse{}: ", label_count);

            // elseの内容を生成
            generate(els, context);
            
            println!(".Lend{}: ", label_count);

        } else {
            // if単体の場合
            println!("  je .Lend{}", label_count);

            // thenの内容を生成
            generate(then, context);

            println!(".Lend{}: ", {label_count});
        }
        
        return;
    }
    
    // 上で処理したノード以外は両側に何か持っているはず
    match &node.lhs {
        Some(lhs) => generate(lhs, context),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    match &node.rhs {
        Some(rhs) => generate(rhs, context),
        None => panic!("gen() error: missing node.rhs — received None instead"),
    }

    println!("  pop rdi"); // 左側の項の値
    println!("  pop rax"); // 右側の項の値

    match node.kind {
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
            unimplemented!("{:?}", node.kind);
        }
    }

    println!("  push rax");
}