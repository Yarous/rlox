use super::*;
use crate::frontend::parser::Parser;
use crate::frontend::scanner::Scanner;

fn parse(source: &str) -> Vec<Stmt> {
    let mut scanner = Scanner::new(source);
    let scan_result = scanner.scan_tokens();
    assert!(scan_result.errors.is_empty());
    let tokens = scan_result.tokens;
    let tokens = Box::leak(tokens.into_boxed_slice());
    let result = Parser::new(tokens).parse();
    assert!(result.errors.is_empty());
    result.statements.unwrap()
}

fn interpreter() -> Interpreter {
    Interpreter::new()
}

fn eval(source: &str) -> Result<Value, ControlFlow> {
    let statements = parse(source);
    let mut interpreter = interpreter();

    match statements.last().unwrap() {
        Stmt::Expression(expr) | Stmt::Print(expr) => interpreter.evaluate(expr),
        _ => panic!("expected expression-compatible final statement"),
    }
}

fn run_then_eval(source: &str) -> Result<Value, ControlFlow> {
    let statements = parse(source);
    let mut interpreter = interpreter();
    let (last, prefix) = statements.split_last().unwrap();

    for stmt in prefix {
        interpreter.execute(stmt)?;
    }

    match last {
        Stmt::Expression(expr) | Stmt::Print(expr) => interpreter.evaluate(expr),
        _ => panic!("expected expression-compatible final statement"),
    }
}

fn runtime_error(source: &str) -> ValidationError {
    match eval(source).unwrap_err() {
        ControlFlow::RuntimeError(error) => error.error_type,
        _ => panic!("expected runtime error"),
    }
}

#[test]
fn evaluates_numeric_arithmetic() {
    assert_eq!(eval("(1 + 2) * 3 - 4 / 2;").unwrap(), Value::Number(7.0));
}

#[test]
fn evaluates_unary_expressions() {
    assert_eq!(eval("-5;").unwrap(), Value::Number(-5.0));
    assert_eq!(eval("!false;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("!nil;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("!0;").unwrap(), Value::Boolean(false));
}

#[test]
fn evaluates_comparisons() {
    assert_eq!(eval("2 > 1;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("2 >= 2;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("1 < 2;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("2 <= 2;").unwrap(), Value::Boolean(true));
}

#[test]
fn evaluates_equality() {
    assert_eq!(eval("nil == nil;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("true != false;").unwrap(), Value::Boolean(true));
    assert_eq!(eval("1 == \"1\";").unwrap(), Value::Boolean(false));
}

#[test]
fn evaluates_string_concatenation() {
    assert_eq!(
        eval("\"hello\" + \" \" + \"lox\";").unwrap(),
        Value::String("hello lox".to_string())
    );
}

#[test]
fn evaluates_string_coercing_concatenation() {
    assert_eq!(
        eval("\"value=\" + 3;").unwrap(),
        Value::String("value=3".to_string())
    );
    assert_eq!(
        eval("3 + \" apples\";").unwrap(),
        Value::String("3 apples".to_string())
    );
}

#[test]
fn short_circuits_or() {
    assert_eq!(eval("true or missing;").unwrap(), Value::Boolean(true));
}

#[test]
fn short_circuits_and() {
    assert_eq!(eval("false and missing;").unwrap(), Value::Boolean(false));
}

#[test]
fn evaluates_ternary_branches() {
    assert_eq!(eval("true ? 1 : missing;").unwrap(), Value::Number(1.0));
    assert_eq!(eval("false ? missing : 2;").unwrap(), Value::Number(2.0));
}

#[test]
fn reports_division_by_zero() {
    assert!(matches!(
        runtime_error("1 / 0;"),
        ValidationError::DivisionByZero
    ));
}

#[test]
fn reports_invalid_unary_operand() {
    assert!(matches!(
        runtime_error("-\"x\";"),
        ValidationError::InvalidOperandTypes { .. }
    ));
}

#[test]
fn reports_invalid_binary_operand() {
    assert!(matches!(
        runtime_error("true * 2;"),
        ValidationError::InvalidOperandTypes { .. }
    ));
}

#[test]
fn reports_undefined_variable() {
    assert!(matches!(
        runtime_error("missing;"),
        ValidationError::UndefinedVariable(name) if name == "missing"
    ));
}

#[test]
fn reports_uninitialized_variable() {
    assert!(matches!(
        run_then_eval("var a; a;").unwrap_err(),
        ControlFlow::RuntimeError(RuntimeError {
            error_type: ValidationError::UninitializedVariable(name),
            ..
        }) if name == "a"
    ));
}

#[test]
fn evaluates_variable_assignment() {
    assert_eq!(
        run_then_eval("var a = 1; a = a + 2;").unwrap(),
        Value::Number(3.0)
    );
}

#[test]
fn block_scope_shadows_outer_variable() {
    assert_eq!(
        run_then_eval("var a = 1; { var a = 2; } a;").unwrap(),
        Value::Number(1.0)
    );
}

#[test]
fn assignment_updates_outer_scope() {
    assert_eq!(
        run_then_eval("var a = 1; { a = 4; } a;").unwrap(),
        Value::Number(4.0)
    );
}

#[test]
fn evaluates_while_loop() {
    assert_eq!(
        run_then_eval("var i = 0; while (i < 3) { i = i + 1; } i;").unwrap(),
        Value::Number(3.0)
    );
}

#[test]
fn evaluates_break() {
    assert_eq!(
        run_then_eval("var i = 0; while (true) { i = i + 1; break; } i;").unwrap(),
        Value::Number(1.0)
    );
}

#[test]
fn evaluates_function_return() {
    assert_eq!(
        run_then_eval("fun add(a, b) { return a + b; } add(2, 3);").unwrap(),
        Value::Number(5.0)
    );
}

#[test]
fn function_without_return_returns_nil() {
    assert_eq!(run_then_eval("fun noop() {} noop();").unwrap(), Value::Nil);
}

#[test]
fn evaluates_lambda_return() {
    assert_eq!(
        run_then_eval("var double = fun (x) { return x * 2; }; double(4);").unwrap(),
        Value::Number(8.0)
    );
}

#[test]
fn evaluates_closure_capturing_environment() {
    assert_eq!(
            run_then_eval(
                "fun make_counter() { var i = 0; fun next() { i = i + 1; return i; } return next; } var counter = make_counter(); counter(); counter();"
            )
            .unwrap(),
            Value::Number(2.0)
        );
}

#[test]
fn reports_non_callable_value() {
    assert!(matches!(
        runtime_error("1();"),
        ValidationError::NonCallable(message) if message == "Can only call functions and classes"
    ));
}

#[test]
fn reports_wrong_arity() {
    assert!(matches!(
        run_then_eval("fun one(a) { return a; } one();").unwrap_err(),
        ControlFlow::RuntimeError(RuntimeError {
            error_type: ValidationError::NotMatchedArity(message),
            ..
        }) if message == "Expected 1 arguments but got 0."
    ));
}
