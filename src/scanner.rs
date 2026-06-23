use crate::Lox;
use crate::token::{Token, TokenKind};

struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn add_token(&mut self, token_kind: TokenKind) {
        self.tokens.push(Token::new(
            token_kind,
            self.source
                .chars()
                .skip(self.start)
                .take(self.current - self.start)
                .collect(),
            self.start,
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    #[inline]
    fn switch(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            false
        } else {
            if self.source.chars().nth(self.current + 1).unwrap() != expected {
                return false;
            };
            self.current += 1;
            true
        }
    }

    fn scan_token(&mut self, lox: &mut Lox) {
        let c: char = self.source.chars().nth(self.current).unwrap();

        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                if self.switch('=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.switch('=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.switch('=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.switch('=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            _ => {
                lox.report_error(self.line, "Failed to parse lexeme as token");
            }
        }
    }

    pub fn scan_tokens(&mut self, lox: &mut Lox) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token(lox)
        }

        self.tokens.push(Token::new(TokenKind::EOF, "".into(), 0, self.line));
    }
}
