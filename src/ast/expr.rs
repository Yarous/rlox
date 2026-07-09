use crate::ast::Stmt;
use crate::frontend::token::{Literal, Token, TokenKind};

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
    Lambda {
        parameters: Vec<Token>,
        body: Vec<Stmt>,
    },
}
