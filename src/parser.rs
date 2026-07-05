use crate::expr::{Expr, Op};
use crate::stmt::Stmt;
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
    pub(crate) statements: Option<Vec<Stmt>>,
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

impl<'tokens> Parser<'tokens> {
    pub(crate) fn new(tokens: &'tokens [Token]) -> Self {
        Self {
            tokens,
            current: 0,
            errors: vec![],
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

    // grammar-rules

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.switch(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let name = self
            .consume(TokenKind::Identifier, "Expect variable name!")?
            .clone();

        let mut initializer: Option<Expr> = None;
        if self.switch(&[TokenKind::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError<'tokens>> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt)
            }
        }

        self.consume(TokenKind::RightBrace, "Expect } after block.")?;
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        if self.switch(&[TokenKind::Print]) {
            return self.print_statement();
        } else if self.switch(&[TokenKind::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        } else if self.switch(&[TokenKind::If]) {
            return self.if_statement()
        }
        self.expression_statement()
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after 'if'")?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch: Option<Box<Stmt>> = None;
        if self.switch(&[TokenKind::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        };

        Ok(Stmt::IfStmt {
            condition,
            then_branch,
            else_branch
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value")?;

        Ok(Stmt::PrintStmt(expr))
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression")?;

        Ok(Stmt::ExprStmt(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError<'tokens>> {
        self.comma()
    }

    fn comma(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let mut expr = self.assignment()?;

        while self.switch(&[TokenKind::Comma]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.assignment()?);

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let expr = self.ternary()?;

        if self.switch(&[TokenKind::Equal]) {
            let value = self.assignment()?;

            return if let Expr::Variable(name) = expr {
                Ok(Expr::Assignment {
                    name,
                    value: Box::new(value),
                })
            } else {
                Err(self.error("Invalid assignment target!"))
            };
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    fn equality(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    fn comparison(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    fn term(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    fn factor(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    fn unary(&mut self) -> Result<Expr, ParseError<'tokens>> {
        if self.switch(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.unary()?);

            return Ok(Expr::Unary { operator, right });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError<'tokens>> {
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
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.switch(&[TokenKind::Identifier]) {
            Ok(Expr::Variable(self.previous().clone()))
        } else {
            Err(self.error("Expect expression."))
        }
    }
}
