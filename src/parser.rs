use crate::Lox;
use crate::expr::{Expr, Op};
use crate::token::{Literal, Token, TokenKind};

pub(crate) struct Parser<'tokens, 'lox> {
    tokens: &'tokens [Token],
    current: usize,
    lox: &'lox mut Lox,
}

#[derive(Debug)]
struct ParseError {
    token_type: TokenKind,
    lexeme: String,
    line: usize,
    message: &'static str,
}

impl ParseError {
    fn new(token: &Token, error_message: &'static str) -> Self {
        Self {
            token_type: token.token_type,
            lexeme: token.lexeme.clone(),
            line: token.line,
            message: error_message,
        }
    }
}

type Result = std::result::Result<Expr, ParseError>;

impl<'tokens, 'lox> Parser<'tokens, 'lox> {
    pub(crate) fn new(tokens: &'tokens [Token], lox: &'lox mut Lox) -> Self {
        Parser {
            tokens,
            current: 0,
            lox,
        }
    }

    pub(crate) fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
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
    ) -> std::result::Result<&Token, ParseError> {
        if self.check(token_kind) {
            return Ok(self.advance());
        }
        Err(self.error(error_message))
    }

    fn error(&self, error_message: &'static str) -> ParseError {
        let token = self.peek();
        ParseError::new(token, error_message)
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

    fn expression(&mut self) -> Result {
        self.comma()
    }

    fn comma(&mut self) -> Result {
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

    fn ternary(&mut self) -> Result {
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

    fn equality(&mut self) -> Result {
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

    fn comparison(&mut self) -> Result {
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

    fn term(&mut self) -> Result {
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

    fn factor(&mut self) -> Result {
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

    fn unary(&mut self) -> Result {
        if self.switch(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.unary()?);

            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result {
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
            let operator = self.previous();
            self.error("Missing left operand");

            return match operator.token_type {
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
            return Ok(Expr::Literal(Literal::False));
        }
        if self.switch(&[TokenKind::True]) {
            return Ok(Expr::Literal(Literal::True));
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
