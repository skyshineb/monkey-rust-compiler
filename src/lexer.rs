use crate::position::Position;
use crate::token::{lookup_ident, Token, TokenKind};

/// Lexer for Monkey source input.
#[derive(Debug, Clone)]
pub struct Lexer {
    source: String,
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: impl Into<String>) -> Self {
        let source = input.into();
        let mut lexer = Self {
            input: source.chars().collect(),
            source,
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,
            col: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn input(&self) -> &str {
        &self.source
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let pos = self.current_pos();
        match self.ch {
            Some('=') if self.peek_char() == Some('=') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::Eq, "==", pos)
            }
            Some('!') if self.peek_char() == Some('=') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::NotEq, "!=", pos)
            }
            Some('<') if self.peek_char() == Some('=') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::Le, "<=", pos)
            }
            Some('>') if self.peek_char() == Some('=') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::Ge, ">=", pos)
            }
            Some('&') if self.peek_char() == Some('&') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::And, "&&", pos)
            }
            Some('|') if self.peek_char() == Some('|') => {
                self.read_char();
                self.read_char();
                Token::new(TokenKind::Or, "||", pos)
            }
            Some('=') => self.single_char_token(TokenKind::Assign, '=', pos),
            Some('+') => self.single_char_token(TokenKind::Plus, '+', pos),
            Some('-') => self.single_char_token(TokenKind::Minus, '-', pos),
            Some('!') => self.single_char_token(TokenKind::Bang, '!', pos),
            Some('*') => self.single_char_token(TokenKind::Asterisk, '*', pos),
            Some('/') => self.single_char_token(TokenKind::Slash, '/', pos),
            Some('<') => self.single_char_token(TokenKind::Lt, '<', pos),
            Some('>') => self.single_char_token(TokenKind::Gt, '>', pos),
            Some(',') => self.single_char_token(TokenKind::Comma, ',', pos),
            Some(';') => self.single_char_token(TokenKind::Semicolon, ';', pos),
            Some(':') => self.single_char_token(TokenKind::Colon, ':', pos),
            Some('(') => self.single_char_token(TokenKind::LParen, '(', pos),
            Some(')') => self.single_char_token(TokenKind::RParen, ')', pos),
            Some('{') => self.single_char_token(TokenKind::LBrace, '{', pos),
            Some('}') => self.single_char_token(TokenKind::RBrace, '}', pos),
            Some('[') => self.single_char_token(TokenKind::LBracket, '[', pos),
            Some(']') => self.single_char_token(TokenKind::RBracket, ']', pos),
            Some('"') => {
                let (literal, terminated) = self.read_string();
                let kind = if terminated {
                    TokenKind::String
                } else {
                    TokenKind::Illegal
                };
                Token::new(kind, literal, pos)
            }
            Some(ch) if is_ident_start(ch) => {
                let literal = self.read_identifier();
                let kind = lookup_ident(&literal);
                Token::new(kind, literal, pos)
            }
            Some(ch) if ch.is_ascii_digit() => {
                let literal = self.read_number();
                Token::new(TokenKind::Int, literal, pos)
            }
            Some(ch) => {
                self.read_char();
                Token::new(TokenKind::Illegal, ch.to_string(), pos)
            }
            None => Token::new(TokenKind::Eof, "", pos),
        }
    }

    pub fn tokenize_all(mut self) -> Vec<Token> {
        // TODO(step-4): parser should consume tokens incrementally from `next_token`.
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn read_char(&mut self) {
        let prev = self.ch;
        if let Some(next) = self.input.get(self.read_position).copied() {
            self.position = self.read_position;
            self.read_position += 1;
            self.ch = Some(next);

            match prev {
                Some('\n') => {
                    self.line += 1;
                    self.col = 1;
                }
                Some(_) => {
                    self.col += 1;
                }
                None if self.col == 0 => {
                    self.line = 1;
                    self.col = 1;
                }
                None => {}
            }
        } else {
            self.position = self.read_position;
            self.ch = None;

            match prev {
                Some('\n') => {
                    self.line += 1;
                    self.col = 1;
                }
                Some(_) => {
                    self.col += 1;
                }
                None if self.col == 0 => {
                    self.line = 1;
                    self.col = 1;
                }
                None => {}
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.read_position).copied()
    }

    fn current_pos(&self) -> Position {
        Position::new(self.line, self.col)
    }

    fn single_char_token(&mut self, kind: TokenKind, ch: char, pos: Position) -> Token {
        self.read_char();
        Token::new(kind, ch.to_string(), pos)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while let Some(ch) = self.ch {
                if ch.is_whitespace() {
                    self.read_char();
                } else {
                    break;
                }
            }

            if self.ch == Some('#') {
                self.skip_line_comment();
                continue;
            }

            break;
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.ch {
            if ch == '\n' {
                break;
            }
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while let Some(ch) = self.ch {
            if is_ident_continue(ch) {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start..self.position].iter().collect()
    }

    fn read_number(&mut self) -> String {
        let start = self.position;
        while let Some(ch) = self.ch {
            if ch.is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start..self.position].iter().collect()
    }

    fn read_string(&mut self) -> (String, bool) {
        let start = self.position + 1;
        self.read_char();

        while let Some(ch) = self.ch {
            if ch == '"' {
                let content: String = self.input[start..self.position].iter().collect();
                self.read_char();
                return (content, true);
            }
            self.read_char();
        }

        let content: String = self.input[start..self.position].iter().collect();
        (content, false)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}
