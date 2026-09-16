use crate::ast::{Literal, Param};
use crate::environment::{EnvRef, Environment};
use crate::error::KarmaError;
use crate::token::TokenKind;
use crate::typed_ast::{TypedExpr, TypedExprKind, TypedProgram, TypedStmt, TypedStmtKind};
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct FunctionDef {
    params: Vec<Param>,
    body: Vec<TypedStmt>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLimits {
    pub max_steps: u64,
    pub max_call_depth: usize,
}

impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_steps: 1_000_000,
            max_call_depth: 512,
        }
    }
}

enum Flow {
    Continue,
    Return(Value),
}

pub struct Interpreter {
    globals: EnvRef,
    functions: HashMap<String, FunctionDef>,
    output: Vec<String>,
    limits: RuntimeLimits,
    steps: u64,
    call_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::with_limits(RuntimeLimits::default())
    }

    pub fn with_limits(limits: RuntimeLimits) -> Self {
        Self {
            globals: Environment::root(),
            functions: HashMap::new(),
            output: Vec::new(),
            limits,
            steps: 0,
            call_depth: 0,
        }
    }

    pub fn run(mut self, program: &TypedProgram) -> Result<Vec<String>, KarmaError> {
        self.register_top_level_functions(program)?;
        let env = self.globals.clone();
        for stmt in program {
            if matches!(&stmt.kind, TypedStmtKind::Function { .. }) {
                continue;
            }
            match self.execute(stmt, env.clone())? {
                Flow::Continue => {}
                Flow::Return(_) => {
                    return Err(KarmaError::runtime(
                        "internal invariant: top-level return passed type checking",
                    ))
                }
            }
        }
        Ok(self.output)
    }

    fn register_top_level_functions(&mut self, program: &TypedProgram) -> Result<(), KarmaError> {
        for stmt in program {
            if let TypedStmtKind::Function {
                name, params, body, ..
            } = &stmt.kind
            {
                if name == "print" {
                    return Err(KarmaError::runtime(
                        "'print' is a reserved built-in function",
                    ));
                }
                if self.functions.contains_key(name) {
                    return Err(KarmaError::runtime(format!(
                        "function '{name}' is already defined"
                    )));
                }
                self.functions.insert(
                    name.clone(),
                    FunctionDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &TypedStmt, env: EnvRef) -> Result<Flow, KarmaError> {
        self.tick()?;
        match &stmt.kind {
            TypedStmtKind::Binding {
                name,
                mutable,
                initializer,
                ..
            } => {
                let value = self.evaluate(initializer, env.clone())?;
                env.borrow_mut().define(name.clone(), value, *mutable)?;
                Ok(Flow::Continue)
            }
            TypedStmtKind::Expression(expr) => {
                self.evaluate(expr, env)?;
                Ok(Flow::Continue)
            }
            TypedStmtKind::Block(statements) => {
                let block_env = Environment::child(env);
                self.execute_block(statements, block_env)
            }
            TypedStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate(condition, env.clone())?;
                match condition {
                    Value::Bool(true) => self.execute(then_branch, env),
                    Value::Bool(false) => {
                        if let Some(else_branch) = else_branch {
                            self.execute(else_branch, env)
                        } else {
                            Ok(Flow::Continue)
                        }
                    }
                    _ => Err(KarmaError::runtime(
                        "internal invariant: non-Bool if condition passed type checking",
                    )),
                }
            }
            TypedStmtKind::While { condition, body } => {
                loop {
                    let condition_value = self.evaluate(condition, env.clone())?;
                    match condition_value {
                        Value::Bool(true) => match self.execute(body, env.clone())? {
                            Flow::Continue => {}
                            flow @ Flow::Return(_) => return Ok(flow),
                        },
                        Value::Bool(false) => break,
                        _ => {
                            return Err(KarmaError::runtime(
                                "internal invariant: non-Bool while condition passed type checking",
                            ))
                        }
                    }
                }
                Ok(Flow::Continue)
            }
            TypedStmtKind::Function { .. } => Ok(Flow::Continue),
            TypedStmtKind::Return(value) => {
                let value = match value {
                    Some(expr) => self.evaluate(expr, env)?,
                    None => Value::Unit,
                };
                Ok(Flow::Return(value))
            }
        }
    }

    fn execute_block(&mut self, statements: &[TypedStmt], env: EnvRef) -> Result<Flow, KarmaError> {
        for stmt in statements {
            match self.execute(stmt, env.clone())? {
                Flow::Continue => {}
                flow @ Flow::Return(_) => return Ok(flow),
            }
        }
        Ok(Flow::Continue)
    }

    fn evaluate(&mut self, expr: &TypedExpr, env: EnvRef) -> Result<Value, KarmaError> {
        self.tick()?;
        match &expr.kind {
            TypedExprKind::Literal(literal) => Ok(match literal {
                Literal::Int(v) => Value::Int(*v),
                Literal::Bool(v) => Value::Bool(*v),
                Literal::String(v) => Value::String(v.clone()),
            }),
            TypedExprKind::Variable(name) => env.borrow().get(name),
            TypedExprKind::Assign { name, value } => {
                let value = self.evaluate(value, env.clone())?;
                env.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
            TypedExprKind::Unary { op, right } => {
                let right = self.evaluate(right, env)?;
                self.eval_unary(op, right)
            }
            TypedExprKind::Binary { left, op, right } => {
                let left = self.evaluate(left, env.clone())?;
                let right = self.evaluate(right, env)?;
                self.eval_binary(left, op, right)
            }
            TypedExprKind::Call { callee, arguments } => self.call(callee, arguments, env),
        }
    }

    fn eval_unary(&self, op: &TokenKind, right: Value) -> Result<Value, KarmaError> {
        match op {
            TokenKind::Bang => match right {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                _ => Err(KarmaError::runtime(
                    "internal invariant: non-Bool operand passed type checking for '!'",
                )),
            },
            TokenKind::Minus => match right {
                Value::Int(v) => v
                    .checked_neg()
                    .map(Value::Int)
                    .ok_or_else(|| KarmaError::runtime("integer overflow in unary '-'")),
                _ => Err(KarmaError::runtime(
                    "internal invariant: non-Int operand passed type checking for '-'",
                )),
            },
            _ => Err(KarmaError::runtime("unsupported unary operator")),
        }
    }

    fn eval_binary(&self, left: Value, op: &TokenKind, right: Value) -> Result<Value, KarmaError> {
        use TokenKind::*;
        match op {
            Plus => match (left, right) {
                (Value::Int(a), Value::Int(b)) => a
                    .checked_add(b)
                    .map(Value::Int)
                    .ok_or_else(|| KarmaError::runtime("integer overflow in '+'")),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                _ => Err(KarmaError::runtime(
                    "internal invariant: invalid '+' operands passed type checking",
                )),
            },
            Minus | Star | Slash => self.eval_integer_arithmetic(left, op, right),
            Greater | GreaterEqual | Less | LessEqual => {
                self.eval_integer_comparison(left, op, right)
            }
            EqualEqual => Ok(Value::Bool(left == right)),
            BangEqual => Ok(Value::Bool(left != right)),
            _ => Err(KarmaError::runtime("unsupported binary operator")),
        }
    }

    fn eval_integer_arithmetic(
        &self,
        left: Value,
        op: &TokenKind,
        right: Value,
    ) -> Result<Value, KarmaError> {
        let (a, b) = match (left, right) {
            (Value::Int(a), Value::Int(b)) => (a, b),
            _ => {
                return Err(KarmaError::runtime(
                    "internal invariant: non-Int arithmetic operands passed type checking",
                ))
            }
        };

        let value = match op {
            TokenKind::Minus => a.checked_sub(b),
            TokenKind::Star => a.checked_mul(b),
            TokenKind::Slash => {
                if b == 0 {
                    return Err(KarmaError::runtime("division by zero"));
                }
                a.checked_div(b)
            }
            _ => None,
        };

        value
            .map(Value::Int)
            .ok_or_else(|| KarmaError::runtime("integer overflow in arithmetic operation"))
    }

    fn eval_integer_comparison(
        &self,
        left: Value,
        op: &TokenKind,
        right: Value,
    ) -> Result<Value, KarmaError> {
        let (a, b) = match (left, right) {
            (Value::Int(a), Value::Int(b)) => (a, b),
            _ => {
                return Err(KarmaError::runtime(
                    "internal invariant: non-Int comparison operands passed type checking",
                ))
            }
        };
        let result = match op {
            TokenKind::Greater => a > b,
            TokenKind::GreaterEqual => a >= b,
            TokenKind::Less => a < b,
            TokenKind::LessEqual => a <= b,
            _ => false,
        };
        Ok(Value::Bool(result))
    }

    fn call(
        &mut self,
        callee: &str,
        arguments: &[TypedExpr],
        env: EnvRef,
    ) -> Result<Value, KarmaError> {
        if callee == "print" {
            if arguments.len() != 1 {
                return Err(KarmaError::runtime(format!(
                    "print expects 1 argument, got {}",
                    arguments.len()
                )));
            }
            let value = self.evaluate(&arguments[0], env)?;
            self.output.push(value.to_string());
            return Ok(Value::Unit);
        }

        let function = self
            .functions
            .get(callee)
            .cloned()
            .ok_or_else(|| KarmaError::runtime(format!("undefined function '{callee}'")))?;

        if function.params.len() != arguments.len() {
            return Err(KarmaError::runtime(format!(
                "function '{callee}' expects {} arguments, got {}",
                function.params.len(),
                arguments.len()
            )));
        }

        if self.call_depth >= self.limits.max_call_depth {
            return Err(KarmaError::runtime("maximum call depth exceeded"));
        }

        let mut values = Vec::with_capacity(arguments.len());
        for arg in arguments {
            values.push(self.evaluate(arg, env.clone())?);
        }

        self.call_depth += 1;
        let call_env = Environment::child(self.globals.clone());
        for (param, value) in function.params.iter().zip(values) {
            call_env
                .borrow_mut()
                .define(param.name.clone(), value, false)?;
        }

        let result = match self.execute_block(&function.body, call_env) {
            Ok(Flow::Continue) => Ok(Value::Unit),
            Ok(Flow::Return(value)) => Ok(value),
            Err(err) => Err(err),
        };
        self.call_depth -= 1;
        result
    }

    fn tick(&mut self) -> Result<(), KarmaError> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limits.max_steps {
            return Err(KarmaError::runtime(format!(
                "execution step limit ({}) exceeded",
                self.limits.max_steps
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::type_checker::TypeChecker;

    fn run(source: &str) -> Result<Vec<String>, KarmaError> {
        let tokens = Lexer::new(source).scan_tokens()?;
        let program = Parser::new(tokens).parse()?;
        let typed_program = TypeChecker::new().check(&program)?;
        Interpreter::new().run(&typed_program)
    }

    #[test]
    fn runs_arithmetic() {
        assert_eq!(run("print(2 + 3 * 4);").unwrap(), vec!["14"]);
    }

    #[test]
    fn runs_typed_function_and_recursion() {
        let source = r#"
            fn fact(n: Int) -> Int {
                if n <= 1 { return 1; }
                return n * fact(n - 1);
            }
            print(fact(5));
        "#;
        assert_eq!(run(source).unwrap(), vec!["120"]);
    }

    #[test]
    fn runs_mutable_binding() {
        let source = "mut count: Int = 1; count = count + 1; print(count);";
        assert_eq!(run(source).unwrap(), vec!["2"]);
    }

    #[test]
    fn detects_division_by_zero_at_runtime() {
        let error = run("print(1 / 0);").unwrap_err();
        assert!(error.message.contains("division by zero"));
    }

    #[test]
    fn detects_integer_overflow_at_runtime() {
        let error = run("print(9223372036854775807 + 1);").unwrap_err();
        assert!(error.message.contains("overflow"));
    }
}
