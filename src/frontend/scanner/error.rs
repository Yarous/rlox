use crate::frontend::token::Token;

pub(crate) struct ScanError {
    pub(crate) message: &'static str,
    pub(crate) line: usize,
}

pub(crate) struct ScanResult {
    pub(crate) tokens: Vec<Token>,
    pub(crate) errors: Vec<ScanError>,
}

impl ScanResult {
    pub(super) fn new(tokens: Vec<Token>, errors: Vec<ScanError>) -> Self {
        Self { tokens, errors }
    }
}

impl ScanError {
    pub(super) fn new(message: &'static str, _problem_byte: u8, line: usize) -> Self {
        Self { message, line }
    }
}
