use crate::token::{Literal, Token, TokenKind};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Op {
    pub(crate) token_type: TokenKind,
    pub(crate) line: usize,
}

impl From<&Token> for Op {
    fn from(token: &Token) -> Self {
        Self {
            token_type: token.token_type,
            line: token.line,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Op,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    Unary {
        operator: Op,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Op,
        right: Box<Expr>,
    },
    Literal(Literal),
    Grouping(Box<Expr>),
    Variable(Token),
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        }
    }
}
