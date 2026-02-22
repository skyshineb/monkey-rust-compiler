use crate::ast::{BlockStatement, Expression, Identifier, Program, Statement};
use crate::lexer::Lexer;
use crate::parse_error::ParseError;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Or,
    And,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

fn token_precedence(kind: &TokenKind) -> Precedence {
    match kind {
        TokenKind::Or => Precedence::Or,
        TokenKind::And => Precedence::And,
        TokenKind::Eq | TokenKind::NotEq => Precedence::Equals,
        TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => Precedence::LessGreater,
        TokenKind::Plus | TokenKind::Minus => Precedence::Sum,
        TokenKind::Slash | TokenKind::Asterisk => Precedence::Product,
        TokenKind::LParen => Precedence::Call,
        TokenKind::LBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

/// Pratt parser for Monkey source.
#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            cur_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    pub fn parse_program(&mut self) -> Program {
        // TODO(step-6): evaluator/compiler will consume the parsed AST.
        let mut statements = Vec::new();

        while !self.cur_token_is(TokenKind::Eof) {
            if self.cur_token_is(TokenKind::Semicolon) {
                self.next_token();
                continue;
            }

            match self.parse_statement() {
                Some(stmt) => {
                    statements.push(stmt);
                    self.next_token();
                }
                None => {
                    if self.cur_token_is(TokenKind::RBrace) {
                        self.next_token();
                    } else {
                        self.synchronize_statement();
                    }
                }
            }
        }

        Program::new(statements)
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn cur_token_is(&self, kind: TokenKind) -> bool {
        self.cur_token.kind == kind
    }

    fn peek_token_is(&self, kind: TokenKind) -> bool {
        self.peek_token.kind == kind
    }

    fn cur_precedence(&self) -> Precedence {
        token_precedence(&self.cur_token.kind)
    }

    fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token.kind)
    }

    fn expect_peek(&mut self, expected: TokenKind) -> bool {
        if self.peek_token.kind == expected {
            self.next_token();
            true
        } else {
            self.peek_error(expected, self.peek_token.kind.clone(), self.peek_token.pos);
            false
        }
    }

    fn peek_error(
        &mut self,
        expected: TokenKind,
        actual: TokenKind,
        pos: crate::position::Position,
    ) {
        self.errors.push(ParseError::new(
            pos,
            format!("expected next token to be {expected}, got {actual}"),
        ));
    }

    fn no_prefix_parse_fn_error(&mut self, token_kind: TokenKind, pos: crate::position::Position) {
        self.errors.push(ParseError::new(
            pos,
            format!("no prefix parse function for {token_kind}"),
        ));
    }

    fn synchronize_statement(&mut self) {
        while !matches!(
            self.cur_token.kind,
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) {
            self.next_token();
        }

        if self.cur_token_is(TokenKind::Semicolon) {
            self.next_token();
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::Break => Some(self.parse_break_statement()),
            TokenKind::Continue => Some(self.parse_continue_statement()),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let pos = self.cur_token.pos;
        if !self.expect_peek(TokenKind::Ident) {
            return None;
        }
        let name = Identifier::new(self.cur_token.literal.clone(), self.cur_token.pos);

        if !self.expect_peek(TokenKind::Assign) {
            return None;
        }

        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenKind::Semicolon) {
            self.next_token();
        }

        Some(Statement::Let { name, value, pos })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        let pos = self.cur_token.pos;
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenKind::Semicolon) {
            self.next_token();
        }

        Some(Statement::Return { value, pos })
    }

    fn parse_while_statement(&mut self) -> Option<Statement> {
        let pos = self.cur_token.pos;
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let body = self.parse_block_statement(self.cur_token.pos);
        Some(Statement::While {
            condition,
            body,
            pos,
        })
    }

    fn parse_break_statement(&mut self) -> Statement {
        let pos = self.cur_token.pos;
        if self.peek_token_is(TokenKind::Semicolon) {
            self.next_token();
        }
        Statement::Break { pos }
    }

    fn parse_continue_statement(&mut self) -> Statement {
        let pos = self.cur_token.pos;
        if self.peek_token_is(TokenKind::Semicolon) {
            self.next_token();
        }
        Statement::Continue { pos }
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let pos = self.cur_token.pos;
        let expression = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(TokenKind::Semicolon) {
            self.next_token();
        }

        Some(Statement::Expression { expression, pos })
    }

    fn parse_block_statement(
        &mut self,
        open_brace_pos: crate::position::Position,
    ) -> BlockStatement {
        let mut statements = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenKind::RBrace) && !self.cur_token_is(TokenKind::Eof) {
            if self.cur_token_is(TokenKind::Semicolon) {
                self.next_token();
                continue;
            }

            match self.parse_statement() {
                Some(stmt) => {
                    statements.push(stmt);
                    self.next_token();
                }
                None => self.synchronize_statement(),
            }
        }

        BlockStatement::new(statements, open_brace_pos)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left = match self.cur_token.kind {
            TokenKind::Ident => Some(Expression::Identifier {
                value: self.cur_token.literal.clone(),
                pos: self.cur_token.pos,
            }),
            TokenKind::Int => self.parse_integer_literal(),
            TokenKind::True | TokenKind::False => Some(Expression::BooleanLiteral {
                value: self.cur_token_is(TokenKind::True),
                pos: self.cur_token.pos,
            }),
            TokenKind::String => Some(Expression::StringLiteral {
                value: self.cur_token.literal.clone(),
                pos: self.cur_token.pos,
            }),
            TokenKind::Bang | TokenKind::Minus => self.parse_prefix_expression(),
            TokenKind::LParen => self.parse_grouped_expression(),
            TokenKind::If => self.parse_if_expression(),
            TokenKind::Function => self.parse_function_literal(),
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::LBrace => self.parse_hash_literal(),
            _ => {
                self.no_prefix_parse_fn_error(self.cur_token.kind.clone(), self.cur_token.pos);
                None
            }
        }?;

        while !self.peek_token_is(TokenKind::Semicolon) && precedence < self.peek_precedence() {
            match self.peek_token.kind {
                TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Slash
                | TokenKind::Asterisk
                | TokenKind::Eq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge
                | TokenKind::And
                | TokenKind::Or => {
                    self.next_token();
                    left = self.parse_infix_expression(left)?;
                }
                TokenKind::LParen => {
                    self.next_token();
                    left = self.parse_call_expression(left)?;
                }
                TokenKind::LBracket => {
                    self.next_token();
                    left = self.parse_index_expression(left)?;
                }
                _ => return Some(left),
            }
        }

        Some(left)
    }

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        let raw = self.cur_token.literal.clone();
        match raw.parse::<i64>() {
            Ok(value) => Some(Expression::IntegerLiteral {
                value,
                raw,
                pos: self.cur_token.pos,
            }),
            Err(_) => {
                self.errors.push(ParseError::new(
                    self.cur_token.pos,
                    format!("invalid integer literal {raw}"),
                ));
                None
            }
        }
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let pos = self.cur_token.pos;
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix)?;
        Some(Expression::Prefix {
            operator,
            right: Box::new(right),
            pos,
        })
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(exp)
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let pos = self.cur_token.pos;
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement(self.cur_token.pos);
        let alternative = if self.peek_token_is(TokenKind::Else) {
            self.next_token(); // else
            if self.peek_token_is(TokenKind::LBrace) {
                self.next_token();
                Some(self.parse_block_statement(self.cur_token.pos))
            } else if self.peek_token_is(TokenKind::If) {
                self.next_token();
                let nested_if = self.parse_if_expression()?;
                let nested_pos = nested_if.pos();
                Some(BlockStatement::new(
                    vec![Statement::Expression {
                        expression: nested_if,
                        pos: nested_pos,
                    }],
                    nested_pos,
                ))
            } else {
                self.errors.push(ParseError::new(
                    self.peek_token.pos,
                    format!(
                        "expected next token to be {}, got {}",
                        TokenKind::LBrace,
                        self.peek_token.kind
                    ),
                ));
                return None;
            }
        } else {
            None
        };

        Some(Expression::If {
            condition: Box::new(condition),
            consequence,
            alternative,
            pos,
        })
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let pos = self.cur_token.pos;
        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }
        let parameters = self.parse_function_parameters()?;

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }
        let body = self.parse_block_statement(self.cur_token.pos);
        Some(Expression::FunctionLiteral {
            parameters,
            body,
            pos,
        })
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        let mut params = Vec::new();

        if self.peek_token_is(TokenKind::RParen) {
            self.next_token();
            return Some(params);
        }

        self.next_token();
        if !self.cur_token_is(TokenKind::Ident) {
            self.errors.push(ParseError::new(
                self.cur_token.pos,
                "expected identifier in parameter list",
            ));
            return None;
        }
        params.push(Identifier::new(
            self.cur_token.literal.clone(),
            self.cur_token.pos,
        ));

        while self.peek_token_is(TokenKind::Comma) {
            self.next_token();
            self.next_token();
            if !self.cur_token_is(TokenKind::Ident) {
                self.errors.push(ParseError::new(
                    self.cur_token.pos,
                    "expected identifier in parameter list",
                ));
                return None;
            }
            params.push(Identifier::new(
                self.cur_token.literal.clone(),
                self.cur_token.pos,
            ));
        }

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }
        Some(params)
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let pos = self.cur_token.pos;
        let elements = self.parse_expression_list(TokenKind::RBracket)?;
        Some(Expression::ArrayLiteral { elements, pos })
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let pos = self.cur_token.pos;
        let mut pairs = Vec::new();

        if self.peek_token_is(TokenKind::RBrace) {
            self.next_token();
            return Some(Expression::HashLiteral { pairs, pos });
        }

        loop {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if !self.expect_peek(TokenKind::Colon) {
                return None;
            }

            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            pairs.push((key, value));

            if !self.peek_token_is(TokenKind::Comma) {
                break;
            }
            self.next_token();
        }

        if !self.expect_peek(TokenKind::RBrace) {
            return None;
        }

        Some(Expression::HashLiteral { pairs, pos })
    }

    fn parse_expression_list(&mut self, end: TokenKind) -> Option<Vec<Expression>> {
        let mut list = Vec::new();

        if self.peek_token.kind == end {
            self.next_token();
            return Some(list);
        }

        self.next_token();
        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(TokenKind::Comma) {
            self.next_token();
            self.next_token();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if !self.expect_peek(end) {
            return None;
        }
        Some(list)
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let pos = self.cur_token.pos;
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Some(Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            pos,
        })
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let pos = self.cur_token.pos;
        let arguments = self.parse_expression_list(TokenKind::RParen)?;
        Some(Expression::Call {
            function: Box::new(function),
            arguments,
            pos,
        })
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        let pos = self.cur_token.pos;
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;
        if !self.expect_peek(TokenKind::RBracket) {
            return None;
        }
        Some(Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
            pos,
        })
    }
}
