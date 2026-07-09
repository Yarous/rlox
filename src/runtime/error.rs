use crate::runtime::value::{Value, ValueType};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub(crate) enum ValidationError {
    DivisionByZero,
    InvalidOperandTypes {
        expected: &'static [ValueType],
        actual_left: Option<Value>,
        actual_right: Value,
    },
    UndefinedVariable(String),
    UninitializedVariable(String),
    NonCallable(String),
    NotMatchedArity(String),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let begin = "Validation Error: ";
        match self {
            ValidationError::DivisionByZero => write!(f, "{begin}Division by Zero"),
            ValidationError::InvalidOperandTypes {
                expected,
                actual_left,
                actual_right,
            } => {
                let expected_str = expected
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" or ");
                if let Some(actual_left) = actual_left {
                    write!(
                        f,
                        "{begin} Invalid operand types. Expected: [{expected_str}], but got: [{actual_left}] and [{actual_right}]"
                    )
                } else {
                    write!(
                        f,
                        "{begin} Invalid operand types. Expected: [{expected_str}], but got: [{actual_right}]"
                    )
                }
            }
            ValidationError::UndefinedVariable(where_undef) => {
                write!(f, "{begin} Undefined variable: {where_undef}")
            }
            ValidationError::UninitializedVariable(where_uninit) => {
                write!(f, "{begin} Uninitialized variable: {where_uninit}")
            }
            ValidationError::NonCallable(non_callable_reason) => {
                write!(f, "{begin} Non-callable expression: {non_callable_reason}")
            }
            ValidationError::NotMatchedArity(st) => {
                write!(f, "{begin} {st}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeError {
    pub(crate) line: usize,
    pub(crate) error_type: ValidationError,
}
