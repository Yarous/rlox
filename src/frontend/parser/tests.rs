use super::*;
use crate::ast::Expr;
use crate::frontend::scanner::Scanner;

fn parse(source: &str) -> ParseResult<'_> {
    let mut scanner = Scanner::new(source);
    let scan_result = scanner.scan_tokens();
    assert!(scan_result.errors.is_empty());
    let tokens = scan_result.tokens;
    let tokens = Box::leak(tokens.into_boxed_slice());
    Parser::new(tokens).parse()
}

fn parse_ok(source: &str) -> Vec<Stmt> {
    let result = parse(source);
    assert!(result.errors.is_empty());
    result.statements.unwrap()
}

fn parse_errors(source: &str) -> Vec<&'static str> {
    parse(source)
        .errors
        .into_iter()
        .map(|error| error.message)
        .collect()
}

#[test]
fn parses_empty_program() {
    let statements = parse_ok("");

    assert!(statements.is_empty());
}

#[test]
fn parses_print_statement() {
    let statements = parse_ok("print 1 + 2 * 3;");

    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::Print(expr) => assert_eq!(expr.to_string(), "(Number Plus (Number Star Number))"),
        _ => panic!("expected print statement"),
    }
}

#[test]
fn parses_variable_without_initializer() {
    let statements = parse_ok("var a;");

    match &statements[0] {
        Stmt::Var { name, initializer } => {
            assert_eq!(name.lexeme, "a");
            assert!(initializer.is_none());
        }
        _ => panic!("expected var statement"),
    }
}

#[test]
fn parses_variable_with_initializer() {
    let statements = parse_ok("var a = 42;");

    match &statements[0] {
        Stmt::Var { name, initializer } => {
            assert_eq!(name.lexeme, "a");
            assert_eq!(initializer.as_ref().unwrap().to_string(), "Number");
        }
        _ => panic!("expected var statement"),
    }
}

#[test]
fn parses_assignment_expression() {
    let statements = parse_ok("a = 3;");

    match &statements[0] {
        Stmt::Expression(Expr::Assignment { name, value }) => {
            assert_eq!(name.lexeme, "a");
            assert_eq!(value.to_string(), "Number");
        }
        _ => panic!("expected assignment expression"),
    }
}

#[test]
fn rejects_invalid_assignment_target() {
    let errors = parse_errors("(a + b) = 3;");

    assert!(errors.contains(&"Invalid assignment target!"));
}

#[test]
fn parses_right_associative_assignment() {
    let statements = parse_ok("a = b = 1;");

    match &statements[0] {
        Stmt::Expression(expr) => {
            assert_eq!(expr.to_string(), "Identifier a = Identifier b = Number");
        }
        _ => panic!("expected expression statement"),
    }
}

#[test]
fn parses_ternary_expression() {
    let statements = parse_ok("print true ? 1 : 2;");

    match &statements[0] {
        Stmt::Print(expr) => assert_eq!(expr.to_string(), "(Boolean ? Number : Number)"),
        _ => panic!("expected print statement"),
    }
}

#[test]
fn parses_logical_expression() {
    let statements = parse_ok("print true or false and nil;");

    match &statements[0] {
        Stmt::Print(expr) => assert_eq!(expr.to_string(), "(Boolean Or (Boolean And Nil))"),
        _ => panic!("expected print statement"),
    }
}

#[test]
fn parses_grouping_and_unary_precedence() {
    let statements = parse_ok("print -(1 + 2) * 3;");

    match &statements[0] {
        Stmt::Print(expr) => {
            assert_eq!(
                expr.to_string(),
                "((Minus ((Number Plus Number))) Star Number)"
            );
        }
        _ => panic!("expected print statement"),
    }
}

#[test]
fn parses_if_else_statement() {
    let statements = parse_ok("if (true) print 1; else print 2;");

    match &statements[0] {
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert_eq!(condition.to_string(), "Boolean");
            assert!(matches!(then_branch.as_ref(), Stmt::Print(_)));
            assert!(matches!(else_branch.as_deref(), Some(Stmt::Print(_))));
        }
        _ => panic!("expected if statement"),
    }
}

#[test]
fn parses_while_statement() {
    let statements = parse_ok("while (false) print 1;");

    match &statements[0] {
        Stmt::While { condition, body } => {
            assert_eq!(condition.to_string(), "Boolean");
            assert!(matches!(body.as_ref(), Stmt::Print(_)));
        }
        _ => panic!("expected while statement"),
    }
}

#[test]
fn parses_for_statement_as_block_and_while() {
    let statements = parse_ok("for (var i = 0; i < 3; i = i + 1) print i;");

    match &statements[0] {
        Stmt::Block(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], Stmt::Var { .. }));
            assert!(matches!(items[1], Stmt::While { .. }));
        }
        _ => panic!("expected desugared block"),
    }
}

#[test]
fn parses_break_inside_loop() {
    let statements = parse_ok("while (true) break;");

    match &statements[0] {
        Stmt::While { body, .. } => assert!(matches!(body.as_ref(), Stmt::Break)),
        _ => panic!("expected while statement"),
    }
}

#[test]
fn rejects_break_outside_loop() {
    let errors = parse_errors("break;");

    assert!(errors.contains(&"Can't use 'break' outside of a loop."));
}

#[test]
fn rejects_return_outside_function() {
    let errors = parse_errors("return 1;");

    assert!(errors.contains(&"return statement only allowed in functions/lambda expressions"));
}

#[test]
fn parses_return_inside_function() {
    let statements = parse_ok("fun answer() { return 42; }");

    match &statements[0] {
        Stmt::Function {
            name,
            parameters,
            body,
        } => {
            assert_eq!(name.lexeme, "answer");
            assert!(parameters.is_empty());
            assert!(matches!(body[0], Stmt::Return { .. }));
        }
        _ => panic!("expected function statement"),
    }
}

#[test]
fn parses_lambda_expression_with_return() {
    let statements = parse_ok("var f = fun (x) { return x; };");

    match &statements[0] {
        Stmt::Var {
            initializer: Some(Expr::Lambda { parameters, body }),
            ..
        } => {
            assert_eq!(parameters[0].lexeme, "x");
            assert!(matches!(body[0], Stmt::Return { .. }));
        }
        _ => panic!("expected lambda initializer"),
    }
}

#[test]
fn rejects_missing_left_operand() {
    let errors = parse_errors("* 2;");

    assert!(errors.contains(&"Missing left operand"));
}

#[test]
fn recovers_after_parse_error() {
    let result = parse("var = 1; print 2;");

    assert!(!result.errors.is_empty());
    assert_eq!(result.statements.unwrap().len(), 1);
}

#[test]
fn rejects_more_than_255_arguments() {
    let arguments = std::iter::repeat_n("1", 256).collect::<Vec<_>>().join(", ");
    let source = format!("f({arguments});");
    let errors = parse_errors(&source);

    assert!(errors.contains(&"cant have more than 255 arguments"));
}

#[test]
fn rejects_more_than_255_parameters() {
    let parameters = (0..256)
        .map(|index| format!("p{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!("fun too_many({parameters}) {{ print 1; }}");
    let errors = parse_errors(&source);

    assert!(errors.contains(&"Cant have more than 255 parameters."));
}
