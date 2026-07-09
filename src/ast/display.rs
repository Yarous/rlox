use crate::ast::{Expr, Stmt};
use std::fmt;
use std::fmt::Display;

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", left, operator.token_type, right),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => write!(f, "({} ? {} : {})", condition, then_expr, else_expr),
            Expr::Unary { operator, right } => write!(f, "({} {})", operator.token_type, right),
            Expr::Logical {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", left, operator.token_type, right),
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Grouping(expr) => write!(f, "({})", expr),
            Expr::Variable(var) => write!(f, "{}", var),
            Expr::Assignment { name, value } => write!(f, "{} = {}", name, value),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => write!(
                f,
                "({} {} {})",
                callee,
                paren,
                arguments
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Expr::Lambda { parameters, body } => write!(
                f,
                "({} {})",
                parameters
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                body.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "({})", expr),
            Stmt::Print(expr) => write!(f, "print {}", expr),
            Stmt::Var { name, initializer } => {
                if let Some(init) = initializer {
                    write!(f, "{} = {}", name, init)
                } else {
                    write!(f, "{}", name)
                }
            }
            Stmt::Block(stmts) => {
                write!(
                    f,
                    "{}",
                    stmts
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(else_branch) = else_branch {
                    write!(f, "if {} {} {}", condition, then_branch, else_branch)
                } else {
                    write!(f, "if {} {}", condition, then_branch)
                }
            }
            Stmt::While { condition, body } => {
                write!(f, "while {} {{ {} }}", condition, body)
            }
            Stmt::Break => write!(f, "break;"),
            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                write!(
                    f,
                    "fn {}({}) {}",
                    name,
                    parameters
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                    body.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            Stmt::Return { keyword, value } => {
                if let Some(value) = value {
                    write!(f, "return {} = {};", keyword, value)
                } else {
                    write!(f, "return {}", keyword)
                }
            }
        }
    }
}
