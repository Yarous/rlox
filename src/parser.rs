use crate::Lox;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenKind};

pub(crate) struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

#[derive(Debug)]
enum ParseError {
    SomeErr(String),
}

type Result = std::result::Result<Expr, ParseError>;

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Parser {
            tokens,
            current: 0,
            lox,
        }
    }

    pub(crate) fn parse(&mut self) -> Expr {
        self.expression().unwrap() // TODO: in the next chapter I will remove this unwrap, trust me =)
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

    fn consume(
        &mut self,
        token_kind: TokenKind,
        error_message: &str,
    ) -> std::result::Result<Token, ParseError> {
        if self.check(token_kind) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), error_message))
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

    fn expression(&mut self) -> Result {
        self.comma()
    }

    fn comma(&mut self) -> Result {
        let mut expr = self.ternary()?;

        while self.switch(&[TokenKind::Comma]) {
            let operator = self.previous();
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
            let operator = self.previous();
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
            let operator = self.previous();
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
            let operator = self.previous();
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
            let operator = self.previous();
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
            let operator = self.previous();
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
            self.error(operator.clone(), "Missing left operand");

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
            return Ok(Expr::Literal(self.previous().literal.unwrap())); // safety: if this blocks runs that means that previous token is 100% literal
        }

        if self.switch(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenKind::RightParen, "Expect ')' after expression")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self.error(self.peek(), "Expect expression."))
        }
    }
}
