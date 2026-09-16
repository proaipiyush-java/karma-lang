use crate::ast::{Expr, ExprKind, Literal, Param, Stmt, StmtKind};
use crate::error::KarmaError;
use crate::source::SourcePos;
use crate::token::TokenKind;
use crate::typed_ast::{TypedExpr, TypedExprKind, TypedProgram, TypedStmt, TypedStmtKind};
use crate::types::Type;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct VariableSymbol {
    ty: Type,
    mutable: bool,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    params: Vec<Type>,
    return_type: Type,
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, VariableSymbol>>,
    functions: HashMap<String, FunctionSignature>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    pub fn check(mut self, program: &[Stmt]) -> Result<TypedProgram, KarmaError> {
        self.register_function_signatures(program)?;

        let mut typed = Vec::with_capacity(program.len());
        for stmt in program {
            typed.push(self.check_stmt(stmt, None, true)?);
        }
        Ok(typed)
    }

    fn register_function_signatures(&mut self, program: &[Stmt]) -> Result<(), KarmaError> {
        self.functions.insert(
            "print".to_string(),
            FunctionSignature {
                params: Vec::new(), // print is special-cased as polymorphic.
                return_type: Type::Unit,
            },
        );

        for stmt in program {
            if let StmtKind::Function {
                name,
                params,
                return_type,
                ..
            } = &stmt.kind
            {
                if name == "print" {
                    return Err(type_error_at(
                        stmt.pos,
                        "'print' is a reserved built-in function",
                    ));
                }
                if self.functions.contains_key(name) {
                    return Err(type_error_at(
                        stmt.pos,
                        format!("function '{name}' is already defined"),
                    ));
                }
                self.functions.insert(
                    name.clone(),
                    FunctionSignature {
                        params: params.iter().map(|p| p.ty.clone()).collect(),
                        return_type: return_type.clone(),
                    },
                );
            }
        }
        Ok(())
    }

    fn check_stmt(
        &mut self,
        stmt: &Stmt,
        current_return: Option<&Type>,
        allow_function: bool,
    ) -> Result<TypedStmt, KarmaError> {
        let kind = match &stmt.kind {
            StmtKind::Binding {
                name,
                mutable,
                annotation,
                initializer,
            } => {
                let typed_initializer = self.check_expr(initializer)?;
                let resolved_type = if let Some(annotation) = annotation {
                    self.require_same_type(
                        annotation,
                        &typed_initializer.ty,
                        initializer.pos,
                        format!("initializer for '{name}'"),
                    )?;
                    annotation.clone()
                } else {
                    typed_initializer.ty.clone()
                };

                self.define_variable(
                    name,
                    VariableSymbol {
                        ty: resolved_type.clone(),
                        mutable: *mutable,
                    },
                    stmt.pos,
                )?;

                TypedStmtKind::Binding {
                    name: name.clone(),
                    mutable: *mutable,
                    ty: resolved_type,
                    initializer: typed_initializer,
                }
            }
            StmtKind::Expression(expr) => TypedStmtKind::Expression(self.check_expr(expr)?),
            StmtKind::Block(statements) => {
                self.push_scope();
                let result = (|| {
                    let mut typed = Vec::with_capacity(statements.len());
                    for statement in statements {
                        typed.push(self.check_stmt(statement, current_return, false)?);
                    }
                    Ok(TypedStmtKind::Block(typed))
                })();
                self.pop_scope();
                result?
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let typed_condition = self.check_expr(condition)?;
                self.require_type(
                    &typed_condition.ty,
                    &Type::Bool,
                    condition.pos,
                    "if condition",
                )?;
                let typed_then = Box::new(self.check_stmt(then_branch, current_return, false)?);
                let typed_else = match else_branch {
                    Some(branch) => {
                        Some(Box::new(self.check_stmt(branch, current_return, false)?))
                    }
                    None => None,
                };
                TypedStmtKind::If {
                    condition: typed_condition,
                    then_branch: typed_then,
                    else_branch: typed_else,
                }
            }
            StmtKind::While { condition, body } => {
                let typed_condition = self.check_expr(condition)?;
                self.require_type(
                    &typed_condition.ty,
                    &Type::Bool,
                    condition.pos,
                    "while condition",
                )?;
                let typed_body = Box::new(self.check_stmt(body, current_return, false)?);
                TypedStmtKind::While {
                    condition: typed_condition,
                    body: typed_body,
                }
            }
            StmtKind::Function {
                name,
                params,
                return_type,
                body,
            } => {
                if !allow_function || current_return.is_some() {
                    return Err(type_error_at(
                        stmt.pos,
                        "functions must be declared at top level in Karma v0.2",
                    ));
                }
                self.check_function(name, params, return_type, body, stmt.pos)?
            }
            StmtKind::Return(value) => {
                let expected = current_return.ok_or_else(|| {
                    type_error_at(stmt.pos, "'return' can only be used inside a function")
                })?;
                match value {
                    Some(expr) => {
                        let typed_expr = self.check_expr(expr)?;
                        self.require_same_type(expected, &typed_expr.ty, expr.pos, "return value")?;
                        TypedStmtKind::Return(Some(typed_expr))
                    }
                    None => {
                        self.require_type(expected, &Type::Unit, stmt.pos, "empty return")?;
                        TypedStmtKind::Return(None)
                    }
                }
            }
        };

        Ok(TypedStmt::new(kind, stmt.pos))
    }

    fn check_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: &Type,
        body: &[Stmt],
        pos: SourcePos,
    ) -> Result<TypedStmtKind, KarmaError> {
        self.push_scope();
        let result = (|| {
            for param in params {
                self.define_variable(
                    &param.name,
                    VariableSymbol {
                        ty: param.ty.clone(),
                        mutable: false,
                    },
                    param.pos,
                )?;
            }

            let mut typed_body = Vec::with_capacity(body.len());
            for statement in body {
                typed_body.push(self.check_stmt(statement, Some(return_type), false)?);
            }

            if *return_type != Type::Unit && !statements_guarantee_return(&typed_body) {
                return Err(type_error_at(
                    pos,
                    format!(
                        "function '{name}' returns {return_type} but not every control-flow path returns a value"
                    ),
                ));
            }

            Ok(TypedStmtKind::Function {
                name: name.to_string(),
                params: params.to_vec(),
                return_type: return_type.clone(),
                body: typed_body,
            })
        })();
        self.pop_scope();
        result
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<TypedExpr, KarmaError> {
        let (kind, ty) = match &expr.kind {
            ExprKind::Literal(literal) => {
                let ty = match literal {
                    Literal::Int(_) => Type::Int,
                    Literal::Bool(_) => Type::Bool,
                    Literal::String(_) => Type::String,
                };
                (TypedExprKind::Literal(literal.clone()), ty)
            }
            ExprKind::Variable(name) => {
                let symbol = self.lookup_variable(name).ok_or_else(|| {
                    type_error_at(expr.pos, format!("undefined variable '{name}'"))
                })?;
                (TypedExprKind::Variable(name.clone()), symbol.ty.clone())
            }
            ExprKind::Assign { name, value } => {
                let symbol = self.lookup_variable(name).cloned().ok_or_else(|| {
                    type_error_at(expr.pos, format!("undefined variable '{name}'"))
                })?;
                if !symbol.mutable {
                    return Err(type_error_at(
                        expr.pos,
                        format!(
                            "cannot assign to immutable binding '{name}'; declare it with 'mut' if mutation is required"
                        ),
                    ));
                }
                let typed_value = self.check_expr(value)?;
                self.require_same_type(
                    &symbol.ty,
                    &typed_value.ty,
                    value.pos,
                    format!("assignment to '{name}'"),
                )?;
                (
                    TypedExprKind::Assign {
                        name: name.clone(),
                        value: Box::new(typed_value),
                    },
                    symbol.ty,
                )
            }
            ExprKind::Unary { op, right } => {
                let typed_right = self.check_expr(right)?;
                let result_type = match op {
                    TokenKind::Bang => {
                        self.require_type(
                            &typed_right.ty,
                            &Type::Bool,
                            right.pos,
                            "unary '!' operand",
                        )?;
                        Type::Bool
                    }
                    TokenKind::Minus => {
                        self.require_type(
                            &typed_right.ty,
                            &Type::Int,
                            right.pos,
                            "unary '-' operand",
                        )?;
                        Type::Int
                    }
                    _ => {
                        return Err(type_error_at(expr.pos, "unsupported unary operator"));
                    }
                };
                (
                    TypedExprKind::Unary {
                        op: op.clone(),
                        right: Box::new(typed_right),
                    },
                    result_type,
                )
            }
            ExprKind::Binary { left, op, right } => {
                let typed_left = self.check_expr(left)?;
                let typed_right = self.check_expr(right)?;
                let result_type =
                    self.binary_result_type(op, &typed_left, &typed_right, expr.pos)?;
                (
                    TypedExprKind::Binary {
                        left: Box::new(typed_left),
                        op: op.clone(),
                        right: Box::new(typed_right),
                    },
                    result_type,
                )
            }
            ExprKind::Call { callee, arguments } => {
                let mut typed_arguments = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    typed_arguments.push(self.check_expr(argument)?);
                }

                if callee == "print" {
                    if typed_arguments.len() != 1 {
                        return Err(type_error_at(
                            expr.pos,
                            format!("print expects 1 argument, got {}", typed_arguments.len()),
                        ));
                    }
                    (
                        TypedExprKind::Call {
                            callee: callee.clone(),
                            arguments: typed_arguments,
                        },
                        Type::Unit,
                    )
                } else {
                    let signature = self.functions.get(callee).cloned().ok_or_else(|| {
                        type_error_at(expr.pos, format!("undefined function '{callee}'"))
                    })?;

                    if signature.params.len() != typed_arguments.len() {
                        return Err(type_error_at(
                            expr.pos,
                            format!(
                                "function '{callee}' expects {} arguments, got {}",
                                signature.params.len(),
                                typed_arguments.len()
                            ),
                        ));
                    }

                    for (index, (expected, actual)) in signature
                        .params
                        .iter()
                        .zip(typed_arguments.iter())
                        .enumerate()
                    {
                        if expected != &actual.ty {
                            return Err(type_error_at(
                                actual.pos,
                                format!(
                                    "argument {} of '{callee}' expects {expected}, found {}",
                                    index + 1,
                                    actual.ty
                                ),
                            ));
                        }
                    }

                    (
                        TypedExprKind::Call {
                            callee: callee.clone(),
                            arguments: typed_arguments,
                        },
                        signature.return_type,
                    )
                }
            }
        };

        Ok(TypedExpr::new(kind, ty, expr.pos))
    }

    fn binary_result_type(
        &self,
        op: &TokenKind,
        left: &TypedExpr,
        right: &TypedExpr,
        pos: SourcePos,
    ) -> Result<Type, KarmaError> {
        use TokenKind::*;
        match op {
            Plus => {
                if left.ty == Type::Int && right.ty == Type::Int {
                    Ok(Type::Int)
                } else if left.ty == Type::String && right.ty == Type::String {
                    Ok(Type::String)
                } else {
                    Err(type_error_at(
                        pos,
                        format!(
                            "'+' requires Int + Int or String + String, found {} + {}",
                            left.ty, right.ty
                        ),
                    ))
                }
            }
            Minus | Star | Slash => {
                self.require_type(&left.ty, &Type::Int, left.pos, "left arithmetic operand")?;
                self.require_type(&right.ty, &Type::Int, right.pos, "right arithmetic operand")?;
                Ok(Type::Int)
            }
            Greater | GreaterEqual | Less | LessEqual => {
                self.require_type(&left.ty, &Type::Int, left.pos, "left comparison operand")?;
                self.require_type(&right.ty, &Type::Int, right.pos, "right comparison operand")?;
                Ok(Type::Bool)
            }
            EqualEqual | BangEqual => {
                self.require_same_type(&left.ty, &right.ty, pos, "equality comparison")?;
                Ok(Type::Bool)
            }
            _ => Err(type_error_at(pos, "unsupported binary operator")),
        }
    }

    fn define_variable(
        &mut self,
        name: &str,
        symbol: VariableSymbol,
        pos: SourcePos,
    ) -> Result<(), KarmaError> {
        let scope = self
            .scopes
            .last_mut()
            .expect("type checker always has a scope");
        if scope.contains_key(name) {
            return Err(type_error_at(
                pos,
                format!("variable '{name}' is already defined in this scope"),
            ));
        }
        scope.insert(name.to_string(), symbol);
        Ok(())
    }

    fn lookup_variable(&self, name: &str) -> Option<&VariableSymbol> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn require_type(
        &self,
        actual: &Type,
        expected: &Type,
        pos: SourcePos,
        context: &str,
    ) -> Result<(), KarmaError> {
        if actual == expected {
            Ok(())
        } else {
            Err(type_error_at(
                pos,
                format!("{context} expects {expected}, found {actual}"),
            ))
        }
    }

    fn require_same_type(
        &self,
        expected: &Type,
        actual: &Type,
        pos: SourcePos,
        context: impl AsRef<str>,
    ) -> Result<(), KarmaError> {
        if expected == actual {
            Ok(())
        } else {
            Err(type_error_at(
                pos,
                format!("{} expects {expected}, found {actual}", context.as_ref()),
            ))
        }
    }
}

fn statements_guarantee_return(statements: &[TypedStmt]) -> bool {
    for stmt in statements {
        if statement_guarantees_return(stmt) {
            return true;
        }
    }
    false
}

fn statement_guarantees_return(stmt: &TypedStmt) -> bool {
    match &stmt.kind {
        TypedStmtKind::Return(_) => true,
        TypedStmtKind::Block(statements) => statements_guarantee_return(statements),
        TypedStmtKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            statement_guarantees_return(then_branch)
                && else_branch
                    .as_deref()
                    .is_some_and(statement_guarantees_return)
        }
        _ => false,
    }
}

fn type_error_at(pos: SourcePos, message: impl Into<String>) -> KarmaError {
    KarmaError::new("type", message, pos.line, pos.column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check(source: &str) -> Result<TypedProgram, KarmaError> {
        let tokens = Lexer::new(source).scan_tokens()?;
        let program = Parser::new(tokens).parse()?;
        TypeChecker::new().check(&program)
    }

    #[test]
    fn accepts_explicit_and_inferred_types() {
        check("let age: Int = 38; let name = \"Karma\";").unwrap();
    }

    #[test]
    fn rejects_binding_type_mismatch() {
        let error = check("let age: Int = \"thirty eight\";").unwrap_err();
        assert_eq!(error.phase, "type");
        assert!(error.message.contains("expects Int, found String"));
    }

    #[test]
    fn immutable_is_default() {
        let error = check("let count: Int = 0; count = 1;").unwrap_err();
        assert!(error.message.contains("immutable binding 'count'"));
    }

    #[test]
    fn mut_allows_same_type_assignment() {
        check("mut count: Int = 0; count = count + 1;").unwrap();
    }

    #[test]
    fn mut_still_enforces_type() {
        let error = check("mut count: Int = 0; count = \"one\";").unwrap_err();
        assert!(error.message.contains("expects Int, found String"));
    }

    #[test]
    fn condition_must_be_bool() {
        let error = check("if 1 { print(1); }").unwrap_err();
        assert!(error.message.contains("if condition expects Bool"));
    }

    #[test]
    fn checks_function_argument_types() {
        let source = r#"
            fn add(a: Int, b: Int) -> Int { return a + b; }
            print(add(1, "two"));
        "#;
        let error = check(source).unwrap_err();
        assert!(error
            .message
            .contains("argument 2 of 'add' expects Int, found String"));
    }

    #[test]
    fn checks_function_return_type() {
        let source = "fn answer() -> Int { return \"wrong\"; }";
        let error = check(source).unwrap_err();
        assert!(error
            .message
            .contains("return value expects Int, found String"));
    }

    #[test]
    fn requires_return_on_all_paths() {
        let source = "fn f(ok: Bool) -> Int { if ok { return 1; } }";
        let error = check(source).unwrap_err();
        assert!(error
            .message
            .contains("not every control-flow path returns"));
    }

    #[test]
    fn supports_recursive_typed_function() {
        let source = r#"
            fn fact(n: Int) -> Int {
                if n <= 1 { return 1; }
                return n * fact(n - 1);
            }
            print(fact(5));
        "#;
        check(source).unwrap();
    }
}
