use std::fmt::Write;

use crate::tokenizer::{Tokenizer, TokenType, Token};
use crate::chunk::{Chunk, Byte, OpCode};
use crate::error::{Error, ErrorResult};
use crate::value::Value;

pub struct Parser {
    tokenizer: Tokenizer,
    chunk: Chunk,
    current: Token,
    previous: Token
}

#[repr(u8)]
#[allow(dead_code)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}

type ParseFn = fn(&mut Parser, bool) -> ErrorResult<()>;

struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence,
}

impl ParseRule {
    pub fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        ParseRule {
            prefix,
            infix,
            precedence,
        }
    }
}

impl Parser {
    pub fn new(code: &str) -> Self {
        Self {
            tokenizer: Tokenizer::new(code),
            chunk: Chunk::new(),
            previous: Token::new_no_text(TokenType::EndOfFile, 0),
            current: Token::new_no_text(TokenType::EndOfFile, 0)
        }
    }

    pub fn parse(&mut self) -> ErrorResult<Chunk> {
        self.chunk = Chunk::new();

        self.advance()?;
        
        while !self.match_tok(TokenType::EndOfFile)? {
            self.declaration()?;
        }

        self.end_compilation()?;

        Ok(self.chunk.clone())
    }

    fn declaration(&mut self) -> ErrorResult<()> {
        if self.match_tok(TokenType::Var)? {
            self.var_declaration()?;
        } else {
            self.statement()?;
        }
        Ok(())
    }

    fn var_declaration(&mut self) -> ErrorResult<()> {
        let global = self.parse_variable(Error::from("Expected variable name."))?;

        if self.match_tok(TokenType::Eq)? {
            self.expression()?;
        } else {
            self.write_byte(OpCode::Nil as u8);
        }

        self.consume(TokenType::Semicolon, Error::from("Expected ';' after variable declaration."))?;

        self.define_variable(global);
        Ok(())
    }

    fn parse_variable(&mut self, error_msg: Error) -> ErrorResult<u8> {
        self.consume(TokenType::Ident, error_msg)?;

        Ok(self.identifier_constant(self.previous.clone())?)
    }

    fn identifier_constant(&mut self, identifier_token: Token) -> ErrorResult<u8> {
        self.make_constant(Value::String(identifier_token.text))
    }

    fn define_variable(&mut self, global: u8) {
        self.write_bytes(OpCode::DefineGlobal as u8, global);
    }

    fn variable(&mut self, can_assign: bool) -> ErrorResult<()> {
        self.named_variable(self.previous.clone(), can_assign)?;
        Ok(())
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) -> ErrorResult<()>  {
        let arg = self.identifier_constant(name)?;
        if can_assign && self.match_tok(TokenType::Eq)? {
            self.expression()?;
            self.write_bytes(OpCode::SetGlobal as u8, arg);
        } else {
            self.write_bytes(OpCode::GetGlobal as u8, arg);
        }
        Ok(())
    }
    
    fn statement(&mut self) -> ErrorResult<()> {
        if self.match_tok(TokenType::Print)? {
            self.print_statement()?;
        } else if self.match_tok(TokenType::LBrace)? {
            self.block()?;
        } else {
            self.expression_statement()?;
        }

        Ok(())
    }

    fn block(&mut self) -> ErrorResult<()> {
        while !self.check(TokenType::RBrace) && !self.check(TokenType::EndOfFile) {
            self.declaration()?;
        }

        self.consume(TokenType::RBrace, Error::from("Expect '(' after block."))?;

        Ok(())
    }

    fn expression_statement(&mut self) -> ErrorResult<()> {
        self.expression()?;
        self.consume(TokenType::Semicolon, Error::from("Expect ';' after expression."))?;
        self.write_byte(OpCode::Pop as u8);
        Ok(())
    }

    pub fn print_statement(&mut self) -> ErrorResult<()> {
        self.expression()?;
        self.consume(TokenType::Semicolon, Error::from("Expected ';' after value."))?;
        self.write_byte(OpCode::Print as u8);
        Ok(())
    }

    fn advance(&mut self) -> ErrorResult<()> {
        self.previous = self.current.clone();

        loop {
            self.current = self.tokenizer.scan_token();

            if &self.current.ty == &TokenType::Error {
                let txt = self.current.text.clone();
                return self.error_at_current(txt);
            } else {
                return Ok(())
            }
        }
    }

    fn match_tok(&mut self, ty: TokenType) -> ErrorResult<bool> {
        if !self.check(ty) {
            return Ok(false);
        }

        self.advance()?;
        Ok(true)
    }

    fn check(&self, ty: TokenType) -> bool {
        self.current.ty == ty
    }

    fn error_at_current(&mut self, message: String) -> ErrorResult<()> {
        self.error_at(self.current.clone(), message)
    }

    fn error(&mut self, message: String) -> ErrorResult<()> {
        self.error_at(self.previous.clone(), message)
    }

    #[allow(unused_must_use)]
    fn error_at(&mut self, token: Token, message: String) -> ErrorResult<()> {
        let mut error_string = Error::new();
        error_string.write_str(&format!("[line {}] Error: ", token.line));
        error_string.write_str(&message);

        Err(error_string)
    }

    fn expression(&mut self) -> ErrorResult<()> {
        self.parse_precedence(Precedence::Assignment as u8)?;
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) {
        self.chunk.write_byte(Byte::new(byte, self.previous.line));
    }

    fn write_bytes(&mut self, byte1: u8, byte2: u8) {
        self.write_byte(byte1);
        self.write_byte(byte2);
    }

    fn consume(&mut self, ty: TokenType, message: Error) -> ErrorResult<()> {
        if self.current.ty == ty {
            self.advance()?;
            return Ok(())
        }

        self.error_at_current(message)
    }


    fn number(&mut self, _: bool) -> ErrorResult<()> {
        if let TokenType::Number = self.previous.ty {
            let v = self.previous.text.parse().unwrap();
            self.write_constant(Value::Number(v))?;
            return Ok(())
        }

        unreachable!()
    }

    fn write_constant(&mut self, value: Value) -> ErrorResult<()> {
        let value_byte = self.make_constant(value)?;
        self.write_bytes(OpCode::Constant as u8, value_byte);
        Ok(())
    }

    fn make_constant(&mut self, value: Value) -> ErrorResult<u8> {
        let constant = self.chunk.write_value(value);

        if constant > u8::MAX {
            self.error("Too many constants in one chunk.".to_string())?;
        }

        Ok(constant)
    }

    fn end_compilation(&mut self) -> ErrorResult<()> {
        self.write_byte(OpCode::Return as u8);
        Ok(())
    }

    fn unary(&mut self, _: bool) -> ErrorResult<()> {
        let op_type = self.previous.ty.clone();

        self.parse_precedence(Precedence::Unary as u8)?;

        match op_type {
            TokenType::Minus => self.write_byte(OpCode::Negate as u8),
            TokenType::Bang => self.write_byte(OpCode::Not as u8),
            _ => unreachable!()
        }

        Ok(())
    }

    fn binary(&mut self, _: bool) -> ErrorResult<()> {
        let op_type = self.previous.ty.clone();
        let parse_rule = Self::get_parse_rule(op_type.clone());
        self.parse_precedence(parse_rule.precedence as u8 + 1)?;

        match op_type {
            TokenType::Plus => self.write_byte(OpCode::Add as u8),
            TokenType::Minus => self.write_byte(OpCode::Subtract as u8),
            TokenType::Asterisk => self.write_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.write_byte(OpCode::Divide as u8),
            TokenType::BangEq => self.write_bytes(OpCode::Equal as u8, OpCode::Not as u8),
            TokenType::EqEq => self.write_byte(OpCode::Equal as u8),
            TokenType::Greater => self.write_byte(OpCode::Greater as u8),
            TokenType::GreaterEq => self.write_bytes(OpCode::Less as u8, OpCode::Not as u8),
            TokenType::Less => self.write_byte(OpCode::Less as u8),
            TokenType::LessEq => self.write_bytes(OpCode::Greater as u8, OpCode::Not as u8),
            _ => unreachable!()   
        }

        Ok(())
    }

    fn grouping(&mut self, _: bool) -> ErrorResult<()> {
        self.expression()?;
        self.consume(TokenType::RParen, Error::from("Expected ')' after expression."))?;
        Ok(())
    }

    fn literal(&mut self, _: bool) -> ErrorResult<()> {
        match self.previous.ty {
            TokenType::False => self.write_byte(OpCode::False as u8),
            TokenType::True => self.write_byte(OpCode::True as u8),
            TokenType::Nil => self.write_byte(OpCode::Nil as u8),
            _ => unreachable!()
        }

        Ok(())
    }

    fn string(&mut self, _: bool) -> ErrorResult<()> {
        let string: String;

        if let TokenType::String = self.previous.ty.clone() {
            string = self.previous.text.clone();
        } else {
            unreachable!()
        }

        self.write_constant(Value::String(string))?;
        Ok(())
    }

    fn parse_precedence(&mut self, precedence: u8) -> ErrorResult<()> {
        self.advance()?;

        let prefix_rule = Self::get_parse_rule(self.previous.ty.clone()).prefix;

        if prefix_rule.is_none() {
            return self.error("Expected expression.".to_string())
        }

        let can_assign = precedence <= Precedence::Assignment as u8;
        prefix_rule.unwrap()(self, can_assign)?;

        while precedence
            <= Self::get_parse_rule(self.current.clone().ty).precedence as u8
        {
            self.advance()?;
            let infix_rule = Self::get_parse_rule(self.previous.ty.clone()).infix;
            infix_rule.unwrap()(self, can_assign)?;
        }

        Ok(())
    }

    fn get_parse_rule(t: TokenType) -> ParseRule {
        match t {
            TokenType::LParen => ParseRule::new(Some(Self::grouping), None, Precedence::None),
            TokenType::Minus => ParseRule::new(Some(Self::unary), Some(Self::binary), Precedence::Term),
            TokenType::Plus => ParseRule::new(None, Some(Self::binary), Precedence::Term),
            TokenType::Semicolon => ParseRule::new(None, None, Precedence::None),
            TokenType::Slash => ParseRule::new(None, Some(Self::binary), Precedence::Factor),
            TokenType::Asterisk => ParseRule::new(None, Some(Self::binary), Precedence::Factor),
            TokenType::Number => ParseRule::new(Some(Self::number), None, Precedence::None),
            TokenType::Bang => ParseRule::new(Some(Self::unary), None, Precedence::None),
            TokenType::BangEq => ParseRule::new(None, Some(Self::binary), Precedence::Equality),
            TokenType::EqEq => ParseRule::new(None, Some(Self::binary), Precedence::Equality),
            TokenType::Greater => ParseRule::new(None, Some(Self::binary), Precedence::Comparison),
            TokenType::GreaterEq => ParseRule::new(None, Some(Self::binary), Precedence::Comparison),
            TokenType::Less => ParseRule::new(None, Some(Self::binary), Precedence::Comparison),
            TokenType::LessEq => ParseRule::new(None, Some(Self::binary), Precedence::Comparison),
            TokenType::False => ParseRule::new(Some(Self::literal), None, Precedence::None),
            TokenType::True => ParseRule::new(Some(Self::literal), None, Precedence::None),
            TokenType::Nil => ParseRule::new(Some(Self::literal), None, Precedence::None),
            TokenType::String => ParseRule::new(Some(Self::string), None, Precedence::None),
            TokenType::Ident => ParseRule::new(Some(Self::variable), None, Precedence::None),
            _ => ParseRule::new(None, None, Precedence::None),
        }
    }
}