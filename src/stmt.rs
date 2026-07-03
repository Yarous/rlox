use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    ExprStmt(Expr),
    PrintStmt(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>)
}
