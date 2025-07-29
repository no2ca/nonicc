use rs9cc::types::{ Token, TokenKind::* };
use rs9cc::lexer::Tokenizer;

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