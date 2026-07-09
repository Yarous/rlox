use crate::runtime::callable::LoxCallable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;

pub(crate) struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: &[Value],
    ) -> Result<Value, ControlFlow> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        Ok(Value::Number(now))
    }

    fn name(&self) -> String {
        "Clock".to_string()
    }
}
