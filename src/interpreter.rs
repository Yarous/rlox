use crate::Lox;
use crate::environment::{EnvRef, Environment};
use crate::expr::{Expr, Op};
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenKind};
use std::fmt::Display;
use std::rc::Rc;

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
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let begin = "Validation Error: ";
        match self {
            ValidationError::DivisionByZero => write!(f, "{}Division by Zero", begin),
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
                        "{} Invalid operand types. Expected: [{}], but got: [{}] and [{}]",
                        begin, expected_str, actual_left, actual_right
                    )
                } else {
                    write!(
                        f,
                        "{} Invalid operand types. Expected: [{}], but got: [{}]",
                        begin, expected_str, actual_right
                    )
                }
            }
            ValidationError::UndefinedVariable(where_undef) => {
                write!(f, "{} Undefined variable: {}", begin, where_undef)
            }
            ValidationError::UninitializedVariable(where_uninit) => {
                write!(f, "{} Uninitialized variable: {}", begin, where_uninit)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeError {
    pub(crate) line: usize,
    pub(crate) error_type: ValidationError,
}

pub(crate) enum ControlFlow {
    RuntimeError(RuntimeError),
    Break,
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
            Literal::Boolean(value) => Value::Boolean(value),
            Literal::Nil => Value::Nil,
        }
    }
}

pub(crate) struct Interpreter<'lox> {
    lox: &'lox mut Lox,
    environment: EnvRef,
}

impl<'lox> Interpreter<'lox> {
    pub(crate) fn new(lox: &'lox mut Lox) -> Self {
        Self {
            lox,
            environment: Environment::new(),
        }
    }

    pub(crate) fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            match self.execute(stmt) {
                Ok(_) => (),
                Err(error) => {
                    if let ControlFlow::RuntimeError(reported_error) = error {
                        self.lox.runtime_error(reported_error)
                    }
                }
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        match stmt {
            Stmt::ExprStmt(_) => self.evaluate_expression_statement(stmt),
            Stmt::Print(_) => self.evaluate_print_statement(stmt),
            Stmt::Var { name, initializer } => self.evaluate_var_statement(name, initializer),
            Stmt::Block(stmts) => {
                let block_env = Environment::add_enclosing(Rc::clone(&self.environment));
                self.evaluate_block_statement(stmts, block_env)
            }
            Stmt::If { .. } => self.evaluate_if_statement(stmt),
            Stmt::While { .. } => self.evaluate_while_statement(stmt),
            Stmt::Break => Err(ControlFlow::Break),
        }
    }

    fn evaluate_while_statement(&mut self, stmt: &Stmt) -> Result<(), ControlFlow> {
        if let Stmt::While { condition, body } = stmt {
            while {
                let while_condition = self.evaluate(condition)?;
                self.truthy(&while_condition)
            } {
                match self.execute(body) {
                    Ok(_) => {}
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

    fn evaluate_block_statement(
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
            };
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

        self.environment.borrow_mut().define(name, value);
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
            Stmt::ExprStmt(expr) => {
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
                println!("{}", value);
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
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        (Value::String(left), right) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
                        (left, Value::String(right)) => {
                            Ok(Value::String(format!("{}{}", left, right)))
                        }
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
