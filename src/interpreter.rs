use crate::Lox;
use crate::expr::Expr;
use crate::token::{Literal, Token, TokenKind};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) enum ValidationError {
    DivisionByZero,
    InvalidOperandTypes {
        expected: Vec<ValueType>,
        actual_left: Option<Value>,
        actual_right: Value,
    },
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let begin = "Runtime/Validation Error: ";
        match self {
            ValidationError::DivisionByZero => write!(f, "{}{}", begin, "Division by Zero"),
            ValidationError::InvalidOperandTypes {
                expected,
                actual_left,
                actual_right,
            } => {
                let expected_str: Vec<String> = expected.iter().map(|v| v.to_string()).collect();
                let expected_joined = expected_str.join(" or ");
                if let Some(actual_left) = actual_left {
                    write!(
                        f,
                        "{} Invalid operand types. Expected: [{}], but got: [{}] and [{}]",
                        begin, expected_joined, actual_left, actual_right
                    )
                } else {
                    write!(
                        f,
                        "{} Invalid operand types. Expected: [{}], but got: [{}]",
                        begin, expected_joined, actual_right
                    )
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeError {
    pub(crate) token: Token,
    pub(crate) error_type: ValidationError,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ValueType {
    Number,
    String,
    Boolean,
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Number => write!(f, "Number"),
            ValueType::String => write!(f, "String"),
            ValueType::Boolean => write!(f, "Boolean"),
            ValueType::Nil => write!(f, "Nil"),
        }
    }
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Number(number) => Value::Number(number),
            Literal::String(string) => Value::String(string),
            Literal::True => Value::Boolean(true),
            Literal::False => Value::Boolean(false),
            Literal::Nil => Value::Nil,
        }
    }
}

pub(crate) struct Interpreter<'a> {
    lox: &'a mut Lox,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn new(lox: &'a mut Lox) -> Self {
        Self { lox }
    }

    pub(crate) fn interpret(&mut self, expression: Expr) {
        match self.evaluate(&expression) {
            Ok(value) => println!("{}", value),
            Err(error) => self.lox.runtime_error(error),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(literal) => Ok(Value::from(literal.clone())),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary { operator, right } => {
                let right_val = self.evaluate(right)?;

                match operator.token_type {
                    TokenKind::Minus => {
                        if let Value::Number(number) = right_val {
                            Ok(Value::Number(-number))
                        } else {
                            Err(RuntimeError {
                                token: operator.clone(),
                                error_type: ValidationError::InvalidOperandTypes {
                                    expected: vec![ValueType::Number],
                                    actual_left: None,
                                    actual_right: right_val,
                                },
                            })
                        }
                    }
                    TokenKind::Bang => Ok(Value::Boolean(!self.truthy(&right_val))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenKind::Minus => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Number(l - r)
                        })
                    }
                    TokenKind::Plus => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        (Value::String(left), right) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        (left, Value::String(right)) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        _ => Err(RuntimeError {
                            token: operator.clone(),
                            error_type: ValidationError::InvalidOperandTypes {
                                expected: vec![ValueType::Number, ValueType::String],
                                actual_left: Some(left),
                                actual_right: right,
                            },
                        }),
                    },
                    TokenKind::Star => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Number(l * r)
                        })
                    }
                    TokenKind::Slash => {
                        if let (Value::Number(left), Value::Number(right)) =
                            (left.clone(), right.clone())
                        {
                            if right != 0_f64 {
                                Ok(Value::Number(left / right))
                            } else {
                                Err(RuntimeError {
                                    token: operator.clone(),
                                    error_type: ValidationError::DivisionByZero,
                                })
                            }
                        } else {
                            Err(RuntimeError {
                                token: operator.clone(),
                                error_type: ValidationError::InvalidOperandTypes {
                                    expected: vec![ValueType::Number],
                                    actual_left: Some(left),
                                    actual_right: right,
                                },
                            })
                        }
                    }
                    TokenKind::Greater => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Boolean(l > r)
                        })
                    }
                    TokenKind::GreaterEqual => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Boolean(l >= r)
                        })
                    }
                    TokenKind::Less => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Boolean(l < r)
                        })
                    }
                    TokenKind::LessEqual => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Boolean(l <= r)
                        })
                    }
                    TokenKind::EqualEqual => Ok(Value::Boolean(left == right)),
                    TokenKind::BangEqual => Ok(Value::Boolean(left != right)),
                    _ => unreachable!(),
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition = self.evaluate(condition)?;

                if self.truthy(&condition) {
                    self.evaluate(&then_expr)
                } else {
                    self.evaluate(&else_expr)
                }
            }
        }
    }

    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    fn check_number_operands<T>(
        &self,
        operator: &Token,
        left: &Value,
        right: &Value,
        operation: T,
    ) -> Result<Value, RuntimeError>
    where
        T: FnOnce(f64, f64) -> Value,
    {
        if let (Value::Number(left), Value::Number(right)) = (left, right) {
            Ok(operation(*left, *right))
        } else {
            Err(RuntimeError {
                token: operator.clone(),
                error_type: ValidationError::InvalidOperandTypes {
                    expected: vec![ValueType::Number],
                    actual_left: Some(left.clone()),
                    actual_right: right.clone(),
                },
            })
        }
    }
}
