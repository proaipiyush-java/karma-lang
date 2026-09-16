use crate::ast::{Expr, Literal, Stmt};
use crate::error::KarmaError;
use crate::token::{Token, TokenKind};

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
            self.let_declaration()
        } else if self.match_simple(&TokenKind::Fn) {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Stmt, KarmaError> {
        let name = self.consume_identifier("expected variable name after 'let'")?;
        self.consume_simple(&TokenKind::Equal, "expected '=' after variable name")?;
        let initializer = self.expression()?;
        self.consume_simple(&TokenKind::Semicolon, "expected ';' after variable declaration")?;
        Ok(Stmt::Let { name, initializer })
    }

    fn function_declaration(&mut self) -> Result<Stmt, KarmaError> {
        let name = self.consume_identifier("expected function name after 'fn'")?;
        self.consume_simple(&TokenKind::LeftParen, "expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check_simple(&TokenKind::RightParen) {
            loop {
                if params.len() >= 64 {
                    return Err(self.error_here("functions are limited to 64 parameters in v0.1"));
                }
                params.push(self.consume_identifier("expected parameter name")?);
                if !self.match_simple(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume_simple(&TokenKind::RightParen, "expected ')' after parameters")?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' before function body")?;
        let body = self.block_items()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn statement(&mut self) -> Result<Stmt, KarmaError> {
        if self.match_simple(&TokenKind::If) {
            self.if_statement()
        } else if self.match_simple(&TokenKind::While) {
            self.while_statement()
        } else if self.match_simple(&TokenKind::Return) {
            self.return_statement()
        } else if self.match_simple(&TokenKind::LeftBrace) {
            Ok(Stmt::Block(self.block_items()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, KarmaError> {
        let condition = self.expression()?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' after if condition")?;
        let then_branch = Box::new(Stmt::Block(self.block_items()?));
        let else_branch = if self.match_simple(&TokenKind::Else) {
            if self.match_simple(&TokenKind::If) {
                Some(Box::new(self.if_statement()?))
            } else {
                self.consume_simple(&TokenKind::LeftBrace, "expected '{' after else")?;
                Some(Box::new(Stmt::Block(self.block_items()?)))
            }
        } else {
            None
        };
        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn while_statement(&mut self) -> Result<Stmt, KarmaError> {
        let condition = self.expression()?;
        self.consume_simple(&TokenKind::LeftBrace, "expected '{' after while condition")?;
        let body = Box::new(Stmt::Block(self.block_items()?));
        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, KarmaError> {
        let value = if self.check_simple(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume_simple(&TokenKind::Semicolon, "expected ';' after return value")?;
        Ok(Stmt::Return(value))
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
        self.consume_simple(&TokenKind::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, KarmaError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, KarmaError> {
        let expr = self.equality()?;
        if self.match_simple(&TokenKind::Equal) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign { name, value: Box::new(value) });
            }
            return Err(self.error_previous("invalid assignment target"));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.comparison()?;
        while self.match_any(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let op = self.previous().kind.clone();
            let right = self.comparison()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
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
            let op = self.previous().kind.clone();
            let right = self.term()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.factor()?;
        while self.match_any(&[TokenKind::Plus, TokenKind::Minus]) {
            let op = self.previous().kind.clone();
            let right = self.factor()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.unary()?;
        while self.match_any(&[TokenKind::Star, TokenKind::Slash]) {
            let op = self.previous().kind.clone();
            let right = self.unary()?;
            expr = Expr::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, KarmaError> {
        if self.match_any(&[TokenKind::Bang, TokenKind::Minus]) {
            let op = self.previous().kind.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary { op, right: Box::new(right) });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, KarmaError> {
        let mut expr = self.primary()?;
        while self.match_simple(&TokenKind::LeftParen) {
            let callee = if let Expr::Variable(name) = expr {
                name
            } else {
                return Err(self.error_previous("only named functions are callable in v0.1"));
            };
            let mut arguments = Vec::new();
            if !self.check_simple(&TokenKind::RightParen) {
                loop {
                    if arguments.len() >= 64 {
                        return Err(self.error_here("calls are limited to 64 arguments in v0.1"));
                    }
                    arguments.push(self.expression()?);
                    if !self.match_simple(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume_simple(&TokenKind::RightParen, "expected ')' after arguments")?;
            expr = Expr::Call { callee, arguments };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, KarmaError> {
        if self.match_simple(&TokenKind::False) {
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.match_simple(&TokenKind::True) {
            return Ok(Expr::Literal(Literal::Bool(true)));
        }

        match self.peek().kind.clone() {
            TokenKind::Integer(value) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(value)))
            }
            TokenKind::String(value) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(value)))
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable(name))
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
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
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
        match self.peek().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_here(message)),
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn parses_function() {
        let tokens = Lexer::new("fn add(a, b) { return a + b; }")
            .scan_tokens()
            .unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        assert!(matches!(&program[0], Stmt::Function { name, params, .. } if name == "add" && params.len() == 2));
    }
}
