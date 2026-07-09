mod function;
mod lambda;
mod native;

pub(crate) use function::LoxFunction;
pub(crate) use lambda::LoxLambda;
pub(crate) use native::Clock;

use crate::runtime::control_flow::ControlFlow;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;
use std::fmt;
use std::fmt::Display;

pub(crate) trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Value],
    ) -> Result<Value, ControlFlow>;
    fn name(&self) -> String;
}

impl fmt::Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}

impl Display for dyn LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.name())
    }
}
