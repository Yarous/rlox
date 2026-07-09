use crate::token::{Literal, Token, TokenKind};

pub(crate) struct Scanner<'src> {
    source: &'src str,
    bytes: &'src [u8],
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

pub(crate) struct ScanError {
    pub(crate) message: &'static str,
    pub(crate) line: usize,
}

pub(crate) struct ScanResult {
    pub(crate) tokens: Vec<Token>,
    pub(crate) errors: Vec<ScanError>,
}

impl ScanResult {
    fn new(tokens: Vec<Token>, errors: Vec<ScanError>) -> Self {
        Self { tokens, errors }
    }
}

impl ScanError {
    fn new(message: &'static str, _problem_byte: u8, line: usize) -> Self {
        Self { message, line }
    }
}

impl<'src> Scanner<'src> {
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

    pub fn scan_tokens(&mut self) -> ScanResult {
        let mut errors: Vec<ScanError> = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(error) => errors.push(error),
            }
        }

        self.tokens
            .push(Token::new(TokenKind::Eof, "".into(), None, self.line));
        ScanResult::new(std::mem::take(&mut self.tokens), errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(source: &str) -> ScanResult {
        Scanner::new(source).scan_tokens()
    }

    fn kinds(source: &str) -> Vec<TokenKind> {
        scan(source)
            .tokens
            .into_iter()
            .map(|token| token.token_type)
            .collect()
    }

    #[test]
    fn scans_single_character_tokens() {
        assert_eq!(
            kinds("(){},.-+;:*?"),
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Minus,
                TokenKind::Plus,
                TokenKind::Semicolon,
                TokenKind::Colon,
                TokenKind::Star,
                TokenKind::Question,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_one_or_two_character_tokens() {
        assert_eq!(
            kinds("! != = == > >= < <= /"),
            vec![
                TokenKind::Bang,
                TokenKind::BangEqual,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Slash,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_keywords_and_identifiers() {
        assert_eq!(
            kinds(
                "and class else false fun for if nil or print return super this true var while break alpha _x x9"
            ),
            vec![
                TokenKind::And,
                TokenKind::Class,
                TokenKind::Else,
                TokenKind::False,
                TokenKind::Fun,
                TokenKind::For,
                TokenKind::If,
                TokenKind::Nil,
                TokenKind::Or,
                TokenKind::Print,
                TokenKind::Return,
                TokenKind::Super,
                TokenKind::This,
                TokenKind::True,
                TokenKind::Var,
                TokenKind::While,
                TokenKind::Break,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn scans_number_literals() {
        let result = scan("123 45.67");

        assert!(result.errors.is_empty());
        assert_eq!(result.tokens[0].literal, Some(Literal::Number(123.0)));
        assert_eq!(result.tokens[1].literal, Some(Literal::Number(45.67)));
    }

    #[test]
    fn scans_string_literals() {
        let result = scan("\"lox\" \"line\\nnext\"");

        assert!(result.errors.is_empty());
        assert_eq!(
            result.tokens[0].literal,
            Some(Literal::String("lox".to_string()))
        );
        assert_eq!(
            result.tokens[1].literal,
            Some(Literal::String("line\\nnext".to_string()))
        );
    }

    #[test]
    fn ignores_line_comments() {
        assert_eq!(
            kinds("print 1; // ignored\nprint 2;"),
            vec![
                TokenKind::Print,
                TokenKind::Number,
                TokenKind::Semicolon,
                TokenKind::Print,
                TokenKind::Number,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn ignores_block_comments_and_tracks_lines() {
        let result = scan("print 1; /* a\nb\nc */ print 2;");

        assert!(result.errors.is_empty());
        assert_eq!(result.tokens[3].token_type, TokenKind::Print);
        assert_eq!(result.tokens[3].line, 3);
    }

    #[test]
    fn reports_unterminated_string() {
        let result = scan("\"unterminated");

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].message, "Unterminated string");
        assert_eq!(result.tokens.last().unwrap().token_type, TokenKind::Eof);
    }

    #[test]
    fn reports_unterminated_block_comment() {
        let result = scan("/* nope");

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].message, "Unterminated block comment");
    }

    #[test]
    fn reports_unexpected_character_and_continues() {
        let result = scan("@ print 1;");

        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].message, "Failed to parse lexeme as token");
        assert_eq!(
            result
                .tokens
                .into_iter()
                .map(|token| token.token_type)
                .collect::<Vec<_>>(),
            vec![
                TokenKind::Print,
                TokenKind::Number,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }
}
