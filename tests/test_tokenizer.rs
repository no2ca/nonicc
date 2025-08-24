use nonicc::types::{ Token, TokenKind::*, TypeKind };
use nonicc::lexer::Tokenizer;

/// トークナイザのテスト
/// EOFは最後の位置+1
#[test]
fn tokenize_single_number() {
    let mut tokinizer = Tokenizer::new("0;");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_NUM, val: Some(0), str: "0".to_string(), len: 1, pos: 0 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 1 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 2 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_number_with_whitespace() {
    let mut tokinizer = Tokenizer::new(" 42; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_NUM, val: Some(42), str: "42".to_string(), len: 2, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 3 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 5 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_string_with_whitespace() {
    let mut tokinizer = Tokenizer::new(" foo; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "foo".to_string(), len: 3, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 4 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 6 }
    ];
    
    assert_eq!(tokens, expected);
} 

 
#[test]
fn tokenize_ambiguous_equal() {
    let mut tokinizer = Tokenizer::new(" == == = ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_RESERVED, val: None, str: "==".to_string(), len: 2, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: "==".to_string(), len: 2, pos: 4 }, 
        Token { kind: TK_RESERVED, val: None, str: "=".to_string(), len: 1, pos: 7 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 9 }
    ];
    
    assert_eq!(tokens, expected);
} 

#[test]
fn tokenize_ambiguous_inequal() {
    let mut tokinizer = Tokenizer::new(" < <= >= > ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_RESERVED, val: None, str: "<".to_string(), len: 1, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: "<=".to_string(), len: 2, pos: 3 }, 
        Token { kind: TK_RESERVED, val: None, str: ">=".to_string(), len: 2, pos: 6 }, 
        Token { kind: TK_RESERVED, val: None, str: ">".to_string(), len: 1, pos: 9 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 11 }
    ];
    
    assert_eq!(tokens, expected);
} 

#[test]
fn tokenize_return() {
    let mut tokinizer = Tokenizer::new(" return ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_RETURN, val: None, str: "return".to_string(), len: 6, pos: 1 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 8 }
    ];
    
    assert_eq!(tokens, expected);
} 

#[test]
fn tokenize_returnx() {
    let mut tokinizer = Tokenizer::new(" returnx ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "returnx".to_string(), len: 7, pos: 1 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 9 }
    ];
    
    assert_eq!(tokens, expected);
} 

#[test]
fn tokenize_if() {
    let mut tokinizer = Tokenizer::new(" if ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IF, val: None, str: "if".to_string(), len: 2, pos: 1 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 4 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_ifx() {
    let mut tokinizer = Tokenizer::new(" ifx ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "ifx".to_string(), len: 3, pos: 1 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 5 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_ident_with_num() {
    let mut tokinizer = Tokenizer::new(" A2b_C3; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "A2b_C3".to_string(), len: 6, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 7 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 9 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_while() {
    let mut tokinizer = Tokenizer::new(" while; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_WHILE, val: None, str: "while".to_string(), len: 5, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 6 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 8 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_whilex() {
    let mut tokinizer = Tokenizer::new(" whilex; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "whilex".to_string(), len: 6, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 7 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 9 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_for() {
    let mut tokinizer = Tokenizer::new(" for; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_FOR, val: None, str: "for".to_string(), len: 3, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 4 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 6 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_forx() {
    let mut tokinizer = Tokenizer::new(" forx; ");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "forx".to_string(), len: 4, pos: 1 }, 
        Token { kind: TK_RESERVED, val: None, str: ";".to_string(), len: 1, pos: 5 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 7 }
    ];
    
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_type_int() {
    let mut tokinizer = Tokenizer::new("int");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_TYPE(TypeKind::Int), val: None, str: String::from("int"), len: 3, pos: 0 },
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 3 }
    ];
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_int_() {
    let mut tokinizer = Tokenizer::new("int_");
    let tokens = tokinizer.tokenize();
    let expected = vec![
        Token { kind: TK_IDENT, val: None, str: "int_".to_string(), len: 4, pos: 0 }, 
        Token { kind: TK_EOF, val: None, str: "<EOF>".to_string(), len: 1, pos: 4 }
    ];
    assert_eq!(tokens, expected);
}