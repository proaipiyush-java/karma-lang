use crate::ast::{Literal, Param};
use crate::source::SourcePos;
use crate::token::TokenKind;
use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub pos: SourcePos,
}

impl TypedExpr {
    pub fn new(kind: TypedExprKind, ty: Type, pos: SourcePos) -> Self {
        Self { kind, ty, pos }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExprKind {
    Literal(Literal),
    Variable(String),
    Assign {
        name: String,
        value: Box<TypedExpr>,
    },
    Unary {
        op: TokenKind,
        right: Box<TypedExpr>,
    },
    Binary {
        left: Box<TypedExpr>,
        op: TokenKind,
        right: Box<TypedExpr>,
    },
    Call {
        callee: String,
        arguments: Vec<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedStmt {
    pub kind: TypedStmtKind,
    pub pos: SourcePos,
}

impl TypedStmt {
    pub fn new(kind: TypedStmtKind, pos: SourcePos) -> Self {
        Self { kind, pos }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStmtKind {
    Binding {
        name: String,
        mutable: bool,
        ty: Type,
        initializer: TypedExpr,
    },
    Expression(TypedExpr),
    Block(Vec<TypedStmt>),
    If {
        condition: TypedExpr,
        then_branch: Box<TypedStmt>,
        else_branch: Option<Box<TypedStmt>>,
    },
    While {
        condition: TypedExpr,
        body: Box<TypedStmt>,
    },
    Function {
        name: String,
        params: Vec<Param>,
        return_type: Type,
        body: Vec<TypedStmt>,
    },
    Return(Option<TypedExpr>),
}

pub type TypedProgram = Vec<TypedStmt>;
