use crate::ast::{Expr, ExprKind, Literal, Param, Stmt, StmtKind};
use crate::error::KarmaError;
use crate::source::SourcePos;
use crate::token::{Token, TokenKind};
use crate::types::Type;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>, KarmaError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, KarmaError> {
        if self.match_simple(&TokenKind::Let) {
            let pos = self.previous().pos();
            self.binding_declaration(false, pos)
        } else if self.match_simple(&TokenKind::Mut) {
            let pos = self.previous().pos();
            self.binding_declaration(true, pos)
        } else if self.match_simple(&TokenKind::Fn) {
            let pos = self.previous().pos();
            self.function_declaration(pos)
        } else {
            self.statement()
        }
    }

    fn binding_declaration(&mut self, mutable: bool, pos: SourcePos) -> Result<Stmt, KarmaError> {
        let name = self.consume_identifier("expected variable name after binding keyword")?;
        let annotation = if self.match_simple(&TokenKind::Colon) {
            Some(self.consume_type("expected type after ':'")?)
        } else {
            None
        };
        self.consume_simple(&TokenKind::Equal, "expected '=' after variable declaration")?;
        let initializer = self.expression()?;
        self.consume_simple(
            &TokenKind::Semicolon,
            "expected ';' after variable declaration",
        )?;
        Ok(Stmt::new(
            StmtKind::Binding {
                name,
                mutable,
                annotation,
                initializer,
            },
            pos,
        ))
    }

    fn function_declaration(&mut self, pos: SourcePos) -> Result<Stmt, KarmaError> {
        let name = self.consume_identifier("expected function name after 'fn'")?;
        self.consume_simple(&TokenKind::LeftParen, "expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check_simple(&TokenKind::RightParen) {
            loop {
                if params.len() >= 64 {
                    return Err(self.error_here("functions are limited to 64 parameters in v0.2"));
                }
                let param_token = self.consume_identifier_token("expected parameter name")?;
                let param_name = match &param_token.kind {
                    TokenKind::Identifier(name) => name.clone(),
                    _ => unreachable!(),
                };
                self.consume_simple(&TokenKind::Colon, "expected ':' after parameter name")?;
                let ty = self.consume_type("expected parameter type after ':'")?;
                params.push(Param {
                    name: param_name,
                    ty,
                    pos: param_token.pos(),
                });
                if !self.match_simple(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume_simple(&TokenKind::RightParen, "expected ')' after parameters")?;
        self.consume_simple(&TokenKind::Arrow, "expected '->' after function parameters")?;
        let return_type = self.consume_type("expected function return type after '->'")?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' before function body")?;
        let body = self.block_items()?;

        Ok(Stmt::new(
            StmtKind::Function {
                name,
                params,
                return_type,
                body,
            },
            pos,
        ))
    }

    fn statement(&mut self) -> Result<Stmt, KarmaError> {
        if self.match_simple(&TokenKind::If) {
            let pos = self.previous().pos();
            self.if_statement(pos)
        } else if self.match_simple(&TokenKind::While) {
            let pos = self.previous().pos();
            self.while_statement(pos)
        } else if self.match_simple(&TokenKind::Return) {
            let pos = self.previous().pos();
            self.return_statement(pos)
        } else if self.match_simple(&TokenKind::LeftBrace) {
            let pos = self.previous().pos();
            Ok(Stmt::new(StmtKind::Block(self.block_items()?), pos))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self, pos: SourcePos) -> Result<Stmt, KarmaError> {
        let condition = self.expression()?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' after if condition")?;
        let then_pos = self.previous().pos();
        let then_branch = Box::new(Stmt::new(StmtKind::Block(self.block_items()?), then_pos));

        let else_branch = if self.match_simple(&TokenKind::Else) {
            let else_pos = self.previous().pos();
            if self.match_simple(&TokenKind::If) {
                let nested_if_pos = self.previous().pos();
                Some(Box::new(self.if_statement(nested_if_pos)?))
            } else {
                self.consume_simple(&TokenKind::LeftBrace, "expected '{' after else")?;
                Some(Box::new(Stmt::new(
                    StmtKind::Block(self.block_items()?),
                    else_pos,
                )))
            }
        } else {
            None
        };

        Ok(Stmt::new(
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
            pos,
        ))
    }

    fn while_statement(&mut self, pos: SourcePos) -> Result<Stmt, KarmaError> {
        let condition = self.expression()?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' after while condition")?;
        let block_pos = self.previous().pos();
        let body = Box::new(Stmt::new(StmtKind::Block(self.block_items()?), block_pos));
        Ok(Stmt::new(StmtKind::While { condition, body }, pos))
    }

    fn return_statement(&mut self, pos: SourcePos) -> Result<Stmt, KarmaError> {
        let value = if self.check_simple(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume_simple(&TokenKind::Semicolon, "expected ';' after return value")?;
        Ok(Stmt::new(StmtKind::Return(value), pos))
    }

    fn block_items(&mut self) -> Result<Vec<Stmt>, KarmaError> {
        let mut statements = Vec::new();
        while !self.check_simple(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume_simple(&TokenKind::RightBrace, "expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, KarmaError> {
        let expr = self.expression()?;
        let pos = expr.pos;
        self.consume_simple(&TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::new(StmtKind::Expression(expr), pos))
    }

    fn expression(&mut self) -> Result<Expr, KarmaError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, KarmaError> {
        let expr = self.equality()?;
        if self.match_simple(&TokenKind::Equal) {
            let value = self.assignment()?;
            if let ExprKind::Variable(name) = &expr.kind {
                return Ok(Expr::new(
                    ExprKind::Assign {
                        name: name.clone(),
                        value: Box::new(value),
                    },
                    expr.pos,
                ));
            }
            return Err(self.error_previous("invalid assignment target"));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.comparison()?;
        while self.match_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let op_token = self.previous().clone();
            let op_pos = op_token.pos();
            let right = self.comparison()?;
            expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(expr),
                    op: op_token.kind,
                    right: Box::new(right),
                },
                op_pos,
            );
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.term()?;
        while self.match_any(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let op_token = self.previous().clone();
            let op_pos = op_token.pos();
            let right = self.term()?;
            expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(expr),
                    op: op_token.kind,
                    right: Box::new(right),
                },
                op_pos,
            );
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.factor()?;
        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.previous().clone();
            let op_pos = op_token.pos();
            let right = self.factor()?;
            expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(expr),
                    op: op_token.kind,
                    right: Box::new(right),
                },
                op_pos,
            );
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.unary()?;
        while self.match_any(&[TokenKind::Star, TokenKind::Slash]) {
            let op_token = self.previous().clone();
            let op_pos = op_token.pos();
            let right = self.unary()?;
            expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(expr),
                    op: op_token.kind,
                    right: Box::new(right),
                },
                op_pos,
            );
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, KarmaError> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let op_token = self.previous().clone();
            let op_pos = op_token.pos();
            let right = self.unary()?;
            return Ok(Expr::new(
                ExprKind::Unary {
                    op: op_token.kind,
                    right: Box::new(right),
                },
                op_pos,
            ));
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.primary()?;
        while self.match_simple(&TokenKind::LeftParen) {
            let call_pos = expr.pos;
            let callee = if let ExprKind::Variable(name) = &expr.kind {
                name.clone()
            } else {
                return Err(self.error_previous("only named functions are callable in v0.2"));
            };

            let mut arguments = Vec::new();
            if !self.check_simple(&TokenKind::RightParen) {
                loop {
                    if arguments.len() >= 64 {
                        return Err(self.error_here("calls are limited to 64 arguments in v0.2"));
                    }
                    arguments.push(self.expression()?);
                    if !self.match_simple(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume_simple(&TokenKind::RightParen, "expected ')' after arguments")?;
            expr = Expr::new(ExprKind::Call { callee, arguments }, call_pos);
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, KarmaError> {
        if self.match_simple(&TokenKind::False) {
            let token = self.previous().clone();
            return Ok(Expr::new(
                ExprKind::Literal(Literal::Bool(false)),
                token.pos(),
            ));
        }
        if self.match_simple(&TokenKind::True) {
            let token = self.previous().clone();
            return Ok(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                token.pos(),
            ));
        }

        let token = self.peek().clone();
        let pos = token.pos();
        match token.kind {
            TokenKind::Integer(value) => {
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Int(value)), pos))
            }
            TokenKind::String(value) => {
                self.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::String(value)), pos))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::new(ExprKind::Variable(name), pos))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume_simple(&TokenKind::RightParen, "expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(self.error_here("expected expression")),
        }
    }

    fn consume_type(&mut self, message: &str) -> Result<Type, KarmaError> {
        let token = self.peek().clone();
        let ty = match token.kind {
            TokenKind::TypeInt => Type::Int,
            TokenKind::TypeBool => Type::Bool,
            TokenKind::TypeString => Type::String,
            TokenKind::TypeUnit => Type::Unit,
            _ => return Err(KarmaError::new("parser", message, token.line, token.column)),
        };
        self.advance();
        Ok(ty)
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check_simple(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn match_simple(&mut self, kind: &TokenKind) -> bool {
        if self.check_simple(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_simple(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() && !matches!(kind, TokenKind::Eof) {
            return false;
        }
        same_variant(&self.peek().kind, kind)
    }

    fn consume_simple(&mut self, kind: &TokenKind, message: &str) -> Result<(), KarmaError> {
        if self.check_simple(kind) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_here(message))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, KarmaError> {
        let token = self.consume_identifier_token(message)?;
        if let TokenKind::Identifier(name) = token.kind {
            Ok(name)
        } else {
            unreachable!()
        }
    }

    fn consume_identifier_token(&mut self, message: &str) -> Result<Token, KarmaError> {
        match self.peek().kind.clone() {
            TokenKind::Identifier(_) => Ok(self.advance().clone()),
            _ => Err(self.error_here(message)),
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn error_here(&self, message: &str) -> KarmaError {
        let token = self.peek();
        KarmaError::new("parser", message, token.line, token.column)
    }

    fn error_previous(&self, message: &str) -> KarmaError {
        let token = self.previous();
        KarmaError::new("parser", message, token.line, token.column)
    }
}

fn same_variant(a: &TokenKind, b: &TokenKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Result<Vec<Stmt>, KarmaError> {
        Parser::new(Lexer::new(source).scan_tokens()?).parse()
    }

    #[test]
    fn parses_typed_and_inferred_bindings() {
        let program = parse("let age: Int = 38; let name = \"Karma\";").unwrap();
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn parses_mutable_binding() {
        let program = parse("mut count: Int = 0; count = count + 1;").unwrap();
        assert_eq!(program.len(), 2);
    }

    #[test]
    fn parses_typed_function() {
        let program = parse("fn add(a: Int, b: Int) -> Int { return a + b; }").unwrap();
        assert_eq!(program.len(), 1);
    }

    #[test]
    fn requires_function_parameter_types() {
        let error = parse("fn add(a, b: Int) -> Int { return b; }").unwrap_err();
        assert!(error.message.contains("expected ':'"));
    }
}
