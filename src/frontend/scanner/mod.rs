use crate::frontend::token::{Literal, Token, TokenKind};

mod error;

pub(crate) use error::{ScanError, ScanResult};

pub(crate) struct Scanner<'src> {
    source: &'src str,
    bytes: &'src [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'src> Scanner<'src> {
    #[must_use]
    pub(crate) fn new(source: &'src str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            tokens: Vec::with_capacity(source.len() / 4 + 8),
            start: 0,
            current: 0,
            line: 1,
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
            "break" => TokenKind::Break,
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

    fn add_one_or_two_token(
        &mut self,
        expected: u8,
        one: TokenKind,
        two: TokenKind,
    ) -> Result<(), ScanError> {
        let kind = if self.match_bytes(expected) { two } else { one };
        self.add_token(kind);
        Ok(())
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

    fn peek_previous(&self) -> u8 {
        self.bytes.get(self.current - 1).copied().unwrap_or(b'\0')
    }

    fn peek_next(&self) -> u8 {
        self.bytes.get(self.current + 1).copied().unwrap_or(b'\0')
    }

    #[inline]
    fn string(&mut self) -> Result<(), ScanError> {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(ScanError::new(
                "Unterminated string",
                self.peek(),
                self.line,
            ));
        }

        self.advance();
        let string = self.source[self.start + 1..self.current - 1].to_string();
        let string = Literal::String(string);
        self.add_token_with_literal(TokenKind::String, Some(string));
        Ok(())
    }
    #[inline]
    fn number(&mut self) -> Result<(), ScanError> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let number = match self.source[self.start..self.current].parse::<f64>() {
            Ok(num) => num,
            Err(_) => {
                return Err(ScanError::new(
                    "Scanner only accepts valid Lox number syntax",
                    self.peek(),
                    self.line,
                ));
            }
        };
        self.add_token_with_literal(TokenKind::Number, Some(Literal::Number(number)));
        Ok(())
    }

    fn identifier(&mut self) {
        while self.is_alphanumerical(self.peek() as char) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_kind = self.keyword_kind(text);

        self.add_token(token_kind);
    }

    fn block_comment(&mut self) -> Result<(), ScanError> {
        while !self.is_at_end() {
            match self.advance() {
                b'\n' => self.line += 1,
                b'*' if self.peek() == b'/' => {
                    self.advance();
                    return Ok(());
                }
                _ => (),
            }
        }
        Err(ScanError::new(
            "Unterminated block comment",
            self.peek(),
            self.line,
        ))
    }

    fn scan_token(&mut self) -> Result<(), ScanError> {
        let c = self.advance();

        match c {
            b'(' => {
                self.add_token(TokenKind::LeftParen);
                Ok(())
            }
            b')' => {
                self.add_token(TokenKind::RightParen);
                Ok(())
            }
            b'{' => {
                self.add_token(TokenKind::LeftBrace);
                Ok(())
            }
            b'}' => {
                self.add_token(TokenKind::RightBrace);
                Ok(())
            }
            b',' => {
                self.add_token(TokenKind::Comma);
                Ok(())
            }
            b'.' => {
                self.add_token(TokenKind::Dot);
                Ok(())
            }
            b'-' => {
                self.add_token(TokenKind::Minus);
                Ok(())
            }
            b'+' => {
                self.add_token(TokenKind::Plus);
                Ok(())
            }
            b'?' => {
                self.add_token(TokenKind::Question);
                Ok(())
            }
            b';' => {
                self.add_token(TokenKind::Semicolon);
                Ok(())
            }
            b':' => {
                self.add_token(TokenKind::Colon);
                Ok(())
            }
            b'*' => {
                self.add_token(TokenKind::Star);
                Ok(())
            }
            b'!' => self.add_one_or_two_token(b'=', TokenKind::Bang, TokenKind::BangEqual),
            b'=' => self.add_one_or_two_token(b'=', TokenKind::Equal, TokenKind::EqualEqual),
            b'>' => self.add_one_or_two_token(b'=', TokenKind::Greater, TokenKind::GreaterEqual),
            b'<' => self.add_one_or_two_token(b'=', TokenKind::Less, TokenKind::LessEqual),
            b'/' => {
                if self.match_bytes(b'*') {
                    self.block_comment()
                } else if self.match_bytes(b'/') {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(())
                } else {
                    self.add_token(TokenKind::Slash);
                    Ok(())
                }
            }
            b'"' => self.string(),
            b'0'..=b'9' => self.number(),
            b' ' => Ok(()),
            b'\t' => Ok(()),
            b'\r' => Ok(()),
            b'\n' => {
                self.line += 1;
                Ok(())
            }
            c => {
                if self.is_alphabetical(c as char) {
                    self.identifier();
                    Ok(())
                } else {
                    Err(ScanError::new(
                        "Failed to parse lexeme as token",
                        self.peek_previous(),
                        self.line,
                    ))
                }
            }
        }
    }

    #[must_use]
    pub fn scan_tokens(&mut self) -> ScanResult {
        let mut errors: Vec<ScanError> = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(()) => (),
                Err(error) => errors.push(error),
            }
        }

        self.tokens
            .push(Token::new(TokenKind::Eof, "".into(), None, self.line));
        ScanResult::new(std::mem::take(&mut self.tokens), errors)
    }
}

#[cfg(test)]
mod tests;
