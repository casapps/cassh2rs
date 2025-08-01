use cassh2rs::parser::{Lexer, Token, shell_dialect::ShellDialect, lexer::{QuoteType, RedirectOp}};

#[test]
fn test_basic_tokens() {
    let input = "echo hello world";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::Echo);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("hello".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Word("world".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Eof);
}

#[test]
fn test_string_tokens() {
    let input = r#"'single' "double" $'ansi\n'"#;
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    match lexer.next_token().unwrap() {
        Token::String(s, QuoteType::Single) => assert_eq!(s, "single"),
        _ => panic!("Expected single quoted string"),
    }
    
    match lexer.next_token().unwrap() {
        Token::String(s, QuoteType::Double) => assert_eq!(s, "double"),
        _ => panic!("Expected double quoted string"),
    }
    
    match lexer.next_token().unwrap() {
        Token::String(s, QuoteType::Ansi) => assert_eq!(s, "ansi\n"),
        _ => panic!("Expected ANSI-C quoted string"),
    }
}

#[test]
fn test_operators() {
    let input = "| || & && > >> < << ; ()";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::Pipe);
    assert_eq!(lexer.next_token().unwrap(), Token::Or);
    assert_eq!(lexer.next_token().unwrap(), Token::Background);
    assert_eq!(lexer.next_token().unwrap(), Token::And);
    assert_eq!(lexer.next_token().unwrap(), Token::Redirect(RedirectOp::Out));
    assert_eq!(lexer.next_token().unwrap(), Token::Redirect(RedirectOp::OutAppend));
    assert_eq!(lexer.next_token().unwrap(), Token::Redirect(RedirectOp::In));
    
    // Handle heredoc
    let heredoc_token = lexer.next_token().unwrap();
    match heredoc_token {
        Token::Heredoc(_) => {},
        _ => panic!("Expected heredoc token"),
    }
    
    assert_eq!(lexer.next_token().unwrap(), Token::Semicolon);
    assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
}

#[test]
fn test_variables() {
    let input = "$VAR ${VAR} $(cmd) $((1+2))";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::Dollar);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("VAR".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::DollarBrace);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("VAR".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
    assert_eq!(lexer.next_token().unwrap(), Token::DollarParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("cmd".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
    assert_eq!(lexer.next_token().unwrap(), Token::DollarDoubleParen);
    assert_eq!(lexer.next_token().unwrap(), Token::Number("1".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Plus);
    assert_eq!(lexer.next_token().unwrap(), Token::Number("2".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
    assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
}

#[test]
fn test_keywords() {
    let input = "if then else elif fi for while do done function";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::If);
    assert_eq!(lexer.next_token().unwrap(), Token::Then);
    assert_eq!(lexer.next_token().unwrap(), Token::Else);
    assert_eq!(lexer.next_token().unwrap(), Token::Elif);
    assert_eq!(lexer.next_token().unwrap(), Token::Fi);
    assert_eq!(lexer.next_token().unwrap(), Token::For);
    assert_eq!(lexer.next_token().unwrap(), Token::While);
    assert_eq!(lexer.next_token().unwrap(), Token::Do);
    assert_eq!(lexer.next_token().unwrap(), Token::Done);
    assert_eq!(lexer.next_token().unwrap(), Token::Function);
}

#[test]
fn test_comments() {
    let input = "echo hello # this is a comment\necho world";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::Echo);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("hello".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Newline);
    assert_eq!(lexer.next_token().unwrap(), Token::Echo);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("world".to_string()));
}

#[test]
fn test_test_brackets() {
    let input = "[ -f file ] [[ $var == pattern ]]";
    let mut lexer = Lexer::new(input, ShellDialect::Bash);
    
    assert_eq!(lexer.next_token().unwrap(), Token::LeftBracket);
    assert_eq!(lexer.next_token().unwrap(), Token::Minus);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("f".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Word("file".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::RightBracket);
    assert_eq!(lexer.next_token().unwrap(), Token::DoubleLeftBracket);
    assert_eq!(lexer.next_token().unwrap(), Token::Dollar);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("var".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::Equal);
    assert_eq!(lexer.next_token().unwrap(), Token::Word("pattern".to_string()));
    assert_eq!(lexer.next_token().unwrap(), Token::DoubleRightBracket);
}