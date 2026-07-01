use crate::token::{Literal, Token, TokenKind};

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
    Literal(Literal),
    Grouping(Box<Expr>),
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
            Expr::Literal(literal) => write!(f, "{}", literal),
            Expr::Grouping(expr) => write!(f, "({})", expr),
        }
    }
}
