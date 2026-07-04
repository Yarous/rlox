use crate::interpreter::{RuntimeError, ValidationError, Value};
use crate::token::Token;
use std::collections::HashMap;

#[derive(Debug, Clone)] // TODO: Remove cloning
pub(crate) struct Environment {
    values: HashMap<String, Option<Value>>,
    pub(crate) enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub(crate) fn add_enclosing(&mut self, environment: Environment) {
        self.enclosing = Some(Box::new(environment));
    }

    pub(crate) fn define(&mut self, name: &Token, value: Option<Value>) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.values.get(&name.lexeme) {
            None => Err({
                if self.enclosing.is_some() {
                    return self.enclosing.clone().unwrap().get(name);
                }
                RuntimeError {
                    line: name.line,
                    error_type: ValidationError::UndefinedVariable(name.lexeme.clone()),
                }
            }),
            Some(value) => match value {
                Some(value) => Ok(value.clone()),
                None => Err({
                    RuntimeError {
                        line: name.line,
                        error_type: ValidationError::UninitializedVariable(name.lexeme.clone()),
                    }
                }),
            },
        }
    }

    pub(crate) fn assign(
        &mut self,
        name: &Token,
        value: Option<Value>,
    ) -> Result<Value, RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(self.values.get(&name.lexeme).unwrap().clone().unwrap()); // TODO: remove unwrap X2 here =).
        }

        if self.enclosing.is_some() {
            return self.enclosing.clone().unwrap().get(name);
        }

        Err(RuntimeError {
            line: name.line,
            error_type: ValidationError::UndefinedVariable(name.lexeme.clone()),
        })
    }
}
