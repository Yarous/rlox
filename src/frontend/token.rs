use std::fmt::Display;
use strum::Display;

#[derive(Display, Debug, Clone, Copy, PartialEq)]
pub(crate) enum TokenKind {
    // Single-Character tokens
    LeftParen,  // (
    RightParen, // )
    LeftBrace,  // {
    RightBrace, // }
    Comma,      // ,
    Dot,        // .
    Minus,      // -
    Plus,       // +
    Semicolon,  // ;
    Colon,      // :
    Slash,      // /
    Star,       // *
    Question,   // ?

    Bang,         // !
    BangEqual,    // !=
    Equal,        // =
    EqualEqual,   // ==
    Greater,      // >
    GreaterEqual, // >=
    Less,         // <
    LessEqual,    // <=

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
    Break,

    Eof,
}

#[derive(Debug, Display, Clone, PartialEq)]
pub(crate) enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) token_type: TokenKind,
    pub(crate) lexeme: String,
    pub(crate) literal: Option<Literal>,
    pub(crate) line: usize,
}

impl Token {
    #[must_use]
    pub(crate) fn new(
        token_type: TokenKind,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) -> Token {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(literal) = &self.literal {
            let token_type = self.token_type;
            let lexeme = &self.lexeme;
            write!(f, "{token_type} {lexeme} {literal}")
        } else {
            let token_type = self.token_type;
            let lexeme = &self.lexeme;
            write!(f, "{token_type} {lexeme}")
        }
    }
}
