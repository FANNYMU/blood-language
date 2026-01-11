use crate::ast::{Expr, Op, Stmt};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Integer(i64),
    Boolean(bool),
    Nil,
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Integer(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Nil => write!(f, "nil"),
            Value::Function { name, .. } => write!(f, "<fn {}>", name),
        }
    }
}

struct Variable {
    value: Value,
    mutable: bool,
}

#[derive(Clone)]
enum ExecutionResult {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub struct Interpreter {
    globals: HashMap<String, Variable>,

    call_stack: Vec<Vec<HashMap<String, Variable>>>,

    loop_depth: usize,
    function_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            call_stack: vec![vec![HashMap::new()]],
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), String> {
        for stmt in statements {
            match self.execute_stmt(stmt)? {
                ExecutionResult::Normal => {}
                ExecutionResult::Break => {
                    return Err("Runtime error: 'break' used outside of loop".to_string());
                }
                ExecutionResult::Continue => {
                    return Err("Runtime error: 'continue' used outside of loop".to_string());
                }
                ExecutionResult::Return(_) => {
                    return Err("Runtime error: 'return' used outside of function".to_string());
                }
            }
        }
        Ok(())
    }

    fn current_frame_mut(&mut self) -> &mut Vec<HashMap<String, Variable>> {
        self.call_stack.last_mut().unwrap()
    }

    fn current_frame(&self) -> &Vec<HashMap<String, Variable>> {
        self.call_stack.last().unwrap()
    }

    fn enter_scope(&mut self) {
        self.current_frame_mut().push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.current_frame_mut().pop();
    }

    fn define_variable(&mut self, name: String, value: Value, mutable: bool) -> Result<(), String> {
        if self.function_depth == 0 {
            if self.globals.contains_key(&name) {
                return Err(format!(
                    "Runtime Error: Global variable '{}' already declared.",
                    name
                ));
            }
            self.globals.insert(name, Variable { value, mutable });
        } else {
            let current_scope = self.current_frame_mut().last_mut().unwrap();
            if current_scope.contains_key(&name) {
                return Err(format!(
                    "Runtime Error: Variable '{}' already declared in this scope.",
                    name
                ));
            }
            current_scope.insert(name, Variable { value, mutable });
        }
        Ok(())
    }

    fn assign_variable(&mut self, name: &str, value: Value) -> Result<(), String> {
        for scope in self.current_frame_mut().iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                if !var.mutable {
                    return Err(format!(
                        "Runtime Error: Cannot reassign immutable variable '{}'.",
                        name
                    ));
                }
                var.value = value;
                return Ok(());
            }
        }

        if let Some(var) = self.globals.get_mut(name) {
            if !var.mutable {
                return Err(format!(
                    "Runtime Error: Cannot reassign immutable variable '{}'.",
                    name
                ));
            }
            var.value = value;
            return Ok(());
        }

        Err(format!("Runtime Error: Variable '{}' not found.", name))
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        for scope in self.current_frame().iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(var.value.clone());
            }
        }

        if let Some(var) = self.globals.get(name) {
            return Ok(var.value.clone());
        }

        Err(format!("Runtime Error: Variable '{}' not defined.", name))
    }

    fn execute_stmt(&mut self, stmt: Stmt) -> Result<ExecutionResult, String> {
        match stmt {
            Stmt::Let {
                name,
                mutable,
                value,
            } => {
                let val = self.eval_expr(value)?;
                self.define_variable(name, val, mutable)?;
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.assign_variable(&name, val)?;
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr)?;
                println!("{}", val);
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                let cond_bool = match cond_val {
                    Value::Boolean(b) => b,
                    _ => return Err("Runtime error: condition must be boolean".to_string()),
                };

                if cond_bool {
                    self.enter_scope();
                    for s in then_branch {
                        let res = self.execute_stmt(s)?;
                        if !matches!(res, ExecutionResult::Normal) {
                            self.exit_scope();
                            return Ok(res);
                        }
                    }
                    self.exit_scope();
                } else if let Some(else_stmts) = else_branch {
                    self.enter_scope();
                    for s in else_stmts {
                        let res = self.execute_stmt(s)?;
                        if !matches!(res, ExecutionResult::Normal) {
                            self.exit_scope();
                            return Ok(res);
                        }
                    }
                    self.exit_scope();
                }
            }
            Stmt::While { condition, body } => {
                self.loop_depth += 1;
                loop {
                    let cond_val = self.eval_expr(condition.clone())?;
                    let cond_bool = match cond_val {
                        Value::Boolean(b) => b,
                        _ => {
                            return Err(
                                "Runtime error: while condition must be boolean".to_string()
                            );
                        }
                    };

                    if !cond_bool {
                        break;
                    }

                    self.enter_scope();
                    let mut flow_break = false;
                    let mut flow_return = None;

                    for s in &body {
                        match self.execute_stmt(s.clone())? {
                            ExecutionResult::Normal => {}
                            ExecutionResult::Break => {
                                flow_break = true;
                                break;
                            }
                            ExecutionResult::Continue => {
                                break;
                            }
                            ExecutionResult::Return(v) => {
                                flow_return = Some(v);
                                break;
                            }
                        }
                    }
                    self.exit_scope();

                    if let Some(v) = flow_return {
                        self.loop_depth -= 1;
                        return Ok(ExecutionResult::Return(v));
                    }
                    if flow_break {
                        break;
                    }
                }
                self.loop_depth -= 1;
            }
            Stmt::Loop { body } => {
                self.loop_depth += 1;
                loop {
                    self.enter_scope();
                    let mut flow_break = false;
                    let mut flow_return = None;

                    for s in &body {
                        match self.execute_stmt(s.clone())? {
                            ExecutionResult::Normal => {}
                            ExecutionResult::Break => {
                                flow_break = true;
                                break;
                            }
                            ExecutionResult::Continue => {
                                break;
                            }
                            ExecutionResult::Return(v) => {
                                flow_return = Some(v);
                                break;
                            }
                        }
                    }
                    self.exit_scope();

                    if let Some(v) = flow_return {
                        self.loop_depth -= 1;
                        return Ok(ExecutionResult::Return(v));
                    }
                    if flow_break {
                        break;
                    }
                }
                self.loop_depth -= 1;
            }
            Stmt::Break => {
                if self.loop_depth == 0 {
                    return Err("Runtime error: 'break' used outside of loop".to_string());
                }
                return Ok(ExecutionResult::Break);
            }
            Stmt::Continue => {
                if self.loop_depth == 0 {
                    return Err("Runtime error: 'continue' used outside of loop".to_string());
                }
                return Ok(ExecutionResult::Continue);
            }
            Stmt::Fn { name, params, body } => {
                let func = Value::Function {
                    name: name.clone(),
                    params,
                    body,
                };

                self.define_variable(name, func, false)?;
            }
            Stmt::Return(expr) => {
                if self.function_depth == 0 {
                    return Err("Runtime error: 'return' used outside of function".to_string());
                }
                let val = self.eval_expr(expr)?;
                return Ok(ExecutionResult::Return(val));
            }
        }
        Ok(ExecutionResult::Normal)
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(val) => Ok(Value::Integer(val)),
            Expr::Boolean(val) => Ok(Value::Boolean(val)),
            Expr::Nil => Ok(Value::Nil),
            Expr::Variable(name) => self.get_variable(&name),
            Expr::Unary(op, right) => {
                let r = self.eval_expr(*right)?;
                match op {
                    Op::Not => match r {
                        Value::Boolean(b) => Ok(Value::Boolean(!b)),
                        _ => Err("Runtime Error: 'not' expects a boolean.".to_string()),
                    },
                    _ => unreachable!("Unary op not implemented"),
                }
            }
            Expr::Binary(left, op, right) => {
                let l = self.eval_expr(*left)?;
                let r = self.eval_expr(*right)?;

                match op {
                    Op::Add => self.arithmetic(l, r, |a, b| a + b),
                    Op::Sub => self.arithmetic(l, r, |a, b| a - b),
                    Op::Mul => self.arithmetic(l, r, |a, b| a * b),
                    Op::Div => match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 {
                                return Err("Runtime Error: Division by zero.".to_string());
                            }
                            Ok(Value::Integer(a / b))
                        }
                        _ => Err("Runtime Error: Operands must be integers.".to_string()),
                    },
                    Op::Mod => match (l, r) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if b == 0 {
                                return Err("Runtime Error: Modulo by zero.".to_string());
                            }
                            Ok(Value::Integer(a % b))
                        }
                        _ => Err("Runtime Error: Operands must be integers.".to_string()),
                    },

                    Op::Equal => Ok(Value::Boolean(l == r)),
                    Op::NotEqual => Ok(Value::Boolean(l != r)),
                    Op::Lt => self.comparison(l, r, |a, b| a < b),
                    Op::Gt => self.comparison(l, r, |a, b| a > b),
                    Op::LtEq => self.comparison(l, r, |a, b| a <= b),
                    Op::GtEq => self.comparison(l, r, |a, b| a >= b),

                    Op::And => match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
                        _ => Err("Runtime Error: 'and' operands must be booleans.".to_string()),
                    },
                    Op::Or => match (l, r) {
                        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
                        _ => Err("Runtime Error: 'or' operands must be booleans.".to_string()),
                    },
                    _ => unreachable!("Binary op not implemented"),
                }
            }
            Expr::Call(name, args) => {
                let func_val = self.get_variable(&name)?;
                match func_val {
                    Value::Function {
                        name: _,
                        params,
                        body,
                    } => {
                        if args.len() != params.len() {
                            return Err(format!(
                                "Runtime error: expected {} argument, got {}",
                                params.len(),
                                args.len()
                            ));
                        }

                        let mut arg_vals = Vec::new();
                        for arg in args {
                            arg_vals.push(self.eval_expr(arg)?);
                        }

                        let mut new_frame = vec![HashMap::new()];

                        for (param, val) in params.iter().zip(arg_vals.into_iter()) {
                            new_frame[0].insert(
                                param.clone(),
                                Variable {
                                    value: val,
                                    mutable: false,
                                },
                            );
                        }

                        self.call_stack.push(new_frame);
                        self.function_depth += 1;
                        let old_loop_depth = self.loop_depth;
                        self.loop_depth = 0;

                        let mut return_val = Value::Nil;

                        for stmt in body {
                            match self.execute_stmt(stmt)? {
                                ExecutionResult::Return(v) => {
                                    return_val = v;
                                    break;
                                }
                                ExecutionResult::Normal => {}
                                _ => {
                                    // Break/Continue should be caught by execute_stmt validation if loop_depth is 0.
                                }
                            }
                        }

                        self.loop_depth = old_loop_depth;
                        self.function_depth -= 1;
                        self.call_stack.pop();

                        Ok(return_val)
                    }
                    _ => Err(format!("Runtime Error: '{}' is not a function.", name)),
                }
            }
        }
    }

    fn arithmetic<F>(&self, l: Value, r: Value, op: F) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> i64,
    {
        match (l, r) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(op(a, b))),
            _ => Err("Runtime Error: Operands must be integers.".to_string()),
        }
    }

    fn comparison<F>(&self, l: Value, r: Value, op: F) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (l, r) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(op(a, b))),
            _ => Err("Runtime Error: Comparison operands must be integers.".to_string()),
        }
    }
}
