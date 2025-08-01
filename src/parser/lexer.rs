use super::shell_dialect::ShellDialect;
use anyhow::{Result, bail};
use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Word(String),
    Number(String),
    String(String, QuoteType),
    
    // Operators
    Pipe,                  // |
    PipeErr,               // |&
    Redirect(RedirectOp),
    Background,            // &
    Semicolon,             // ;
    Newline,
    
    // Logical operators
    And,                   // &&
    Or,                    // ||
    
    // Assignment
    Assign,                // =
    PlusAssign,            // +=
    
    // Arithmetic
    Plus,                  // +
    Minus,                 // -
    Star,                  // *
    Slash,                 // /
    Percent,               // %
    
    // Comparison
    Equal,                 // ==
    NotEqual,              // !=
    Less,                  // <
    Greater,               // >
    LessEqual,             // <=
    GreaterEqual,          // >=
    
    // Grouping
    LeftParen,             // (
    RightParen,            // )
    LeftBrace,             // {
    RightBrace,            // }
    LeftBracket,           // [
    RightBracket,          // ]
    DoubleLeftBracket,     // [[
    DoubleRightBracket,    // ]]
    
    // Variables and expansion
    Dollar,                // $
    DollarBrace,           // ${
    DollarParen,           // $(
    DollarDoubleParen,     // $((
    Backtick,              // `
    AtSign,                // @
    Hash,                  // #
    
    // Keywords
    If,
    Then,
    Else,
    Elif,
    Fi,
    Case,
    Esac,
    For,
    In,
    Do,
    Done,
    While,
    Until,
    Function,
    Return,
    Export,
    Local,
    Readonly,
    Declare,
    Typeset,
    Let,
    Select,
    Time,
    
    // Built-ins
    Echo,
    Printf,
    Read,
    Cd,
    Pwd,
    Exit,
    Source,
    Dot,                   // .
    Exec,
    Eval,
    
    // Special
    Bang,                  // !
    Question,              // ?
    Tilde,                 // ~
    Heredoc(String),       // <<EOF
    HereString,            // <<<
    
    // End of input
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    Single,
    Double,
    Backtick,
    Ansi,  // $'...'
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectOp {
    Out,           // >
    OutAppend,     // >>
    In,            // <
    InOut,         // <>
    OutErr,        // >&
    ErrOut,        // 2>&1
    HereDoc,       // <<
    HereString,    // <<<
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_char: Option<char>,
    position: usize,
    line: usize,
    column: usize,
    dialect: ShellDialect,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, dialect: ShellDialect) -> Self {
        let mut lexer = Lexer {
            input: input.chars().peekable(),
            current_char: None,
            position: 0,
            line: 1,
            column: 0,
            dialect,
        };
        lexer.advance();
        lexer
    }
    
    fn advance(&mut self) {
        self.current_char = self.input.next();
        self.position += 1;
        
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
    }
    
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }
    
    fn peek_ahead(&mut self, n: usize) -> Option<char> {
        let mut temp_iter = self.input.clone();
        for _ in 0..n-1 {
            temp_iter.next();
        }
        temp_iter.next()
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn skip_comment(&mut self) {
        // Skip until end of line
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
    
    fn read_word(&mut self) -> String {
        let mut word = String::new();
        
        while let Some(ch) = self.current_char {
            match ch {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '/' => {
                    word.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }
        
        word
    }
    
    fn read_string(&mut self, quote_char: char) -> Result<(String, QuoteType)> {
        let quote_type = match quote_char {
            '\'' => QuoteType::Single,
            '"' => QuoteType::Double,
            '`' => QuoteType::Backtick,
            _ => bail!("Invalid quote character"),
        };
        
        let mut string = String::new();
        self.advance(); // Skip opening quote
        
        while let Some(ch) = self.current_char {
            if ch == quote_char {
                self.advance(); // Skip closing quote
                return Ok((string, quote_type));
            } else if ch == '\\' && quote_type != QuoteType::Single {
                self.advance();
                if let Some(escaped) = self.current_char {
                    string.push('\\');
                    string.push(escaped);
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }
        
        bail!("Unterminated string at line {}, column {}", self.line, self.column)
    }
    
    fn read_ansi_string(&mut self) -> Result<(String, QuoteType)> {
        self.advance(); // Skip $
        self.advance(); // Skip '
        
        let mut string = String::new();
        
        while let Some(ch) = self.current_char {
            if ch == '\'' {
                self.advance();
                return Ok((string, QuoteType::Ansi));
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    // Handle ANSI-C escape sequences
                    let escaped_char = match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        'b' => '\x08',
                        'f' => '\x0C',
                        'a' => '\x07',
                        'v' => '\x0B',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        _ => escaped,
                    };
                    string.push(escaped_char);
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }
        
        bail!("Unterminated ANSI string at line {}, column {}", self.line, self.column)
    }
    
    fn read_number(&mut self) -> String {
        let mut number = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_numeric() || (ch == '.' && !number.contains('.')) {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        number
    }
    
    fn match_keyword(&self, word: &str) -> Option<Token> {
        match word {
            "if" => Some(Token::If),
            "then" => Some(Token::Then),
            "else" => Some(Token::Else),
            "elif" => Some(Token::Elif),
            "fi" => Some(Token::Fi),
            "case" => Some(Token::Case),
            "esac" => Some(Token::Esac),
            "for" => Some(Token::For),
            "in" => Some(Token::In),
            "do" => Some(Token::Do),
            "done" => Some(Token::Done),
            "while" => Some(Token::While),
            "until" => Some(Token::Until),
            "function" => Some(Token::Function),
            "return" => Some(Token::Return),
            "export" => Some(Token::Export),
            "local" => Some(Token::Local),
            "readonly" => Some(Token::Readonly),
            "declare" => Some(Token::Declare),
            "typeset" => Some(Token::Typeset),
            "let" => Some(Token::Let),
            "select" => Some(Token::Select),
            "time" => Some(Token::Time),
            "echo" => Some(Token::Echo),
            "printf" => Some(Token::Printf),
            "read" => Some(Token::Read),
            "cd" => Some(Token::Cd),
            "pwd" => Some(Token::Pwd),
            "exit" => Some(Token::Exit),
            "source" => Some(Token::Source),
            "exec" => Some(Token::Exec),
            "eval" => Some(Token::Eval),
            _ => None,
        }
    }
    
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        
        match self.current_char {
            None => Ok(Token::Eof),
            Some('\n') => {
                self.advance();
                Ok(Token::Newline)
            }
            Some('#') => {
                self.skip_comment();
                self.next_token()
            }
            Some('\'') | Some('"') | Some('`') => {
                let (string, quote_type) = self.read_string(self.current_char.unwrap())?;
                Ok(Token::String(string, quote_type))
            }
            Some('$') => {
                self.advance();
                match self.current_char {
                    Some('\'') => {
                        let (string, quote_type) = self.read_ansi_string()?;
                        Ok(Token::String(string, quote_type))
                    }
                    Some('{') => {
                        self.advance();
                        Ok(Token::DollarBrace)
                    }
                    Some('(') => {
                        self.advance();
                        if self.current_char == Some('(') {
                            self.advance();
                            Ok(Token::DollarDoubleParen)
                        } else {
                            Ok(Token::DollarParen)
                        }
                    }
                    _ => Ok(Token::Dollar),
                }
            }
            Some('|') => {
                self.advance();
                match self.current_char {
                    Some('|') => {
                        self.advance();
                        Ok(Token::Or)
                    }
                    Some('&') => {
                        self.advance();
                        Ok(Token::PipeErr)
                    }
                    _ => Ok(Token::Pipe),
                }
            }
            Some('&') => {
                self.advance();
                match self.current_char {
                    Some('&') => {
                        self.advance();
                        Ok(Token::And)
                    }
                    _ => Ok(Token::Background),
                }
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some('(') => {
                self.advance();
                Ok(Token::LeftParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RightBrace)
            }
            Some('[') => {
                self.advance();
                if self.current_char == Some('[') {
                    self.advance();
                    Ok(Token::DoubleLeftBracket)
                } else {
                    Ok(Token::LeftBracket)
                }
            }
            Some(']') => {
                self.advance();
                if self.current_char == Some(']') {
                    self.advance();
                    Ok(Token::DoubleRightBracket)
                } else {
                    Ok(Token::RightBracket)
                }
            }
            Some('>') => {
                self.advance();
                match self.current_char {
                    Some('>') => {
                        self.advance();
                        Ok(Token::Redirect(RedirectOp::OutAppend))
                    }
                    Some('&') => {
                        self.advance();
                        Ok(Token::Redirect(RedirectOp::OutErr))
                    }
                    Some('=') => {
                        self.advance();
                        Ok(Token::GreaterEqual)
                    }
                    _ => Ok(Token::Redirect(RedirectOp::Out)),
                }
            }
            Some('<') => {
                self.advance();
                match self.current_char {
                    Some('<') => {
                        self.advance();
                        if self.current_char == Some('<') {
                            self.advance();
                            Ok(Token::HereString)
                        } else {
                            // Read heredoc delimiter
                            self.skip_whitespace();
                            let delimiter = self.read_word();
                            Ok(Token::Heredoc(delimiter))
                        }
                    }
                    Some('>') => {
                        self.advance();
                        Ok(Token::Redirect(RedirectOp::InOut))
                    }
                    Some('=') => {
                        self.advance();
                        Ok(Token::LessEqual)
                    }
                    _ => Ok(Token::Redirect(RedirectOp::In)),
                }
            }
            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::Equal)
                } else {
                    Ok(Token::Assign)
                }
            }
            Some('+') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::PlusAssign)
                } else {
                    Ok(Token::Plus)
                }
            }
            Some('-') => {
                self.advance();
                Ok(Token::Minus)
            }
            Some('*') => {
                self.advance();
                Ok(Token::Star)
            }
            Some('/') => {
                self.advance();
                Ok(Token::Slash)
            }
            Some('%') => {
                self.advance();
                Ok(Token::Percent)
            }
            Some('!') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Ok(Token::Bang)
                }
            }
            Some('?') => {
                self.advance();
                Ok(Token::Question)
            }
            Some('~') => {
                self.advance();
                Ok(Token::Tilde)
            }
            Some('@') => {
                self.advance();
                Ok(Token::AtSign)
            }
            Some('.') => {
                self.advance();
                if self.current_char.map(|c| c.is_numeric()).unwrap_or(false) {
                    let mut number = String::from("0.");
                    number.push_str(&self.read_number());
                    Ok(Token::Number(number))
                } else {
                    Ok(Token::Dot)
                }
            }
            Some(ch) if ch.is_numeric() => {
                let number = self.read_number();
                Ok(Token::Number(number))
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let word = self.read_word();
                if let Some(keyword) = self.match_keyword(&word) {
                    Ok(keyword)
                } else {
                    Ok(Token::Word(word))
                }
            }
            Some(ch) => {
                self.advance();
                Ok(Token::Word(ch.to_string()))
            }
        }
    }
}