use crate::app::lox::Lox;
use crate::frontend::parser::Parser;
use crate::frontend::scanner::Scanner;
use crate::runtime::Interpreter;
use crate::runtime::control_flow::ControlFlow;

pub(crate) fn run_source(lox: &mut Lox, source: &str) {
    let mut scanner = Scanner::new(source);
    let scan_result = scanner.scan_tokens();
    if !scan_result.errors.is_empty() {
        for error in scan_result.errors {
            lox.report_error(error.line, "", error.message);
        }
        return;
    }

    let tokens = scan_result.tokens;
    let mut parser = Parser::new(&tokens);
    let parse_result = parser.parse();
    if !parse_result.errors.is_empty() {
        for error in parse_result.errors {
            lox.error(error.token, error.message);
        }
        return;
    }

    let mut interpreter = Interpreter::new(lox);
    match interpreter.interpret(&parse_result.statements.unwrap()) {
        Ok(_) => (),
        Err(error) => {
            if let ControlFlow::RuntimeError(reported_error) = error {
                lox.runtime_error(&reported_error);

                return
            }
            unreachable!()
        }
    }
}
