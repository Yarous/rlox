use crate::ast::Stmt;
use crate::frontend::token::{Token, TokenKind};

mod declarations;
mod error;
mod expressions;
mod statements;

pub(crate) use error::{ParseError, ParseResult};

#[derive(Debug)]
pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    errors: Vec<ParseError<'tokens>>,
    loop_depth: usize,
    function_depth: usize,
}

impl<'tokens> Parser<'tokens> {
    pub(crate) fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            errors: vec![],
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub(crate) fn parse(&mut self) -> ParseResult<'tokens> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt)
            }
        }

        ParseResult {
            statements: Some(statements), // TODO: it could be no statements here =) FIX IT LATER!!!
            errors: std::mem::take(&mut self.errors),
        }
    }

    fn previous(&self) -> &'tokens Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &'tokens Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &'tokens Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenKind::Eof
    }

    fn check(&self, token_type: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn switch(&mut self, types: &[TokenKind]) -> bool {
        for &token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(
        &mut self,
        token_kind: TokenKind,
        error_message: &'static str,
    ) -> Result<&Token, ParseError<'tokens>> {
        if self.check(token_kind) {
            return Ok(self.advance());
        }
        Err(self.error(error_message))
    }

    fn error(&mut self, error_message: &'static str) -> ParseError<'tokens> {
        let error = ParseError::new(self.peek(), error_message);
        self.errors.push(error);
        self.advance();
        error
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenKind::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => {
                    return;
                }
                _ => (),
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests;
