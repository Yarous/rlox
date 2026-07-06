use crate::expr::Expr;
use crate::token::Token;

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
    // Break,
}
