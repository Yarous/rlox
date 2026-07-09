use crate::ast::{Expr, Stmt};
use std::fmt;
use std::fmt::Display;

fn write_joined<T>(f: &mut fmt::Formatter<'_>, values: &[T], separator: &str) -> fmt::Result
where
    T: Display,
{
    if let Some((first, rest)) = values.split_first() {
        write!(f, "{first}")?;
        for value in rest {
            write!(f, "{separator}{value}")?;
        }
    }
    Ok(())
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let operator = operator.token_type;
                write!(f, "({left} {operator} {right})")
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => write!(f, "({condition} ? {then_expr} : {else_expr})"),
            Expr::Unary { operator, right } => {
                let operator = operator.token_type;
                write!(f, "({operator} {right})")
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let operator = operator.token_type;
                write!(f, "({left} {operator} {right})")
            }
            Expr::Literal(literal) => write!(f, "{literal}"),
            Expr::Grouping(expr) => write!(f, "({expr})"),
            Expr::Variable(var) => write!(f, "{var}"),
            Expr::Assignment { name, value } => write!(f, "{name} = {value}"),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                write!(f, "({callee} {paren} ")?;
                write_joined(f, arguments, " ")?;
                write!(f, ")")
            }
            Expr::Lambda { parameters, body } => {
                write!(f, "(")?;
                write_joined(f, parameters, " ")?;
                write!(f, " ")?;
                write_joined(f, body, " ")?;
                write!(f, ")")
            }
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "({expr})"),
            Stmt::Print(expr) => write!(f, "print {expr}"),
            Stmt::Var { name, initializer } => {
                if let Some(init) = initializer {
                    write!(f, "{name} = {init}")
                } else {
                    write!(f, "{name}")
                }
            }
            Stmt::Block(stmts) => write_joined(f, stmts, " "),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(else_branch) = else_branch {
                    write!(f, "if {condition} {then_branch} {else_branch}")
                } else {
                    write!(f, "if {condition} {then_branch}")
                }
            }
            Stmt::While { condition, body } => {
                write!(f, "while {condition} {{ {body} }}")
            }
            Stmt::Break => write!(f, "break;"),
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                write!(f, "fn {name}(")?;
                write_joined(f, parameters, " ")?;
                write!(f, ") ")?;
                write_joined(f, body, " ")
            }
            Stmt::Return { keyword, value } => {
                if let Some(value) = value {
                    write!(f, "return {keyword} = {value};")
                } else {
                    write!(f, "return {keyword}")
                }
            }
        }
    }
}
