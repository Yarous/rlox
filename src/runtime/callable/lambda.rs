use crate::ast::Expr;
use crate::runtime::callable::LoxCallable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::environment::{EnvRef, Environment};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;

pub(crate) struct LoxLambda {
    declaration: Expr,
    closure: EnvRef,
}

impl LoxLambda {
    pub(crate) fn new(declaration: Expr, closure: EnvRef) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl LoxCallable for LoxLambda {
    fn arity(&self) -> usize {
        if let Expr::Lambda {
            parameters,
            body: _body,
        } = &self.declaration
        {
            return parameters.len();
        }
        unreachable!()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Value],
    ) -> Result<Value, ControlFlow> {
        match &self.declaration {
            Expr::Lambda { parameters, body } => {
                let environment = Environment::add_enclosing(self.closure.clone());

                for i in 0..parameters.len() {
                    environment.borrow_mut().define(
                        parameters.get(i).unwrap().lexeme.clone(),
                        arguments.get(i).cloned(),
                    );
                }

                match interpreter.evaluate_block_statement(body, environment) {
                    Ok(_) => (),
                    Err(ControlFlow::Return(value)) => return Ok(value),
                    Err(e) => return Err(e),
                }
                Ok(Value::Nil)
            }
            _ => unreachable!(),
        }
    }

    fn name(&self) -> String {
        "Lambda".to_string()
    }
}
