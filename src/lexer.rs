use crate::error::KarmaError;
use crate::token::{Token, TokenKind};

pub struct Lexer {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, KarmaError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.scan_token(&mut tokens)?;
        }
        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }

    fn scan_token(&mut self, tokens: &mut Vec<Token>) -> Result<(), KarmaError> {
        let line = self.line;
        let column = self.column;
        let c = self.advance();

        match c {
            '(' => tokens.push(Token::new(TokenKind::LeftParen, line, column)),
            ')' => tokens.push(Token::new(TokenKind::RightParen, line, column)),
            '{' => tokens.push(Token::new(TokenKind::LeftBrace, line, column)),
            '}' => tokens.push(Token::new(TokenKind::RightBrace, line, column)),
            ',' => tokens.push(Token::new(TokenKind::Comma, line, column)),
            ';' => tokens.push(Token::new(TokenKind::Semicolon, line, column)),
            ':' => tokens.push(Token::new(TokenKind::Colon, line, column)),
            '+' => tokens.push(Token::new(TokenKind::Plus, line, column)),
            '-' => {
                let kind = if self.matches('>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                };
                tokens.push(Token::new(kind, line, column));
            }
            '*' => tokens.push(Token::new(TokenKind::Star, line, column)),
            '!' => {
                let kind = if self.matches('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                tokens.push(Token::new(kind, line, column));
            }
            '=' => {
                let kind = if self.matches('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                tokens.push(Token::new(kind, line, column));
            }
            '<' => {
                let kind = if self.matches('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                tokens.push(Token::new(kind, line, column));
            }
            '>' => {
                let kind = if self.matches('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                tokens.push(Token::new(kind, line, column));
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    tokens.push(Token::new(TokenKind::Slash, line, column));
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {}
            '"' => tokens.push(Token::new(self.string(line, column)?, line, column)),
            d if d.is_ascii_digit() => {
                tokens.push(Token::new(self.number(d, line, column)?, line, column))
            }
            a if is_identifier_start(a) => {
                tokens.push(Token::new(self.identifier(a), line, column))
            }
            other => {
                return Err(KarmaError::new(
                    "lexer",
                    format!("unexpected character '{other}'"),
                    line,
                    column,
                ))
            }
        }
        Ok(())
    }

    fn string(&mut self, line: usize, column: usize) -> Result<TokenKind, KarmaError> {
        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();
            if c == '\\' {
                if self.is_at_end() {
                    break;
                }
                let escaped = self.advance();
                let decoded = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '"' => '"',
                    '\\' => '\\',
                    other => {
                        return Err(KarmaError::new(
                            "lexer",
                            format!("unsupported escape sequence \\{other}"),
                            self.line,
                            self.column.saturating_sub(1),
                        ))
                    }
                };
                value.push(decoded);
            } else {
                value.push(c);
            }
        }

        if self.is_at_end() {
            return Err(KarmaError::new(
                "lexer",
                "unterminated string",
                line,
                column,
            ));
        }
        self.advance();
        Ok(TokenKind::String(value))
    }

    fn number(&mut self, first: char, line: usize, column: usize) -> Result<TokenKind, KarmaError> {
        let mut text = String::from(first);
        while self.peek().is_ascii_digit() {
            text.push(self.advance());
        }
        let value = text.parse::<i64>().map_err(|_| {
            KarmaError::new(
                "lexer",
                "integer literal is outside Int range",
                line,
                column,
            )
        })?;
        Ok(TokenKind::Integer(value))
    }

    fn identifier(&mut self, first: char) -> TokenKind {
        let mut text = String::from(first);
        while is_identifier_continue(self.peek()) {
            text.push(self.advance());
        }
        match text.as_str() {
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "borrow" => TokenKind::Borrow,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "Int" => TokenKind::TypeInt,
            "Bool" => TokenKind::TypeBool,
            "String" => TokenKind::TypeString,
            "Unit" => TokenKind::TypeUnit,
            _ => TokenKind::Identifier(text),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            return false;
        }
        self.advance();
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }
}

fn is_identifier_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn is_identifier_continue(c: char) -> bool {
    is_identifier_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_typed_binding() {
        let tokens = Lexer::new("let x: Int = 12;").scan_tokens().unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Let));
        assert!(matches!(tokens[1].kind, TokenKind::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].kind, TokenKind::Colon));
        assert!(matches!(tokens[3].kind, TokenKind::TypeInt));
    }

    #[test]
    fn lexes_mut_and_function_arrow() {
        let source = "mut x: Int = 1; fn f(a: Int) -> Int { return a; }";
        let tokens = Lexer::new(source).scan_tokens().unwrap();
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Mut)));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Arrow)));
    }

    #[test]
    fn lexes_borrow_parameter_mode() {
        let tokens = Lexer::new("fn show(s: borrow String) -> Unit { print(s); }")
            .scan_tokens()
            .unwrap();
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Borrow)));
    }

    #[test]
    fn skips_line_comments() {
        let tokens = Lexer::new("let x = 1; // note\nprint(x);")
            .scan_tokens()
            .unwrap();
        assert!(tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Identifier(ref s) if s == "print")));
    }
}
