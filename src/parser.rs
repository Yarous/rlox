use crate::expr::{Expr, Op};
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenKind};

#[derive(Debug)]
pub(crate) struct Parser<'tokens> {
    tokens: &'tokens [Token],
    current: usize,
    errors: Vec<ParseError<'tokens>>,
    loop_depth: usize,
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
            loop_depth: 0,
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
            return self.if_statement();
        } else if self.switch(&[TokenKind::While]) {
            return self.while_statement();
        } else if self.switch(&[TokenKind::For]) {
            return self.for_statement();
        } else if self.switch(&[TokenKind::Break]) {
            return self.break_statement();
        } else if self.switch(&[TokenKind::Fun]) {
            return self.function_statement();
        } else if self.switch(&[TokenKind::Return]) {
            return self.return_statement();
        }
        self.expression_statement()
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let keyword = self.previous().clone();
        let mut value: Option<Expr> = None;

        if !self.check(TokenKind::Semicolon) {
            value = Some(self.expression()?);
        }

        self.consume(TokenKind::Semicolon, "Expect ';' after return value")?;
        Ok(Stmt::Return { keyword, value })
    }

    fn function_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let name = self
            .consume(TokenKind::Identifier, "Expect function name!")?
            .clone();
        self.consume(TokenKind::LeftParen, "Expect '(' after function name!")?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenKind::RightParen) {
            if parameters.len() >= 255 {
                self.error("Cant have more than 255 parameters.");
            }

            parameters.push(
                self.consume(TokenKind::Identifier, "Expected parameter name")?
                    .clone(),
            );
            while self.switch(&[TokenKind::Comma]) {
                if parameters.len() >= 255 {
                    self.error("Cant have more than 255 parameters.");
                }

                parameters.push(
                    self.consume(TokenKind::Identifier, "Expected parameter name")?
                        .clone(),
                )
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    fn break_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        if self.loop_depth == 0 {
            return Err(self.error("Can't use 'break' outside of a loop."));
        }
        self.consume(TokenKind::Semicolon, "Expect ';' after break.")?;
        Ok(Stmt::Break)
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'")?;

        let initializer: Option<Stmt>;
        if self.switch(&[TokenKind::Semicolon]) {
            initializer = None;
        } else if self.switch(&[TokenKind::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenKind::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenKind::Semicolon, "Expect ';' after loop condition")?;
        let mut increment: Option<Expr> = None;
        if !self.check(TokenKind::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenKind::RightParen, "Expect ')' after loop condition.")?;

        self.loop_depth += 1;
        let body = self.statement();
        self.loop_depth -= 1;
        let mut body = body?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::ExprStmt(increment)]);
        }
        if condition.is_none() {
            condition = Some(Expr::Literal(Literal::Boolean(true)))
        }
        body = Stmt::While {
            condition: condition.unwrap(),
            body: Box::new(body),
        };
        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenKind::RightParen, "Expect ')' after 'while'")?;

        self.loop_depth += 1;
        let body = self.statement();
        self.loop_depth -= 1;
        let body = body?;
        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
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

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value")?;

        Ok(Stmt::Print(expr))
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
        let mut expr = self.or()?;

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

    fn or(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let mut expr = self.and()?;

        while self.switch(&[TokenKind::Or]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.and()?);

            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let mut expr = self.equality()?;

        while self.switch(&[TokenKind::And]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.equality()?);

            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let mut expr = self.primary()?;

        loop {
            if self.switch(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError<'tokens>> {
        let mut arguments = Vec::new();
        if !self.check(TokenKind::RightParen) {
            if arguments.len() >= 255 {
                return Err(self.error("cant have more than 255 arguments"));
            }

            arguments.push(self.assignment()?);
            while self.switch(&[TokenKind::Comma]) {
                if arguments.len() >= 255 {
                    return Err(self.error("cant have more than 255 arguments"));
                }
                arguments.push(self.assignment()?);
            }
        }

        let paren = self
            .consume(TokenKind::RightParen, "Expect ')' after arguments.")?
            .clone();

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
