use crate::expr::Expr;
use crate::token::Token;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    ExprStmt(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Break,
    Function {
        name: Token,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::ExprStmt(expr) => write!(f, "({})", expr),
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
