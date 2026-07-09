use crate::frontend::token::{Token, TokenKind};
use crate::runtime::RuntimeError;

#[derive(Debug)]
pub(crate) struct Lox {
    pub(crate) had_error: bool,
    pub(crate) had_runtime_error: bool,
}

impl Lox {
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub(crate) fn report_error(&mut self, line: usize, where_err: &str, message: &str) {
        self.had_error = true;
        eprintln!("[line {line}] Error {where_err}: {message}");
    }

    pub(crate) fn error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenKind::Eof {
            self.report_error(token.line, "at end", message);
        } else {
            self.report_error(
                token.line,
                format!("at '{}'", token.lexeme).as_str(),
                message,
            );
        }
    }

    pub(crate) fn runtime_error(&mut self, error: &RuntimeError) {
        let line = error.line;
        let error_type = &error.error_type;
        eprintln!("[line {line}] Runtime error: {error_type}");

        self.had_runtime_error = true;
    }
}
