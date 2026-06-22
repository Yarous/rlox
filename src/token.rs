use std::fmt::{write, Display, Formatter};
use strum::Display;

#[derive(Display)]
pub(crate) enum TokenKind {
    // Single-Character tokens
    LeftParen,        // (
    RightParen,       // )
    LeftBrace,        // {
    RightBrace,       // }
    Comma,            // ,
    Dot,              // .
    Minus,            // -
    Plus,             // +
    Semicolon,        // "
    Slash,            // /
    Star,             // *

    Bang,             // !
    BangEqual,        // !=
    Equal,            // =
    EqualEqual,       // ==
    Greater,          // >
    GreaterEqual,     // >=
    Less,             // <
    LessEqual,        // <=

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF
}

pub(crate) struct Token {
    token_type: TokenKind,
    lexeme: String,
    literal: usize,
    line: usize,
}

impl Token {
    pub(crate) fn new(token_type: TokenKind, lexeme: String, literal: usize, line: usize) -> Token {
        Self {
            token_type,
            lexeme,
            literal,
            line
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.token_type, self.lexeme, self.literal)
    }
}

