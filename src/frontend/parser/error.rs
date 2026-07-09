use crate::ast::Stmt;
use crate::frontend::token::Token;

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
    pub(super) fn new(token: &'token Token, error_message: &'static str) -> Self {
        Self {
            token,
            message: error_message,
        }
    }
}
