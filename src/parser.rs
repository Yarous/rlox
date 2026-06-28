use crate::Lox;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenKind};

pub(crate) struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

enum ParseError {
    SomeErr(String),
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Parser {
            tokens,
            current: 0,
            lox,
        }
    }

    pub(crate) fn parse(&mut self) -> Expr {
        self.expression()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenKind::EOF
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

    fn consume(&mut self, token_kind: TokenKind, error_message: &str) -> Token {
        if self.check(token_kind) {
            return self.advance();
        }

        let ParseError::SomeErr(err_message) = self.error(self.peek(), error_message);
        panic!("{}", err_message);
    }
    fn error(&mut self, token: Token, error_message: &str) -> ParseError {
        self.lox.error(token, error_message);
        ParseError::SomeErr(error_message.to_string())
    }

    #[allow(unused)]
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

    // grammar-rules

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.switch(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = Box::from(self.comparison());

            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            };
        }
        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.switch(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = Box::from(self.term());

            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            }
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.switch(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous();
            let right = Box::from(self.factor());

            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.switch(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous();
            let right = Box::from(self.unary());

            expr = Expr::Binary {
                left: Box::from(expr),
                operator,
                right,
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.switch(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = Box::from(self.unary());

            return Expr::Unary { operator, right };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.switch(&[TokenKind::False]) {
            return Expr::Literal(Literal::False);
        }
        if self.switch(&[TokenKind::True]) {
            return Expr::Literal(Literal::True);
        }
        if self.switch(&[TokenKind::Nil]) {
            return Expr::Literal(Literal::Nil);
        }

        if self.switch(&[TokenKind::Number, TokenKind::String]) {
            return Expr::Literal(self.previous().literal.unwrap()); // safety: if this blocks runs that means that previous token is 100% literal
        }

        if self.switch(&[TokenKind::LeftParen]) {
            let expr = self.expression();

            self.consume(TokenKind::RightParen, "Expect ')' after expression");
            Expr::Grouping(Box::from(expr))
        } else {
            let ParseError::SomeErr(err_message) = self.error(self.peek(), "Expect expression.");
            panic!("{}", err_message);
        }
    }
}
