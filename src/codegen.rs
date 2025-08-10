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
    
    fn get_label_count(&mut self) -> usize {
        let label_count = self.label_count;
        self.label_count += 1;
        label_count
    }
}

/// 代入先のアドレスをスタックに積んで戻る
fn gen_lvar(node: &Node) {
    if node.kind != NodeKind::ND_LVAR {
        eprintln!("代入の左辺値が変数ではありません");
        exit(1);
    }
    
    println!("  mov rax, rbp");
    println!("  sub rax, {}", node.offset.expect("node.offsetがNoneです"));
    println!("  push rax");
}

fn gen_num(node: &Node) {
    match node.val {
        Some(val) => println!("  push {}", &val),
        None => panic!("gen() error: missing node.val — received None instead"),
    }
}

fn gen_return(node: &Node, context: &mut CodegenContext) {
    match &node.lhs {
        Some(lhs) => generate(&lhs, context),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    let lhs = node.lhs.as_ref().unwrap();
    if lhs.kind == NodeKind::ND_BLOCK || lhs.kind == NodeKind::ND_RETURN || 
    lhs.kind == NodeKind::ND_IF || lhs.kind == NodeKind::ND_CALL  {
    } else {
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");              // このあとmainに戻って余分に出力されるが実行されないので気にしない
}

fn gen_assign(node: &Node, context: &mut CodegenContext) {
    match &node.lhs {
        Some(lhs) => gen_lvar(lhs),
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
}

fn gen_if(node: &Node, context: &mut CodegenContext) {
    let label_count = context.get_label_count();

    // conditionの値を生成
    // スタックトップに積んで戻る
    match &node.cond {
        Some(cond) => generate(cond, context),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }
    
    println!("  pop rax");
    println!("  cmp rax, 0");

    // thenのノードを取得
    let then_stmt = match &node.then {
        Some(then) => then,
        None => panic!("gen() error: missing node.rhs — received None instead"),
    };

    // elseがある場合
    if let Some(els_stmt) = &node.els {
        println!("  je .Lelse{}", label_count);
        
        // thenの内容を生成
        if then_stmt.kind == NodeKind::ND_BLOCK || then_stmt.kind == NodeKind::ND_RETURN || 
        then_stmt.kind == NodeKind::ND_IF || then_stmt.kind == NodeKind::ND_CALL  {
            generate(&then_stmt, context);
        } else {
            generate(&then_stmt, context);
            println!("  pop rax");
        }

        println!("  jmp .Lend{}", label_count);
        println!(".Lelse{}: ", label_count);

        // elseの内容を生成
        if els_stmt.kind == NodeKind::ND_BLOCK || els_stmt.kind == NodeKind::ND_RETURN || 
        els_stmt.kind == NodeKind::ND_IF || els_stmt.kind == NodeKind::ND_CALL {
            generate(&els_stmt, context);
        } else {
            generate(&els_stmt, context);
            println!("  pop rax");
        }
        
        println!(".Lend{}: ", label_count);

    } else {
        // if単体の場合
        println!("  je .Lend{}", label_count);

        // thenの内容を生成
        if then_stmt.kind == NodeKind::ND_BLOCK || then_stmt.kind == NodeKind::ND_RETURN || 
        then_stmt.kind == NodeKind::ND_IF || then_stmt.kind == NodeKind::ND_CALL {
            generate(&then_stmt, context);
        } else {
            generate(&then_stmt, context);
            println!("  pop rax");
        }

        println!(".Lend{}: ", {label_count});
    }
}

fn gen_block(node: &Node, context: &mut CodegenContext) {
    let block_stmt = match &node.stmts {
        Some(block_stmt) => block_stmt,
        None => panic!("gen() error: missing node.block_stmt — received None instead"),
    };
    
    for stmt in block_stmt {
        // 毎回popする必要はない
        // バグの原因になった
        // if-else/return/block-stmtは値を持たないため
        if stmt.kind == NodeKind::ND_BLOCK || stmt.kind == NodeKind::ND_RETURN || 
        stmt.kind == NodeKind::ND_IF || stmt.kind == NodeKind::ND_CALL {
            generate(&stmt, context);
        } else {
            generate(&stmt, context);
            println!("  pop rax");
        }
    }
    
}

fn gen_fn(node: &Node) {
    println!("  call {}", node.fn_name.as_ref().unwrap());
}

pub fn generate(node: &Node, context: &mut CodegenContext) {

    match node.kind { 
        NodeKind::ND_NUM => {
            gen_num(node);
            return;
        }

        // 右辺値として変数を扱う時
        NodeKind::ND_LVAR => {
            gen_lvar(node);
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        
        // return文
        NodeKind::ND_RETURN => {
            gen_return(node, context);
            return;
        }

        // 代入文
        // generate_lvalを呼び出す
        NodeKind::ND_ASSIGN => {
            gen_assign(node, context);
            return;
        }

        // if文
        NodeKind::ND_IF => {
            gen_if(node, context);
            return;
        }
        
        // ブロック
        NodeKind::ND_BLOCK => {
            gen_block(node, context);
            return;
        }
        
        NodeKind::ND_CALL => {
            gen_fn(node);
            return;
        }
        
        _ => ()
    }
    
    // 以下は二項演算子の処理
    match &node.lhs {
        Some(lhs) => generate(lhs, context),
        None => panic!("gen() error: missing node.lhs — received None instead"),
    }

    match &node.rhs {
        Some(rhs) => generate(rhs, context),
        None => panic!("gen() error: missing node.rhs — received None instead"),
    }

    println!("  pop rdi"); // 右側の項の値
    println!("  pop rax"); // 左側の項の値
    
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