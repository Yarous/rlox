use crate::ast::{Expr, Stmt};
use crate::frontend::parser::{ParseError, Parser};
use crate::frontend::token::{Token, TokenKind};

impl<'tokens> Parser<'tokens> {
    pub(super) fn parse_function_parameters(&mut self) -> Result<Vec<Token>, ParseError<'tokens>> {
        self.consume(TokenKind::LeftParen, "Expect '(' before parameters")?;
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
                );
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters.")?;

        Ok(parameters)
    }

    pub(super) fn declaration(&mut self) -> Option<Stmt> {
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

    pub(super) fn var_declaration(&mut self) -> Result<Stmt, ParseError<'tokens>> {
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

    pub(super) fn function_statement(&mut self) -> Result<Stmt, ParseError<'tokens>> {
        let name = self
            .consume(TokenKind::Identifier, "Expect function name!")?
            .clone();

        let parameters = self.parse_function_parameters()?;

        self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;
        self.function_depth += 1;
        let body = self.block()?;
        self.function_depth -= 1;
        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }
}
