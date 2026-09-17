use crate::source::SourcePos;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier(String),
    Integer(i64),
    String(String),
    Let,
    Mut,
    Borrow,
    Fn,
    Return,
    If,
    Else,
    While,
    True,
    False,
    TypeInt,
    TypeBool,
    TypeString,
    TypeUnit,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }

    pub fn pos(&self) -> SourcePos {
        SourcePos::new(self.line, self.column)
    }
}
