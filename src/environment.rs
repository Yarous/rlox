use crate::interpreter::{RuntimeError, ValidationError, Value};
use crate::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)] // TODO: Remove cloning
pub(crate) struct Environment {
    pub(crate) values: HashMap<String, Option<Value>>,
    pub(crate) enclosing: Option<EnvRef>,
}

pub(crate) type EnvRef = Rc<RefCell<Environment>>;

impl Environment {
    pub(crate) fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub(crate) fn add_enclosing(enclosing: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub(crate) fn define(&mut self, name: &Token, value: Option<Value>) {
        self.values.insert(name.lexeme.clone(), value);
    }

    pub(crate) fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return value.clone().ok_or(RuntimeError {
                line: name.line,
                error_type: ValidationError::UninitializedVariable(name.lexeme.clone()),
            });
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }
        Err(RuntimeError {
            line: name.line,
            error_type: ValidationError::UndefinedVariable(name.lexeme.clone()),
        })
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

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError {
            line: name.line,
            error_type: ValidationError::UndefinedVariable(name.lexeme.clone()),
        })
    }
}
