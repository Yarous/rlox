use crate::runtime::error::RuntimeError;
use crate::runtime::value::Value;

#[derive(Debug)]
pub(crate) enum ControlFlow {
    RuntimeError(RuntimeError),
    Break,
    Return(Value),
}
