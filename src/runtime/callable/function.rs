use crate::ast::Stmt;
use crate::runtime::callable::LoxCallable;
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::environment::{EnvRef, Environment};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;

pub(crate) struct LoxFunction {
    declaration: Stmt,
    closure: EnvRef,
}

impl LoxFunction {
    pub(crate) fn new(declaration: Stmt, closure: EnvRef) -> Self {
        match declaration {
            Stmt::Function { .. } => Self {
                declaration,
                closure,
            },
            _ => unreachable!(),
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        match &self.declaration {
            Stmt::Function {
                name: _,
                parameters,
                body: _,
            } => parameters.len(),
            _ => unreachable!(),
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Value],
    ) -> Result<Value, ControlFlow> {
        match &self.declaration {
            Stmt::Function {
                name: _name,
                parameters,
                body,
            } => {
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
        match &self.declaration {
            Stmt::Function {
                name,
                parameters: _,
                body: _,
            } => name.lexeme.clone(),
            _ => unreachable!(),
        }
    }
}
