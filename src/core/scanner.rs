use crate::AmystError;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub token_type: TokenType<'a>,
    pub lexeme: &'a str,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType<'a> {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Arrow,
    Dot,
    DotDot,
    DotDotEqual,
    Minus,
    Plus,
    Semicolon,
    Colon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(&'a str),
    String(&'a str),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fn,
    Mut,
    For,
    If,
    Unit,
    In,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    Eof,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {}", self.token_type, self.lexeme, self.line)
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            tokens: vec![],
            current: 0,
            start: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token<'a>>, AmystError<'a>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: "",
            line: self.line,
        });

        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<(), AmystError<'a>> {
        let c = self.advance();
        match c {
            ':' => Ok(self.add_token(TokenType::Colon)),
            '(' => Ok(self.add_token(TokenType::LeftParen)),
            ')' => Ok(self.add_token(TokenType::RightParen)),
            '{' => Ok(self.add_token(TokenType::LeftBrace)),
            '}' => Ok(self.add_token(TokenType::RightBrace)),
            ',' => Ok(self.add_token(TokenType::Comma)),
            '+' => Ok(self.add_token(TokenType::Plus)),
            ';' => Ok(self.add_token(TokenType::Semicolon)),
            '*' => Ok(self.add_token(TokenType::Star)),
            '"' => self.add_string_token(),
            '-' => {
                let t = if self.matches('>') {
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                };
                Ok(self.add_token(t))
            }
            '!' => {
                let t = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                Ok(self.add_token(t))
            }
            '=' => {
                let t = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                Ok(self.add_token(t))
            }
            '<' => {
                let t = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                Ok(self.add_token(t))
            }
            '>' => {
                let t = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                Ok(self.add_token(t))
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(())
                } else {
                    Ok(self.add_token(TokenType::Slash))
                }
            }
            '.' => {
                let t = if self.matches('.') {
                    if self.matches('=') {
                        TokenType::DotDotEqual
                    } else {
                        TokenType::DotDot
                    }
                } else {
                    TokenType::Dot
                };
                Ok(self.add_token(t))
            }
            ' ' | '\r' | '\t' => Ok(()),
            '\n' => {
                self.line += 1;
                Ok(())
            }
            c => {
                if c.is_digit(10) {
                    self.add_number_token()
                } else if c.is_ascii_alphanumeric() {
                    Ok(self.add_identifier())
                } else {
                    Err(AmystError::UnexpectedCharacter(self.line))
                }
            }
        }
    }

    fn add_identifier(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[self.start..self.current];

        let token_type = match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fn" => TokenType::Fn,
            "mut" => TokenType::Mut,
            "if" => TokenType::If,
            "in" => TokenType::In,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "let" => TokenType::Let,
            "while" => TokenType::While,
            _ => TokenType::Identifier(text),
        };

        self.add_token(token_type);
    }

    fn add_number_token(&mut self) -> Result<(), AmystError<'a>> {
        while self.peek().is_some() && self.peek().unwrap().is_digit(10) {
            self.advance();
        }

        if self.peek() == Some('.') && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_some() && self.peek().unwrap().is_digit(10) {
                self.advance();
            }
        }
        match f64::from_str(&self.source[self.start..self.current]) {
            Ok(n) => Ok(self.add_token(TokenType::Number(n))),
            Err(_) => Err(AmystError::InvalidNumberFormat),
        }
    }
    fn add_string_token(&mut self) -> Result<(), AmystError<'a>> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(AmystError::UnterminatedString);
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];
        self.add_token(TokenType::String(value));
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current..].chars().next().unwrap();
        self.current += c.len_utf8();
        c
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current..].chars().next() != Some(expected) {
            return false;
        }
        self.current += expected.len_utf8();
        true
    }

    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current + 1..]
            .chars()
            .next()
            .unwrap_or('\n')
    }

    fn add_token(&mut self, token_type: TokenType<'a>) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens.push(Token {
            token_type,
            lexeme,
            line: self.line,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::core::Scanner;
    use crate::core::scanner::TokenType;

    #[test]
    fn char_token_is_read() {
        let mut scanner = Scanner::new("(!),*");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(tokens[1].token_type, TokenType::Bang);
        assert_eq!(tokens[2].token_type, TokenType::RightParen);
        assert_eq!(tokens[3].token_type, TokenType::Comma);
        assert_eq!(tokens[4].token_type, TokenType::Star);
    }

    #[test]
    fn multichar_token_is_read() {
        let scanner = Scanner::new("!=,==..=");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[0].token_type, TokenType::BangEqual);
        assert_eq!(tokens[1].token_type, TokenType::Comma);
        assert_eq!(tokens[2].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[3].token_type, TokenType::DotDotEqual);
    }

    #[test]
    fn range_distinguishable_from_number() {
        let scanner = Scanner::new("3..5");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[0].token_type, TokenType::Number(3.0));
        assert_eq!(tokens[1].token_type, TokenType::DotDot);
        assert_eq!(tokens[2].token_type, TokenType::Number(5.0));

        let scanner = Scanner::new("3.5..5");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        for tk in tokens.clone() {
            println!("{:?}", tk.token_type)
        }
        assert_eq!(tokens[0].token_type, TokenType::Number(3.5));
        assert_eq!(tokens[1].token_type, TokenType::DotDot);
        assert_eq!(tokens[2].token_type, TokenType::Number(5.0));
    }

    #[test]
    fn multiline_files_processed() {
        let scanner = Scanner::new("!=,==.\n*+;");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[0].token_type, TokenType::BangEqual);
        assert_eq!(tokens[1].token_type, TokenType::Comma);
        assert_eq!(tokens[2].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[3].token_type, TokenType::Dot);
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[4].token_type, TokenType::Star);
        assert_eq!(tokens[5].token_type, TokenType::Plus);
        assert_eq!(tokens[6].token_type, TokenType::Semicolon);
        assert_eq!(tokens[4].line, 2);
    }

    #[test]
    fn ignore_comments() {
        let scanner = Scanner::new("!=,==.\n//hello_world\n==");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[0].token_type, TokenType::BangEqual);
        assert_eq!(tokens[1].token_type, TokenType::Comma);
        assert_eq!(tokens[2].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[3].token_type, TokenType::Dot);
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[4].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[4].line, 3);
    }

    #[test]
    fn string_token_is_read() {
        let scanner = Scanner::new("==,\"Hola Mundo\" ,!=");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[2].token_type, TokenType::String("Hola Mundo"));
    }

    #[test]
    fn number_token_is_read() {
        let scanner = Scanner::new("==,3 + 4.67 ,!=");
        let tokens = scanner.scan_tokens().expect("Todo mal");
        assert_eq!(tokens[2].token_type, TokenType::Number(3.0));
        assert_eq!(tokens[4].token_type, TokenType::Number(4.67));
    }
}
