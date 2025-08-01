use super::{Lexer, Token, AST, ASTNode, ScriptMetadata};
use super::shell_dialect::ShellDialect;
use anyhow::{Result, Context, bail};
use std::path::Path;

pub struct ShellParser {
    lexer: Lexer<'static>,
    current_token: Token,
    dialect: ShellDialect,
    input: String,
}

impl ShellParser {
    pub fn new(input: String, dialect: ShellDialect) -> Result<Self> {
        // We need to leak the string to get a 'static lifetime for the lexer
        // This is safe because we're storing the String in the parser
        let input_ref = unsafe { std::mem::transmute::<&str, &'static str>(input.as_str()) };
        let mut lexer = Lexer::new(input_ref, dialect);
        let current_token = lexer.next_token()?;
        
        Ok(ShellParser {
            lexer,
            current_token,
            dialect,
            input,
        })
    }
    
    pub fn parse(&mut self) -> Result<AST> {
        let metadata = self.extract_metadata()?;
        let root = self.parse_script()?;
        
        Ok(AST { root, metadata })
    }
    
    fn extract_metadata(&self) -> Result<ScriptMetadata> {
        let mut metadata = ScriptMetadata::default();
        
        // Extract shebang
        if self.input.starts_with("#!") {
            if let Some(line_end) = self.input.find('\n') {
                metadata.shebang = Some(self.input[..line_end].to_string());
            }
        }
        
        // Extract header comments
        for line in self.input.lines() {
            let line = line.trim();
            if line.starts_with("#") {
                let comment = line.trim_start_matches('#').trim();
                if let Some(version) = comment.strip_prefix("@Version:") {
                    metadata.version = Some(version.trim().to_string());
                } else if let Some(author) = comment.strip_prefix("@Author:") {
                    metadata.author = Some(author.trim().to_string());
                } else if let Some(desc) = comment.strip_prefix("@Description:") {
                    metadata.description = Some(desc.trim().to_string());
                } else if let Some(dep) = comment.strip_prefix("@Dependency:") {
                    metadata.dependencies.push(dep.trim().to_string());
                } else if comment.contains(':') {
                    if let Some((key, value)) = comment.split_once(':') {
                        if key.starts_with('@') {
                            metadata.headers.insert(
                                key.trim_start_matches('@').to_string(),
                                value.trim().to_string()
                            );
                        }
                    }
                }
            } else if !line.is_empty() {
                // Stop at first non-comment line
                break;
            }
        }
        
        Ok(metadata)
    }
    
    fn parse_script(&mut self) -> Result<ASTNode> {
        let mut statements = Vec::new();
        
        while self.current_token != Token::Eof {
            // Skip newlines at top level
            if self.current_token == Token::Newline {
                self.advance()?;
                continue;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(Box::new(stmt));
            
            // Consume optional terminators
            while matches!(self.current_token, Token::Semicolon | Token::Newline) {
                self.advance()?;
            }
        }
        
        Ok(ASTNode::Script(statements))
    }
    
    fn parse_statement(&mut self) -> Result<ASTNode> {
        match &self.current_token {
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Until => self.parse_until(),
            Token::For => self.parse_for(),
            Token::Case => self.parse_case(),
            Token::Function => self.parse_function(),
            Token::Export => self.parse_export(),
            Token::Local => self.parse_local(),
            Token::Readonly => self.parse_readonly(),
            Token::Return => self.parse_return(),
            Token::Break => {
                self.advance()?;
                Ok(ASTNode::Break)
            }
            Token::Continue => {
                self.advance()?;
                Ok(ASTNode::Continue)
            }
            Token::Exit => self.parse_exit(),
            Token::LeftBrace => self.parse_block(),
            Token::LeftParen => self.parse_subshell(),
            _ => self.parse_command_or_assignment(),
        }
    }
    
    fn parse_if(&mut self) -> Result<ASTNode> {
        self.expect(Token::If)?;
        let condition = self.parse_condition()?;
        self.expect(Token::Then)?;
        self.skip_newlines();
        
        let then_block = self.parse_block_until(&[Token::Elif, Token::Else, Token::Fi])?;
        
        let mut elif_blocks = Vec::new();
        while self.current_token == Token::Elif {
            self.advance()?;
            let elif_condition = self.parse_condition()?;
            self.expect(Token::Then)?;
            self.skip_newlines();
            let elif_block = self.parse_block_until(&[Token::Elif, Token::Else, Token::Fi])?;
            elif_blocks.push((Box::new(elif_condition), Box::new(elif_block)));
        }
        
        let else_block = if self.current_token == Token::Else {
            self.advance()?;
            self.skip_newlines();
            Some(Box::new(self.parse_block_until(&[Token::Fi])?))
        } else {
            None
        };
        
        self.expect(Token::Fi)?;
        
        Ok(ASTNode::If {
            condition: Box::new(condition),
            then_block: Box::new(then_block),
            elif_blocks,
            else_block,
        })
    }
    
    fn parse_while(&mut self) -> Result<ASTNode> {
        self.expect(Token::While)?;
        let condition = self.parse_condition()?;
        self.expect(Token::Do)?;
        self.skip_newlines();
        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;
        
        Ok(ASTNode::While {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }
    
    fn parse_until(&mut self) -> Result<ASTNode> {
        self.expect(Token::Until)?;
        let condition = self.parse_condition()?;
        self.expect(Token::Do)?;
        self.skip_newlines();
        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;
        
        Ok(ASTNode::Until {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }
    
    fn parse_for(&mut self) -> Result<ASTNode> {
        self.expect(Token::For)?;
        
        let variable = match &self.current_token {
            Token::Word(name) => {
                let var = name.clone();
                self.advance()?;
                var
            }
            _ => bail!("Expected variable name after 'for'"),
        };
        
        // TODO: Implement full for loop parsing including C-style for loops
        // For now, just handle basic for..in loops
        self.expect(Token::In)?;
        
        let mut items = Vec::new();
        while !matches!(self.current_token, Token::Do | Token::Semicolon | Token::Newline) {
            items.push(Box::new(self.parse_word()?));
        }
        
        self.skip_newlines();
        self.expect(Token::Do)?;
        self.skip_newlines();
        
        let body = self.parse_block_until(&[Token::Done])?;
        self.expect(Token::Done)?;
        
        Ok(ASTNode::For {
            variable,
            items: ForItems::List(items),
            body: Box::new(body),
        })
    }
    
    fn parse_case(&mut self) -> Result<ASTNode> {
        // TODO: Implement case statement parsing
        bail!("Case statements not yet implemented")
    }
    
    fn parse_function(&mut self) -> Result<ASTNode> {
        self.expect(Token::Function)?;
        
        let name = match &self.current_token {
            Token::Word(n) => {
                let name = n.clone();
                self.advance()?;
                name
            }
            _ => bail!("Expected function name"),
        };
        
        // Optional parentheses
        if self.current_token == Token::LeftParen {
            self.advance()?;
            self.expect(Token::RightParen)?;
        }
        
        self.skip_newlines();
        
        let body = if self.current_token == Token::LeftBrace {
            self.parse_block()?
        } else {
            self.parse_statement()?
        };
        
        Ok(ASTNode::Function {
            name,
            body: Box::new(body),
        })
    }
    
    fn parse_export(&mut self) -> Result<ASTNode> {
        self.expect(Token::Export)?;
        let mut node = self.parse_assignment_or_word()?;
        
        if let ASTNode::Assignment { export, .. } = &mut node {
            *export = true;
        }
        
        Ok(node)
    }
    
    fn parse_local(&mut self) -> Result<ASTNode> {
        self.expect(Token::Local)?;
        let mut node = self.parse_assignment_or_word()?;
        
        if let ASTNode::Assignment { local, .. } = &mut node {
            *local = true;
        }
        
        Ok(node)
    }
    
    fn parse_readonly(&mut self) -> Result<ASTNode> {
        self.expect(Token::Readonly)?;
        let mut node = self.parse_assignment_or_word()?;
        
        if let ASTNode::Assignment { readonly, .. } = &mut node {
            *readonly = true;
        }
        
        Ok(node)
    }
    
    fn parse_return(&mut self) -> Result<ASTNode> {
        self.expect(Token::Return)?;
        
        let value = if matches!(self.current_token, Token::Newline | Token::Semicolon | Token::Eof) {
            None
        } else {
            Some(Box::new(self.parse_word()?))
        };
        
        Ok(ASTNode::Return(value))
    }
    
    fn parse_exit(&mut self) -> Result<ASTNode> {
        self.expect(Token::Exit)?;
        
        let code = if matches!(self.current_token, Token::Newline | Token::Semicolon | Token::Eof) {
            None
        } else {
            Some(Box::new(self.parse_word()?))
        };
        
        Ok(ASTNode::Exit(code))
    }
    
    fn parse_block(&mut self) -> Result<ASTNode> {
        self.expect(Token::LeftBrace)?;
        self.skip_newlines();
        
        let block = self.parse_block_until(&[Token::RightBrace])?;
        self.expect(Token::RightBrace)?;
        
        Ok(block)
    }
    
    fn parse_subshell(&mut self) -> Result<ASTNode> {
        self.expect(Token::LeftParen)?;
        self.skip_newlines();
        
        let mut statements = Vec::new();
        while self.current_token != Token::RightParen && self.current_token != Token::Eof {
            statements.push(Box::new(self.parse_statement()?));
            self.skip_terminators();
        }
        
        self.expect(Token::RightParen)?;
        
        Ok(ASTNode::Subshell(Box::new(ASTNode::Block(statements))))
    }
    
    fn parse_command_or_assignment(&mut self) -> Result<ASTNode> {
        // Check if this looks like an assignment
        if let Token::Word(name) = &self.current_token {
            let mut chars = self.input[self.lexer.position..].chars();
            if chars.next() == Some('=') || (chars.next() == Some('+') && chars.next() == Some('=')) {
                return self.parse_assignment();
            }
        }
        
        self.parse_pipeline()
    }
    
    fn parse_assignment(&mut self) -> Result<ASTNode> {
        let name = match &self.current_token {
            Token::Word(n) => n.clone(),
            _ => bail!("Expected variable name"),
        };
        
        self.advance()?;
        
        let _op = match &self.current_token {
            Token::Assign => {
                self.advance()?;
                "="
            }
            Token::PlusAssign => {
                self.advance()?;
                "+="
            }
            _ => bail!("Expected assignment operator"),
        };
        
        let value = self.parse_word()?;
        
        Ok(ASTNode::Assignment {
            name,
            value: Box::new(value),
            export: false,
            readonly: false,
            local: false,
        })
    }
    
    fn parse_assignment_or_word(&mut self) -> Result<ASTNode> {
        // This is used after export/local/readonly
        // TODO: Implement proper parsing
        self.parse_word()
    }
    
    fn parse_pipeline(&mut self) -> Result<ASTNode> {
        let mut commands = vec![Box::new(self.parse_command()?)];
        
        while matches!(self.current_token, Token::Pipe | Token::PipeErr) {
            self.advance()?;
            self.skip_newlines();
            commands.push(Box::new(self.parse_command()?));
        }
        
        if commands.len() == 1 {
            Ok(*commands.into_iter().next().unwrap())
        } else {
            Ok(ASTNode::Pipeline(commands))
        }
    }
    
    fn parse_command(&mut self) -> Result<ASTNode> {
        let name = match &self.current_token {
            Token::Word(n) => n.clone(),
            _ => bail!("Expected command name"),
        };
        
        self.advance()?;
        
        let mut args = Vec::new();
        let mut redirections = Vec::new();
        
        while !matches!(
            self.current_token,
            Token::Pipe | Token::PipeErr | Token::Semicolon | Token::Newline | 
            Token::Background | Token::And | Token::Or | Token::Eof
        ) {
            match &self.current_token {
                Token::Redirect(_) => {
                    // TODO: Parse redirections properly
                    self.advance()?;
                    self.advance()?; // Skip target for now
                }
                _ => {
                    args.push(Box::new(self.parse_word()?));
                }
            }
        }
        
        let background = if self.current_token == Token::Background {
            self.advance()?;
            true
        } else {
            false
        };
        
        Ok(ASTNode::Command {
            name,
            args,
            redirections,
            background,
        })
    }
    
    fn parse_condition(&mut self) -> Result<ASTNode> {
        // For now, just parse as a command
        // TODO: Implement proper condition parsing with test commands
        self.parse_pipeline()
    }
    
    fn parse_word(&mut self) -> Result<ASTNode> {
        match &self.current_token.clone() {
            Token::Word(w) => {
                let word = w.clone();
                self.advance()?;
                Ok(ASTNode::String(word, StringType::Unquoted))
            }
            Token::String(s, quote_type) => {
                let string = s.clone();
                let string_type = match quote_type {
                    super::lexer::QuoteType::Single => StringType::SingleQuoted,
                    super::lexer::QuoteType::Double => StringType::DoubleQuoted,
                    super::lexer::QuoteType::Ansi => StringType::AnsiC,
                    super::lexer::QuoteType::Backtick => {
                        self.advance()?;
                        return Ok(ASTNode::CommandSubstitution(
                            Box::new(ASTNode::String(string, StringType::Unquoted))
                        ));
                    }
                };
                self.advance()?;
                Ok(ASTNode::String(string, string_type))
            }
            Token::Number(n) => {
                let num = n.parse::<f64>().context("Invalid number")?;
                self.advance()?;
                Ok(ASTNode::Number(num))
            }
            Token::Dollar => {
                self.advance()?;
                self.parse_variable_or_expansion()
            }
            _ => bail!("Unexpected token: {:?}", self.current_token),
        }
    }
    
    fn parse_variable_or_expansion(&mut self) -> Result<ASTNode> {
        match &self.current_token {
            Token::Word(name) => {
                let var = name.clone();
                self.advance()?;
                Ok(ASTNode::Variable(var))
            }
            Token::LeftBrace => {
                // TODO: Parse parameter expansion
                bail!("Parameter expansion not yet implemented")
            }
            Token::LeftParen => {
                // TODO: Parse command substitution
                bail!("Command substitution not yet implemented")
            }
            _ => bail!("Unexpected token after $: {:?}", self.current_token),
        }
    }
    
    fn parse_block_until(&mut self, terminators: &[Token]) -> Result<ASTNode> {
        let mut statements = Vec::new();
        
        while !terminators.contains(&self.current_token) && self.current_token != Token::Eof {
            if self.current_token == Token::Newline {
                self.advance()?;
                continue;
            }
            
            statements.push(Box::new(self.parse_statement()?));
            self.skip_terminators();
        }
        
        Ok(ASTNode::Block(statements))
    }
    
    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
    
    fn expect(&mut self, expected: Token) -> Result<()> {
        if std::mem::discriminant(&self.current_token) != std::mem::discriminant(&expected) {
            bail!("Expected {:?}, found {:?}", expected, self.current_token);
        }
        self.advance()
    }
    
    fn skip_newlines(&mut self) {
        while self.current_token == Token::Newline {
            let _ = self.advance();
        }
    }
    
    fn skip_terminators(&mut self) {
        while matches!(self.current_token, Token::Semicolon | Token::Newline) {
            let _ = self.advance();
        }
    }
}

use super::ast::{StringType, ForItems};