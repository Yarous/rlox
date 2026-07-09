use crate::ast::{Expr, Stmt};
use crate::frontend::parser::{ParseError, Parser};
use crate::frontend::token::{Literal, TokenKind};

impl<'tokens> Parser<'tokens> {
    pub(super) fn block(&mut self) -> Result<Vec<Stmt>, ParseError<'tokens>> {
        let mut statements = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt)
            }
        }

        self.consume(TokenKind::RightBrace, "Expect } after block.")?;
        Ok(statements)
    }

    pub(super) fn statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
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

    pub(super) fn return_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        if self.function_depth != 0 {
            let keyword = self.previous().clone();
            let mut value: Option<Expr> = None;

            if !self.check(TokenKind::Semicolon) {
                value = Some(self.expression()?);
            }

            self.consume(TokenKind::Semicolon, "Expect ';' after return value")?;
            return Ok(Stmt::Return { keyword, value });
        }

        Err(self.error("return statement only allowed in functions/lambda expressions"))
    }

    pub(super) fn break_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        if self.loop_depth == 0 {
            return Err(self.error("Can't use 'break' outside of a loop."));
        }
        self.consume(TokenKind::Semicolon, "Expect ';' after break.")?;
        Ok(Stmt::Break)
    }

    pub(super) fn for_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
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
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
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

    pub(super) fn while_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
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

    pub(super) fn if_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
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

    pub(super) fn print_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after value")?;

        Ok(Stmt::Print(expr))
    }

    pub(super) fn expression_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expect ';' after expression")?;

        Ok(Stmt::Expression(expr))
    }
}
