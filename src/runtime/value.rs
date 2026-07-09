use crate::frontend::token::Literal;
use crate::runtime::callable::LoxCallable;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    Callable(Rc<dyn LoxCallable>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValueType {
    Number,
    String,
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
            Value::Callable(c) => write!(f, "{c}"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Callable(a), Value::Callable(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Number => write!(f, "Number"),
            ValueType::String => write!(f, "String"),
        }
    }
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Number(number) => Value::Number(number),
            Literal::String(string) => Value::String(string),
            Literal::Boolean(value) => Value::Boolean(value),
            Literal::Nil => Value::Nil,
        }
    }
}
