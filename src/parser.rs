use crate::expr::{Expr, Op};
use crate::token::{Literal, Token, TokenKind};

pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    errors: Vec<ParseError<'tokens>>,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ParseError<'token> {
    pub(crate) token: &'token Token,
    pub(crate) message: &'static str,
}

pub(crate) struct ParseResult<'token> {
    pub(crate) expr: Option<Expr>,
    pub(crate) errors: Vec<ParseError<'token>>,
}

impl<'token> ParseError<'token> {
    fn new(token: &'token Token, error_message: &'static str) -> Self {
        Self {
            token,
            message: error_message,
        }
    }
}

type Result<'token> = std::result::Result<Expr, ParseError<'token>>;

impl<'tokens> Parser<'tokens> {
    pub(crate) fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            errors: vec![],
        }
    }

    pub(crate) fn parse(&mut self) -> ParseResult<'tokens> {
        let expr = match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => {
                self.synchronize();
                None
            }
        };
        ParseResult {
            expr,
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
    ) -> std::result::Result<&Token, ParseError<'tokens>> {
        if self.check(token_kind) {
            return Ok(self.advance());
        }
        Err(self.error(error_message))
    }

    fn error(&mut self, error_message: &'static str) -> ParseError<'tokens> {
        let error = ParseError::new(self.peek(), error_message);
        self.errors.push(error);
        error
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

    fn expression(&mut self) -> Result<'tokens> {
        self.comma()
    }

    fn comma(&mut self) -> Result<'tokens> {
        let mut expr = self.ternary()?;

        while self.switch(&[TokenKind::Comma]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.ternary()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<'tokens> {
        let mut expr = self.equality()?;

        if self.switch(&[TokenKind::Question]) {
            let then_expr = Box::new(self.ternary()?);
            self.consume(TokenKind::Colon, "Expected ':'.")?;
            let else_expr = Box::new(self.ternary()?);

            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_expr,
                else_expr,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<'tokens> {
        let mut expr = self.comparison()?;

        while self.switch(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.comparison()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<'tokens> {
        let mut expr = self.term()?;

        while self.switch(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.term()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<'tokens> {
        let mut expr = self.factor()?;

        while self.switch(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.factor()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<'tokens> {
        let mut expr = self.unary()?;

        while self.switch(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.unary()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<'tokens> {
        if self.switch(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.unary()?);

            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<'tokens> {
        if self.switch(&[
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::EqualEqual,
            TokenKind::BangEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Comma,
        ]) {
            let operator = self.previous().token_type;
            self.error("Missing left operand");

            return match operator {
                TokenKind::Slash | TokenKind::Star => self.factor(),
                TokenKind::Plus | TokenKind::Minus => self.term(),
                TokenKind::Greater
                | TokenKind::GreaterEqual
                | TokenKind::Less
                | TokenKind::LessEqual => self.comparison(),
                TokenKind::EqualEqual | TokenKind::BangEqual => self.equality(),
                TokenKind::Comma => self.comma(),
                _ => self.equality(),
            };
        }

        if self.switch(&[TokenKind::False]) {
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }
        if self.switch(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }
        if self.switch(&[TokenKind::Nil]) {
            return Ok(Expr::Literal(Literal::Nil));
        }

        if self.switch(&[TokenKind::Number, TokenKind::String]) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap())); // safety: if this blocks runs that means that previous token is 100% literal
        }

        if self.switch(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenKind::RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self.error("Expect expression."))
        }
    }
}
