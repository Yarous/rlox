use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{OnceLock, RwLock};
use strum::Display;

// TODO: we can use perfect hash functions instead
pub(crate) static KEYWORDS: OnceLock<RwLock<HashMap<&'static str, TokenKind>>> = OnceLock::new();

pub(crate) fn get_keywords_map() -> &'static RwLock<HashMap<&'static str, TokenKind>> {
    KEYWORDS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("and", TokenKind::And);
        m.insert("or", TokenKind::Or);
        m.insert("class", TokenKind::Class);
        m.insert("else", TokenKind::Else);
        m.insert("false", TokenKind::False);
        m.insert("for", TokenKind::For);
        m.insert("fun", TokenKind::Fun);
        m.insert("if", TokenKind::If);
        m.insert("nil", TokenKind::Nil);
        m.insert("print", TokenKind::Print);
        m.insert("return", TokenKind::Return);
        m.insert("super", TokenKind::Super);
        m.insert("this", TokenKind::This);
        m.insert("true", TokenKind::True);
        m.insert("var", TokenKind::Var);
        m.insert("while", TokenKind::While);

        RwLock::new(m)
    })
}

#[derive(Display, Debug, Clone, Copy)]
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
    Semicolon,  // "
    Slash,      // /
    Star,       // *

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

    EOF,
}

#[derive(Debug, Display)]
pub(crate) enum Literal {
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub(crate) struct Token {
    token_type: TokenKind,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
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
            write!(f, "{} {} {}", self.token_type, self.lexeme, literal)
        } else {
            write!(f, "{} {}", self.token_type, self.lexeme)
        }
    }
}
