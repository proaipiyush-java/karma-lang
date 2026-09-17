use crate::ast::{Expr, ExprKind, Literal, Param, Stmt, StmtKind};
use crate::error::KarmaError;
use crate::source::SourcePos;
use crate::token::TokenKind;
use crate::typed_ast::{
    TypedExpr, TypedExprKind, TypedProgram, TypedStmt, TypedStmtKind, ValueAccess,
};
use crate::types::{ParamMode, Type};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MoveState {
    Available,
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageMode {
    Owned,
    Borrowed,
}

#[derive(Debug, Clone)]
struct VariableSymbol {
    ty: Type,
    mutable: bool,
    storage: StorageMode,
    state: MoveState,
    moved_at: Option<SourcePos>,
}

#[derive(Debug, Clone)]
struct ParameterSignature {
    ty: Type,
    mode: ParamMode,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    params: Vec<ParameterSignature>,
    return_type: Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprUse {
    /// The expression's value is consumed by the surrounding operation.
    Value,
    /// The surrounding operation only needs read-only access for its duration.
    Borrow,
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
        for stmt in program {
            if let StmtKind::Function {
                name,
                params,
                return_type,
                ..
            } = &stmt.kind
            {
                if matches!(name.as_str(), "print" | "clone" | "drop") {
                    return Err(type_error_at(
                        stmt.pos,
                        format!("'{name}' is a reserved built-in function"),
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
                        params: params
                            .iter()
                            .map(|p| ParameterSignature {
                                ty: p.ty.clone(),
                                mode: p.mode,
                            })
                            .collect(),
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
                let typed_initializer = self.check_expr_with_use(initializer, ExprUse::Value)?;
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
                        storage: StorageMode::Owned,
                        state: MoveState::Available,
                        moved_at: None,
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
            StmtKind::Expression(expr) => {
                TypedStmtKind::Expression(self.check_expr_with_use(expr, ExprUse::Value)?)
            }
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
                let typed_condition = self.check_expr_with_use(condition, ExprUse::Borrow)?;
                self.require_type(
                    &typed_condition.ty,
                    &Type::Bool,
                    condition.pos,
                    "if condition",
                )?;

                let baseline = self.scopes.clone();

                self.scopes = baseline.clone();
                let typed_then = Box::new(self.check_stmt(then_branch, current_return, false)?);
                let then_scopes = self.scopes.clone();

                self.scopes = baseline.clone();
                let (typed_else, else_scopes) = match else_branch {
                    Some(branch) => {
                        let typed = Box::new(self.check_stmt(branch, current_return, false)?);
                        (Some(typed), self.scopes.clone())
                    }
                    None => (None, baseline.clone()),
                };

                self.scopes = merge_control_flow(&baseline, &then_scopes, &else_scopes);

                TypedStmtKind::If {
                    condition: typed_condition,
                    then_branch: typed_then,
                    else_branch: typed_else,
                }
            }
            StmtKind::While { condition, body } => {
                let typed_condition = self.check_expr_with_use(condition, ExprUse::Borrow)?;
                self.require_type(
                    &typed_condition.ty,
                    &Type::Bool,
                    condition.pos,
                    "while condition",
                )?;

                let baseline = self.scopes.clone();
                self.scopes = baseline.clone();
                let typed_body = Box::new(self.check_stmt(body, current_return, false)?);
                let after_body = self.scopes.clone();

                if let Some((name, moved_at)) = first_loop_carried_move(&baseline, &after_body) {
                    self.scopes = baseline;
                    return Err(type_error_at(
                        moved_at.unwrap_or(stmt.pos),
                        format!(
                            "loop body moves outer owned value '{name}' without definitely reinitializing it; a later iteration could use a moved value"
                        ),
                    ));
                }

                // The loop may run zero times. Any safe move+reinitialize sequence in
                // the body therefore does not change ownership availability after it.
                self.scopes = baseline;

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
                        "functions must be declared at top level in Karma v0.3",
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
                        let typed_expr = self.check_expr_with_use(expr, ExprUse::Value)?;
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
        // v0.3 deliberately isolates function ownership analysis from top-level
        // runtime bindings. Module constants/statics will get explicit semantics
        // later instead of being accidental captures.
        let outer_scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);

        let result = (|| {
            for param in params {
                self.define_variable(
                    &param.name,
                    VariableSymbol {
                        ty: param.ty.clone(),
                        mutable: false,
                        storage: match param.mode {
                            ParamMode::Owned => StorageMode::Owned,
                            ParamMode::Borrowed => StorageMode::Borrowed,
                        },
                        state: MoveState::Available,
                        moved_at: None,
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

        self.scopes = outer_scopes;
        result
    }

    fn check_expr_with_use(
        &mut self,
        expr: &Expr,
        use_mode: ExprUse,
    ) -> Result<TypedExpr, KarmaError> {
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
                let symbol = self.lookup_variable(name).cloned().ok_or_else(|| {
                    type_error_at(expr.pos, format!("undefined variable '{name}'"))
                })?;

                if symbol.state == MoveState::Moved && symbol.ty.is_owned() {
                    let previous = symbol
                        .moved_at
                        .map(|p| format!(" at {}:{}", p.line, p.column))
                        .unwrap_or_default();
                    return Err(type_error_at(
                        expr.pos,
                        format!(
                            "use of moved value '{name}'; ownership was previously transferred{previous}"
                        ),
                    ));
                }

                let access = if symbol.ty.is_copy() {
                    ValueAccess::Copy
                } else {
                    match use_mode {
                        ExprUse::Borrow => ValueAccess::Borrow,
                        ExprUse::Value => {
                            if symbol.storage == StorageMode::Borrowed {
                                return Err(type_error_at(
                                    expr.pos,
                                    format!(
                                        "cannot move out of borrowed parameter '{name}'; use it read-only or clone({name})"
                                    ),
                                ));
                            }
                            self.mark_moved(name, expr.pos)?;
                            ValueAccess::Move
                        }
                    }
                };

                (
                    TypedExprKind::Variable {
                        name: name.clone(),
                        access,
                    },
                    symbol.ty,
                )
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
                if symbol.storage == StorageMode::Borrowed {
                    return Err(type_error_at(
                        expr.pos,
                        format!("cannot assign to borrowed parameter '{name}'"),
                    ));
                }

                let typed_value = self.check_expr_with_use(value, ExprUse::Value)?;
                self.require_same_type(
                    &symbol.ty,
                    &typed_value.ty,
                    value.pos,
                    format!("assignment to '{name}'"),
                )?;
                self.mark_available(name)?;

                // Assignment stores the new owner in the target binding. It does
                // not also yield another owned copy of the assigned value.
                (
                    TypedExprKind::Assign {
                        name: name.clone(),
                        value: Box::new(typed_value),
                    },
                    Type::Unit,
                )
            }
            ExprKind::Unary { op, right } => {
                let typed_right = self.check_expr_with_use(right, ExprUse::Borrow)?;
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
                    _ => return Err(type_error_at(expr.pos, "unsupported unary operator")),
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
                use TokenKind::*;
                let operand_use = match op {
                    EqualEqual | BangEqual | Greater | GreaterEqual | Less | LessEqual => {
                        ExprUse::Borrow
                    }
                    Plus | Minus | Star | Slash => ExprUse::Value,
                    _ => return Err(type_error_at(expr.pos, "unsupported binary operator")),
                };

                let typed_left = self.check_expr_with_use(left, operand_use)?;
                let typed_right = self.check_expr_with_use(right, operand_use)?;
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
                return self.check_call(expr.pos, callee, arguments);
            }
        };

        Ok(TypedExpr::new(kind, ty, expr.pos))
    }

    fn check_call(
        &mut self,
        pos: SourcePos,
        callee: &str,
        arguments: &[Expr],
    ) -> Result<TypedExpr, KarmaError> {
        if callee == "print" {
            self.require_arity(callee, arguments, 1, pos)?;
            let argument = self.check_expr_with_use(&arguments[0], ExprUse::Borrow)?;
            return Ok(TypedExpr::new(
                TypedExprKind::Call {
                    callee: callee.to_string(),
                    arguments: vec![argument],
                },
                Type::Unit,
                pos,
            ));
        }

        if callee == "clone" {
            self.require_arity(callee, arguments, 1, pos)?;
            let argument = self.check_expr_with_use(&arguments[0], ExprUse::Borrow)?;
            let result_type = argument.ty.clone();
            return Ok(TypedExpr::new(
                TypedExprKind::Call {
                    callee: callee.to_string(),
                    arguments: vec![argument],
                },
                result_type,
                pos,
            ));
        }

        if callee == "drop" {
            self.require_arity(callee, arguments, 1, pos)?;
            let argument = self.check_expr_with_use(&arguments[0], ExprUse::Value)?;
            return Ok(TypedExpr::new(
                TypedExprKind::Call {
                    callee: callee.to_string(),
                    arguments: vec![argument],
                },
                Type::Unit,
                pos,
            ));
        }

        let signature = self
            .functions
            .get(callee)
            .cloned()
            .ok_or_else(|| type_error_at(pos, format!("undefined function '{callee}'")))?;

        if signature.params.len() != arguments.len() {
            return Err(type_error_at(
                pos,
                format!(
                    "function '{callee}' expects {} arguments, got {}",
                    signature.params.len(),
                    arguments.len()
                ),
            ));
        }

        let mut typed_arguments = Vec::with_capacity(arguments.len());
        for (index, (parameter, argument)) in
            signature.params.iter().zip(arguments.iter()).enumerate()
        {
            let use_mode = match parameter.mode {
                ParamMode::Owned => ExprUse::Value,
                ParamMode::Borrowed => ExprUse::Borrow,
            };
            let typed_argument = self.check_expr_with_use(argument, use_mode)?;
            if parameter.ty != typed_argument.ty {
                return Err(type_error_at(
                    typed_argument.pos,
                    format!(
                        "argument {} of '{callee}' expects {} {}, found {}",
                        index + 1,
                        parameter.mode,
                        parameter.ty,
                        typed_argument.ty
                    ),
                ));
            }
            typed_arguments.push(typed_argument);
        }

        Ok(TypedExpr::new(
            TypedExprKind::Call {
                callee: callee.to_string(),
                arguments: typed_arguments,
            },
            signature.return_type,
            pos,
        ))
    }

    fn require_arity(
        &self,
        callee: &str,
        arguments: &[Expr],
        expected: usize,
        pos: SourcePos,
    ) -> Result<(), KarmaError> {
        if arguments.len() == expected {
            Ok(())
        } else {
            Err(type_error_at(
                pos,
                format!(
                    "{callee} expects {expected} argument{}, got {}",
                    if expected == 1 { "" } else { "s" },
                    arguments.len()
                ),
            ))
        }
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

    fn lookup_variable_mut(&mut self, name: &str) -> Option<&mut VariableSymbol> {
        self.scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
    }

    fn mark_moved(&mut self, name: &str, pos: SourcePos) -> Result<(), KarmaError> {
        let symbol = self
            .lookup_variable_mut(name)
            .ok_or_else(|| type_error_at(pos, format!("undefined variable '{name}'")))?;
        symbol.state = MoveState::Moved;
        symbol.moved_at = Some(pos);
        Ok(())
    }

    fn mark_available(&mut self, name: &str) -> Result<(), KarmaError> {
        let symbol = self.lookup_variable_mut(name).ok_or_else(|| {
            type_error_at(SourcePos::default(), format!("undefined variable '{name}'"))
        })?;
        symbol.state = MoveState::Available;
        symbol.moved_at = None;
        Ok(())
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

fn merge_control_flow(
    baseline: &[HashMap<String, VariableSymbol>],
    left: &[HashMap<String, VariableSymbol>],
    right: &[HashMap<String, VariableSymbol>],
) -> Vec<HashMap<String, VariableSymbol>> {
    let mut merged = baseline.to_vec();

    for (scope_index, scope) in merged.iter_mut().enumerate() {
        for (name, symbol) in scope.iter_mut() {
            if symbol.ty.is_copy() {
                continue;
            }

            let fallback = symbol.clone();
            let left_symbol = left
                .get(scope_index)
                .and_then(|s| s.get(name))
                .cloned()
                .unwrap_or_else(|| fallback.clone());
            let right_symbol = right
                .get(scope_index)
                .and_then(|s| s.get(name))
                .cloned()
                .unwrap_or(fallback);

            if left_symbol.state == MoveState::Available
                && right_symbol.state == MoveState::Available
            {
                symbol.state = MoveState::Available;
                symbol.moved_at = None;
            } else {
                symbol.state = MoveState::Moved;
                symbol.moved_at = left_symbol.moved_at.or(right_symbol.moved_at);
            }
        }
    }

    merged
}

fn first_loop_carried_move(
    before: &[HashMap<String, VariableSymbol>],
    after: &[HashMap<String, VariableSymbol>],
) -> Option<(String, Option<SourcePos>)> {
    for (scope_index, scope) in before.iter().enumerate() {
        for (name, symbol) in scope {
            if symbol.ty.is_copy() || symbol.storage == StorageMode::Borrowed {
                continue;
            }
            let Some(after_symbol) = after.get(scope_index).and_then(|s| s.get(name)) else {
                continue;
            };
            if symbol.state == MoveState::Available && after_symbol.state == MoveState::Moved {
                return Some((name.clone(), after_symbol.moved_at));
            }
        }
    }
    None
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
    fn condition_must_be_bool() {
        let error = check("if 1 { print(1); }").unwrap_err();
        assert!(error.message.contains("if condition expects Bool"));
    }

    #[test]
    fn copy_value_does_not_move() {
        check("let a: Int = 10; let b: Int = a; print(a); print(b);").unwrap();
    }

    #[test]
    fn owned_string_moves_between_bindings() {
        let error =
            check("let first: String = \"Karma\"; let second: String = first; print(first);")
                .unwrap_err();
        assert!(error.message.contains("use of moved value 'first'"));
    }

    #[test]
    fn print_borrows_instead_of_moving() {
        check("let name: String = \"Karma\"; print(name); print(name);").unwrap();
    }

    #[test]
    fn owned_parameter_consumes_string() {
        let source = r#"
            fn consume(text: String) -> Unit { print(text); }
            let name: String = "Karma";
            consume(name);
            print(name);
        "#;
        let error = check(source).unwrap_err();
        assert!(error.message.contains("use of moved value 'name'"));
    }

    #[test]
    fn borrowed_parameter_preserves_owner() {
        let source = r#"
            fn show(text: borrow String) -> Unit { print(text); }
            let name: String = "Karma";
            show(name);
            print(name);
        "#;
        check(source).unwrap();
    }

    #[test]
    fn borrowed_parameter_cannot_escape_as_owned_value() {
        let source = r#"
            fn steal(text: borrow String) -> String { return text; }
        "#;
        let error = check(source).unwrap_err();
        assert!(error
            .message
            .contains("cannot move out of borrowed parameter 'text'"));
    }

    #[test]
    fn clone_creates_another_logical_owner() {
        check("let a: String = \"Karma\"; let b: String = clone(a); print(a); print(b);").unwrap();
    }

    #[test]
    fn drop_consumes_owned_value() {
        let error = check("let a: String = \"Karma\"; drop(a); print(a);").unwrap_err();
        assert!(error.message.contains("use of moved value 'a'"));
    }

    #[test]
    fn mutable_binding_can_be_reinitialized_after_move() {
        check("mut a: String = \"one\"; let b: String = a; a = \"two\"; print(a); print(b);")
            .unwrap();
    }

    #[test]
    fn conditional_move_is_unavailable_after_if() {
        let source = r#"
            let enabled: Bool = true;
            let value: String = "Karma";
            if enabled { drop(value); }
            print(value);
        "#;
        let error = check(source).unwrap_err();
        assert!(error.message.contains("use of moved value 'value'"));
    }

    #[test]
    fn loop_cannot_carry_a_moved_outer_owner() {
        let source = r#"
            let value: String = "Karma";
            while false { drop(value); }
        "#;
        let error = check(source).unwrap_err();
        assert!(error
            .message
            .contains("loop body moves outer owned value 'value'"));
    }

    #[test]
    fn loop_move_then_reinitialize_is_safe_for_mut_binding() {
        let source = r#"
            mut value: String = "Karma";
            while false {
                drop(value);
                value = "Again";
            }
            print(value);
        "#;
        check(source).unwrap();
    }

    #[test]
    fn still_checks_function_argument_types() {
        let source = r#"
            fn add(a: Int, b: Int) -> Int { return a + b; }
            print(add(1, "two"));
        "#;
        let error = check(source).unwrap_err();
        assert!(error.message.contains("argument 2 of 'add'"));
        assert!(error.message.contains("found String"));
    }

    #[test]
    fn still_checks_function_return_types() {
        let error = check(r#"fn answer() -> Int { return "wrong"; }"#).unwrap_err();
        assert!(error
            .message
            .contains("return value expects Int, found String"));
    }

    #[test]
    fn string_equality_borrows_both_values() {
        check(
            r#"let a: String = "Karma"; let b: String = "Karma"; let same: Bool = a == b; print(a); print(b); print(same);"#,
        )
        .unwrap();
    }

    #[test]
    fn string_concatenation_consumes_owned_operands() {
        let error = check(
            r#"let a: String = "Kar"; let b: String = "ma"; let c: String = a + b; print(a); print(c);"#,
        )
        .unwrap_err();
        assert!(error.message.contains("use of moved value 'a'"));
    }

    #[test]
    fn functions_do_not_capture_top_level_runtime_bindings() {
        let source = r#"
            let global: String = "Karma";
            fn show() -> Unit { print(global); }
        "#;
        let error = check(source).unwrap_err();
        assert!(error.message.contains("undefined variable 'global'"));
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
