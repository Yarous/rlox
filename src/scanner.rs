use crate::token::{Literal, Token, TokenKind};
use crate::Lox;

pub(crate) struct Scanner<'src, 'lox> {
    source: &'src str,
    bytes: &'src [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    lox: &'lox mut Lox,
}

impl<'src, 'lox> Scanner<'src, 'lox> {
    pub(crate) fn new(source: &'src str, lox: &'lox mut Lox) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
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

    fn keyword_kind(&self, text: &str) -> TokenKind {
        match text {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.bytes.len()
    }

    fn is_alphabetical(&self, c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_alphanumerical(&self, c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    }

    #[inline]
    fn match_bytes(&mut self, expected: u8) -> bool {
        if self.peek() != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> u8 {
        let byte = self.bytes[self.current];
        self.current += 1;
        byte
    }

    fn peek(&self) -> u8 {
        self.bytes.get(self.current).copied().unwrap_or(b'\0')
    }

    fn peek_next(&self) -> u8 {
        self.bytes.get(self.current + 1).copied().unwrap_or(b'\0')
    }

    #[inline]
    fn string(&mut self) {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.lox.report_error(self.line, "", "Unterminated string.");
            return;
        }

        self.advance();
        let string = self.source[self.start + 1..self.current - 1].to_string();
        let string = Literal::String(string);
        self.add_token_with_literal(TokenKind::String, Some(string));
    }
    #[inline]
    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
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
        while self.is_alphanumerical(self.peek() as char) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_kind = self.keyword_kind(text);

        self.add_token(token_kind);
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            b'(' => self.add_token(TokenKind::LeftParen),
            b')' => self.add_token(TokenKind::RightParen),
            b'{' => self.add_token(TokenKind::LeftBrace),
            b'}' => self.add_token(TokenKind::RightBrace),
            b',' => self.add_token(TokenKind::Comma),
            b'.' => self.add_token(TokenKind::Dot),
            b'-' => self.add_token(TokenKind::Minus),
            b'+' => self.add_token(TokenKind::Plus),
            b'?' => self.add_token(TokenKind::Question),
            b';' => self.add_token(TokenKind::Semicolon),
            b':' => self.add_token(TokenKind::Colon),
            b'*' => self.add_token(TokenKind::Star),
            b'!' => {
                if self.match_bytes(b'=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            b'=' => {
                if self.match_bytes(b'=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            b'<' => {
                if self.match_bytes(b'=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            b'>' => {
                if self.match_bytes(b'=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            b'/' => {
                if self.match_bytes(b'/') {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        // TODO: we can just set the pointer on the end of the line
                        self.advance();
                    }
                } else if self.match_bytes(b'*') {
                    while self.peek() != b'*' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            b'"' => self.string(),
            b'0'..=b'9' => self.number(),
            b' ' => (),
            b'\t' => (),
            b'\r' => (),
            b'\n' => {
                self.line += 1;
            }
            c => {
                if self.is_alphabetical(c as char) {
                    dbg!(c as char);
                    self.identifier();
                } else {
                    self.lox
                        .report_error(self.line, "", "Failed to parse lexeme as token");
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenKind::Eof, "".into(), None, self.line));
        std::mem::take(&mut self.tokens)
    }
}
