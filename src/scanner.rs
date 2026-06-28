use crate::Lox;
use crate::token::{Literal, Token, TokenKind, get_keywords_map};

pub(crate) struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    lox: &'a mut Lox,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(source: String, lox: &'a mut Lox) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            lox,
        }
    }

    fn add_token(&mut self, token_kind: TokenKind) {
        self.add_token_with_literal(token_kind, None)
    }

    fn add_token_with_literal(&mut self, token_kind: TokenKind, literal: Option<Literal>) {
        self.tokens.push(Token::new(
            token_kind,
            self.source[self.start..self.current].to_string(),
            literal,
            self.line,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    #[inline]
    fn switch(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        };
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        };
        self.advance();
        true
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        };
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    #[inline]
    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.lox.report_error(self.line, "", "Unterminated string.");
            return;
        }

        self.advance();
        let string = self.source[self.start + 1..self.current].to_string();
        let string = Literal::String(string);
        self.add_token_with_literal(TokenKind::String, Some(string));
    }
    #[inline]
    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let number = self.source[self.start..self.current].to_string();
        let number = Literal::Number(number.parse::<f64>().unwrap());
        self.add_token_with_literal(TokenKind::Number, Some(number));
    }

    fn identifier(&mut self) {
        let map = get_keywords_map().read().unwrap();

        while self.peek().is_alphanumeric() {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_kind = match map.get(&text) {
            Some(kind) => *kind,
            None => TokenKind::Identifier,
        };

        self.add_token(token_kind);
    }

    fn scan_token(&mut self) {
        let c = self.advance();

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
            '/' => {
                if self.switch('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        // TODO: we can just set the pointer on the end of the line
                        self.advance();
                    }
                } else if self.switch('*') {
                    while self.peek() != '*' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            '"' => self.string(),
            '0'..='9' => self.number(),
            ' ' => (),
            '\t' => (),
            '\r' => (),
            '\n' => {
                self.line += 1;
            }
            c => {
                if c.is_alphabetic() {
                    self.identifier();
                } else {
                    self.lox
                        .report_error(self.line, "", "Failed to parse lexeme as token");
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenKind::EOF, "".into(), None, self.line));
        &self.tokens
    }
}
