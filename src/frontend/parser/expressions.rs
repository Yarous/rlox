use crate::ast::{Expr, Op};
use crate::frontend::parser::{ParseError, Parser};
use crate::frontend::token::{Literal, TokenKind};

impl<'tokens> Parser<'tokens> {
    pub(super) fn expression(&mut self) -> Result<Expr, ParseError<'tokens>> {
        self.comma()
    }

    pub(super) fn comma(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn assignment(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn ternary(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn or(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn and(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn equality(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn comparison(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn term(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn factor(&mut self) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn unary(&mut self) -> Result<Expr, ParseError<'tokens>> {
        if self.switch(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = Op::from(self.previous());
            let right = Box::new(self.unary()?);

            return Ok(Expr::Unary { operator, right });
        }

        self.call()
    }

    pub(super) fn call(&mut self) -> Result<Expr, ParseError<'tokens>> {
        let mut expr = self.primary()?;

        loop {
            if self.switch(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break Ok(expr);
            }
        }
    }

    pub(super) fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError<'tokens>> {
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

    pub(super) fn primary(&mut self) -> Result<Expr, ParseError<'tokens>> {
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
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
        }

        if self.switch(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenKind::RightParen, "Expect ')' after expression")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.switch(&[TokenKind::Fun]) {
            let parameters = self.parse_function_parameters()?;

            self.consume(TokenKind::LeftBrace, "Expect '{' before function body.")?;
            self.function_depth += 1;
            let body = self.block()?;
            self.function_depth -= 1;

            return Ok(Expr::Lambda { parameters, body });
        }

        if self.switch(&[TokenKind::Identifier]) {
            Ok(Expr::Variable(self.previous().clone()))
        } else {
            Err(self.error("Expect expression."))
        }
    }
}
