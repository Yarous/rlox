use crate::ast::expr::Expr;
use crate::frontend::token::Token;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Expression(Expr),
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
