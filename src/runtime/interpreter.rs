use crate::ast::{Expr, Op, Stmt};
use crate::frontend::token::{Token, TokenKind};
use crate::runtime::callable::{Clock, LoxFunction, LoxLambda};
use crate::runtime::control_flow::ControlFlow;
use crate::runtime::environment::{EnvRef, Environment};
use crate::runtime::error::{RuntimeError, ValidationError};
use crate::runtime::value::{Value, ValueType};
use std::rc::Rc;

pub(crate) struct Interpreter {
    environment: EnvRef,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        let globals = Environment::new();
        let environment = globals.clone();

        globals
            .borrow_mut()
            .define("clock".to_string(), Some(Value::Callable(Rc::new(Clock))));

        Self { environment }
    }

    pub(crate) fn interpret(&mut self, statements: &[Stmt]) -> Result<(), ControlFlow> {
        for stmt in statements {
            match self.execute(stmt) {
                Ok(()) => (),
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::Expression(_) => self.evaluate_expression_statement(stmt),
            Stmt::Print(_) => self.evaluate_print_statement(stmt),
            Stmt::Var { name, initializer } => self.evaluate_var_statement(name, initializer),
            Stmt::Block(stmts) => {
                let block_env = Environment::add_enclosing(Rc::clone(&self.environment));
                self.evaluate_block_statement(stmts, block_env)
            }
            Stmt::If { .. } => self.evaluate_if_statement(stmt),
            Stmt::While { .. } => self.evaluate_while_statement(stmt),
            Stmt::Break => Err(ControlFlow::Break),
            Stmt::Function { .. } => self.evaluate_function_statement(stmt),
            Stmt::Return { .. } => self.evaluate_return_statement(stmt),
        }
    }

    fn evaluate_lambda_expression(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
        if let Expr::Lambda { .. } = expr {
            let lambda = LoxLambda::new(expr.clone(), self.environment.clone());

            return Ok(Value::Callable(Rc::new(lambda)));
        }
        unreachable!()
    }

    fn evaluate_return_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        if let Stmt::Return {
            keyword: _keyword,
            value,
        } = stmt
        {
            let returning_value: Value;
            if let Some(value) = value {
                returning_value = self.evaluate(value)?;
            } else {
                returning_value = Value::Nil;
            }

            return Err(ControlFlow::Return(returning_value));
        }
        unreachable!()
    }

    fn evaluate_call_expression(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
        if let Expr::Call {
            callee,
            paren,
            arguments,
        } = expr
        {
            let callee = self.evaluate(callee)?;
            let mut arguments_list: Vec<Value> = Vec::new();
            for arg in arguments {
                arguments_list.push(self.evaluate(arg)?);
            }

            let function = match callee {
                Value::Callable(f) => f,
                _ => {
                    return Err(ControlFlow::RuntimeError(RuntimeError {
                        line: paren.line,
                        error_type: ValidationError::NonCallable(
                            "Can only call functions and classes".into(),
                        ),
                    }));
                }
            };
            if arguments_list.len() != function.arity() {
                return Err(ControlFlow::RuntimeError(RuntimeError {
                    line: paren.line,
                    error_type: ValidationError::NotMatchedArity(format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        arguments_list.len()
                    )),
                }));
            }

            return function.call(self, &arguments_list);
        }
        unreachable!()
    }

    fn evaluate_function_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        if let Stmt::Function { name, .. } = stmt {
            let function = LoxFunction::new(stmt.clone(), self.environment.clone());
            self.environment.borrow_mut().define(
                name.lexeme.clone(),
                Some(Value::Callable(Rc::new(function))),
            );

            return Ok(());
        }

        unreachable!()
    }

    fn evaluate_while_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        if let Stmt::While { condition, body } = stmt {
            while {
                let while_condition = self.evaluate(condition)?;
                self.truthy(&while_condition)
            } {
                match self.execute(body) {
                    Ok(()) => {}
                    Err(ControlFlow::Break) => return Ok(()),
                    Err(error) => return Err(error),
                }
            }
            return Ok(());
        }

        unreachable!()
    }

    fn evaluate_logical_expression(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
        if let Expr::Logical {
            left,
            operator,
            right,
        } = expr
        {
            let left = self.evaluate(left)?;
            if operator.token_type == TokenKind::Or {
                if self.truthy(&left) {
                    return Ok(left);
                }
            } else {
                if !self.truthy(&left) {
                    return Ok(left);
                }
            }

            return self.evaluate(right);
        }
        unreachable!();
    }

    fn evaluate_if_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        if let Stmt::If {
            condition,
            then_branch,
            else_branch,
        } = stmt
        {
            let condition = self.evaluate(condition)?;

            if self.truthy(&condition) {
                self.execute(then_branch)?;
            } else if let Some(else_branch) = else_branch {
                self.execute(else_branch)?;
            }
            return Ok(());
        }
        unreachable!();
    }

    pub(crate) fn evaluate_block_statement(
        &mut self,
        statements: &[Stmt],
        environment: EnvRef,
    ) -> Result<(), ControlFlow> {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => {}
                Err(err) => {
                    self.environment = previous;
                    return Err(err);
                }
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn evaluate_var_statement(
        &mut self,
        name: &Token,
        initializer: &Option<Expr>,
    ) -> Result<(), ControlFlow> {
        let value = match initializer {
            Some(expr) => Some(self.evaluate(expr)?),
            None => None,
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        Ok(())
    }

    fn evaluate_assign_expression(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
        if let Expr::Assignment { name, value } = expr {
            let value = Some(self.evaluate(value)?);
            self.environment.borrow_mut().assign(name, value)
        } else {
            unreachable!()
        }
    }

    fn evaluate_expression_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn evaluate_print_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{value}");
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, ControlFlow> {
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
                            Err(ControlFlow::RuntimeError(RuntimeError {
                                line: operator.line,
                                error_type: ValidationError::InvalidOperandTypes {
                                    expected: &[ValueType::Number],
                                    actual_left: None,
                                    actual_right: right_val,
                                },
                            }))
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
                    TokenKind::Plus => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String(format!("{left}{right}")))
                        }
                        (Value::String(left), right) => Ok(Value::String(format!("{left}{right}"))),
                        (left, Value::String(right)) => Ok(Value::String(format!("{left}{right}"))),
                        (left, right) => Err(ControlFlow::RuntimeError(RuntimeError {
                            line: operator.line,
                            error_type: ValidationError::InvalidOperandTypes {
                                expected: &[ValueType::Number, ValueType::String],
                                actual_left: Some(left),
                                actual_right: right,
                            },
                        })),
                    },
                    TokenKind::Star => {
                        self.check_number_operands(operator, &left, &right, |l, r| {
                            Value::Number(l * r)
                        })
                    }
                    TokenKind::Slash => match (left, right) {
                        (Value::Number(left), Value::Number(right)) => {
                            if right != 0_f64 {
                                Ok(Value::Number(left / right))
                            } else {
                                Err(ControlFlow::RuntimeError(RuntimeError {
                                    line: operator.line,
                                    error_type: ValidationError::DivisionByZero,
                                }))
                            }
                        }
                        (left, right) => Err(ControlFlow::RuntimeError(RuntimeError {
                            line: operator.line,
                            error_type: ValidationError::InvalidOperandTypes {
                                expected: &[ValueType::Number],
                                actual_left: Some(left),
                                actual_right: right,
                            },
                        })),
                    },
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
                    TokenKind::Comma => Ok(right),
                    _ => unreachable!(),
                }
            }
            Expr::Logical { .. } => self.evaluate_logical_expression(expr),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition = self.evaluate(condition)?;

                if self.truthy(&condition) {
                    self.evaluate(then_expr)
                } else {
                    self.evaluate(else_expr)
                }
            }
            Expr::Variable(name) => self.environment.borrow().get(name),
            Expr::Assignment { .. } => self.evaluate_assign_expression(expr),
            Expr::Call { .. } => self.evaluate_call_expression(expr),
            Expr::Lambda { .. } => self.evaluate_lambda_expression(expr),
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
        operator: &Op,
        left: &Value,
        right: &Value,
        operation: T,
    ) -> Result<Value, ControlFlow>
    where
        T: FnOnce(f64, f64) -> Value,
    {
        if let (Value::Number(left), Value::Number(right)) = (left, right) {
            Ok(operation(*left, *right))
        } else {
            Err(ControlFlow::RuntimeError(RuntimeError {
                line: operator.line,
                error_type: ValidationError::InvalidOperandTypes {
                    expected: &[ValueType::Number],
                    actual_left: Some(left.clone()),
                    actual_right: right.clone(),
                },
            }))
        }
    }
}

#[cfg(test)]
mod tests;
