use nonicc::{
    lexer::{TokenStream, Tokenizer}, 
    parser::Parser,
    types::{Stmt::*, Type::*},
};


#[test]
fn parse_int_decl() {
    let input = "int main() { int a; }";
    let mut tokenizer = Tokenizer::new(input);
    let tok_vec = tokenizer.tokenize();
    let token_stream = TokenStream::new(tok_vec, input);
    let mut parser = Parser::new(token_stream);
    let mut asts = Vec::new();
    while !parser.tokens.is_eof() {
        asts.push(parser.defun());
    }
    eprintln!("[DEBUG] asts: {:?}", asts);
    let expected = vec![
        Fn { 
            fn_name: String::from("main"), 
            params: vec![], 
            body: vec![VarDecl { name: String::from("a"), ty: Int }] 
        }
    ];
    assert_eq!(asts, expected);
}