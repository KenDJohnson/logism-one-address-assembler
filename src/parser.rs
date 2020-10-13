use logos::{Lexer, Logos, Span};

use super::{AddressedInstruction, Immediate, Instruction, Token};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidToken(String, String, Span),
    UnexpectedEof(String),
    DuplicateLabel(String, Span, Span),
    InstructionOverflow(String, Span),
    DataOverflow(String, Span),
    InvalidNumber(i16, Span),
    UnknownLabel(String),
}

#[derive(Debug, Clone)]
pub struct AddressedProgram {
    pub text: Vec<AddressedInstruction>,
    pub data: Vec<i16>,
}

impl AddressedProgram {
    pub fn assemble_text(&self) -> Vec<u8> {
        let mut assembled = Vec::with_capacity(self.text.len() * 2);
        for instr in &self.text {
            assembled.extend(&instr.bytes());
        }
        assembled
    }

    pub fn data_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.data.len() * 2);
        for data in &self.data {
            bytes.extend(&data.to_be_bytes());
        }

        bytes
    }
}

pub struct Parser<'a> {
    pub input: &'a str,
    pub lexer: Lexer<'a, Token<'a>>,

    pub text: Vec<Instruction<'a>>,
    pub data: Vec<i16>,

    pub text_labels: HashMap<&'a str, (u8, Span)>,
    pub data_labels: HashMap<&'a str, (u8, Span)>,

    pub peeked: Option<Token<'a>>,
}

impl fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Parser")
            .field("input", &self.input)
            .field("text", &self.text)
            .field("data", &self.data)
            .field("text_labels", &self.text_labels)
            .field("data_labels", &self.data_labels)
            .finish()
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            input,
            lexer: Token::lexer(input),
            text: vec![],
            data: vec![],
            text_labels: HashMap::new(),
            data_labels: HashMap::new(),
            peeked: None,
        }
    }

    pub fn parse(input: &'a str) -> Result<Self, ParseError> {
        let mut parser = Self::new(input);
        parser.parse_input()?;
        Ok(parser)
    }

    pub fn address_program(&mut self) -> Result<AddressedProgram, ParseError> {
        let mut text = Vec::with_capacity(self.text.len());
        let data = self.data.clone();

        for instr in self.text.iter() {
            let addressed = match instr {
                Instruction::Add(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Add(address)
                }
                Instruction::Subtract(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Subtract(address)
                }
                Instruction::Multiply(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Multiply(address)
                }
                Instruction::Divide(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Divide(address)
                }
                Instruction::Remainder(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Remainder(address)
                }
                Instruction::And(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::And(address)
                }
                Instruction::BranchZero(label) => {
                    let address = self
                        .text_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::BranchZero(address)
                }
                Instruction::Branch(label) => {
                    let address = self
                        .text_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Branch(address)
                }
                Instruction::Store(label) => {
                    let address = self
                        .data_label_address(label)
                        .ok_or_else(|| ParseError::UnknownLabel((*label).to_owned()))?;
                    AddressedInstruction::Store(address)
                }
                Instruction::AddImmediate(i) => AddressedInstruction::AddImmediate(*i),
                Instruction::SubtractImmediate(i) => AddressedInstruction::SubtractImmediate(*i),
                Instruction::MultiplyImmediate(i) => AddressedInstruction::MultiplyImmediate(*i),
                Instruction::DivideImmediate(i) => AddressedInstruction::DivideImmediate(*i),
                Instruction::RemainderImmediate(i) => AddressedInstruction::RemainderImmediate(*i),
                Instruction::Shift(i) => AddressedInstruction::Shift(*i),
                Instruction::AndImmediate(i) => AddressedInstruction::AndImmediate(*i),

                Instruction::ClearAc => AddressedInstruction::ClearAc,
                Instruction::NoOp => AddressedInstruction::NoOp,
            };
            text.push(addressed);
        }

        Ok(AddressedProgram { text, data })
    }

    fn next_token_opt(&mut self) -> Option<Token<'a>> {
        if self.peeked.is_some() {
            std::mem::take(&mut self.peeked)
        } else {
            self.lexer.next()
        }
    }

    fn next_token<S: ToString>(&mut self, expected: S) -> Result<Token<'a>, ParseError> {
        self.next_token_opt()
            .ok_or_else(|| ParseError::UnexpectedEof(expected.to_string()))
    }

    fn peek_token(&mut self) -> Option<Token<'a>> {
        if let t @ Some(_) = self.peeked.as_ref().cloned() {
            t
        } else {
            self.peeked = self.lexer.next();
            self.peeked.as_ref().cloned()
        }
    }

    fn parse_input(&mut self) -> Result<(), ParseError> {
        let token = self.next_token("expected `.text` or `.data`")?;

        match token {
            Token::Text => self.parse_text()?,
            Token::Data => self.parse_data()?,
            other => {
                return Err(ParseError::InvalidToken(
                    other.to_string(),
                    "expected `.text` or `.data`".to_owned(),
                    self.lexer.span(),
                ))
            }
        }

        Ok(())
    }

    fn text_label_address(&self, label: &str) -> Option<u8> {
        self.text_labels.get(label).map(|(loc, _)| *loc)
    }

    fn data_label_address(&self, label: &str) -> Option<u8> {
        self.data_labels.get(label).map(|(loc, _)| *loc)
    }

    fn add_text_label(&mut self) -> Result<(), ParseError> {
        let label = self.parse_label()?;
        if self.text_labels.contains_key(label) {
            let (_, span) = &self.text_labels[label];
            Err(ParseError::DuplicateLabel(
                label.to_owned(),
                span.clone(),
                self.lexer.span(),
            ))
        } else {
            let location = self.current_text();
            let span = self.lexer.span();

            self.text_labels.insert(label, (location, span));

            Ok(())
        }
    }

    fn add_data_label(&mut self) -> Result<(), ParseError> {
        let label = self.parse_label()?;
        if self.data_labels.contains_key(label) {
            let (_, span) = &self.data_labels[label];
            Err(ParseError::DuplicateLabel(
                label.to_owned(),
                span.clone(),
                self.lexer.span(),
            ))
        } else {
            let location = self.current_data();
            let span = self.lexer.span();

            self.data_labels.insert(label, (location, span));

            Ok(())
        }
    }

    fn parse_immediate(&mut self) -> Result<Immediate, ParseError> {
        match self.next_token("expected an integer")? {
            Token::NumLiteral(i) => match i8::try_from(i) {
                Ok(i) => Ok(i),
                Err(_) => Err(ParseError::InvalidNumber(i, self.lexer.span())),
            },
            other => Err(ParseError::InvalidToken(
                other.to_string(),
                "expected an integer".to_owned(),
                self.lexer.span(),
            )),
        }
    }

    fn parse_immediate_instr(&mut self, token: Token) -> Result<(), ParseError> {
        let ival = self.parse_immediate()?;
        let instr = match token {
            Token::AddImmediate => Instruction::AddImmediate(ival),
            Token::SubtractImmediate => Instruction::SubtractImmediate(ival),
            Token::MultiplyImmediate => Instruction::MultiplyImmediate(ival),
            Token::DivideImmediate => Instruction::DivideImmediate(ival),
            Token::RemainderImmediate => Instruction::RemainderImmediate(ival),
            Token::AndImmediate => Instruction::AndImmediate(ival),
            Token::Shift => Instruction::Shift(ival),
            _ => unreachable!(),
        };

        self.add_instr(instr)
    }

    fn parse_alu_instr(&mut self, token: Token) -> Result<(), ParseError> {
        let label = self.parse_label()?;
        let instr = match token {
            Token::Add => Instruction::Add(label),
            Token::Subtract => Instruction::Subtract(label),
            Token::Multiply => Instruction::Multiply(label),
            Token::Divide => Instruction::Divide(label),
            Token::Remainder => Instruction::Remainder(label),
            Token::And => Instruction::And(label),
            _ => unreachable!(),
        };

        self.add_instr(instr)
    }

    fn parse_label(&mut self) -> Result<&'a str, ParseError> {
        match self.next_token("expected a label")? {
            Token::LabelIdent(val) => Ok(val),
            other => Err(ParseError::InvalidToken(
                other.to_string(),
                "expected a label".to_owned(),
                self.lexer.span(),
            )),
        }
    }

    fn parse_text(&mut self) -> Result<(), ParseError> {
        loop {
            match self.next_token_opt() {
                Some(Token::Label) => self.add_text_label()?,
                Some(Token::Data) => return self.parse_data(),
                Some(t @ Token::Add)
                | Some(t @ Token::Subtract)
                | Some(t @ Token::Multiply)
                | Some(t @ Token::Divide)
                | Some(t @ Token::Remainder)
                | Some(t @ Token::And) => self.parse_alu_instr(t)?,

                Some(t @ Token::AddImmediate)
                | Some(t @ Token::SubtractImmediate)
                | Some(t @ Token::MultiplyImmediate)
                | Some(t @ Token::DivideImmediate)
                | Some(t @ Token::RemainderImmediate)
                | Some(t @ Token::AndImmediate)
                | Some(t @ Token::Shift) => self.parse_immediate_instr(t)?,

                Some(Token::BranchZero) => {
                    let label = self.parse_label()?;
                    self.add_instr(Instruction::BranchZero(label))?;
                }
                Some(Token::Branch) => {
                    let label = self.parse_label()?;
                    self.add_instr(Instruction::Branch(label))?;
                }
                Some(Token::ClearAc) => {
                    self.add_instr(Instruction::ClearAc)?;
                }
                Some(Token::Store) => {
                    let label = self.parse_label()?;
                    self.add_instr(Instruction::Store(label))?;
                }
                Some(Token::NoOp) => {
                    self.add_instr(Instruction::NoOp)?;
                }
                Some(other) => {
                    return Err(ParseError::InvalidToken(
                        other.to_string(),
                        "expected mnemonic, label, or `.data`".to_owned(),
                        self.lexer.span(),
                    ));
                }
                None => break,
            }
        }

        Ok(())
    }

    fn parse_number(&mut self) -> Result<i16, ParseError> {
        match self.next_token("expected `.number`")? {
            Token::Number => match self.next_token("expected an integer")? {
                Token::NumLiteral(val) => Ok(val),
                other => Err(ParseError::InvalidToken(
                    other.to_string(),
                    "expected an integer".to_owned(),
                    self.lexer.span(),
                )),
            },
            other => Err(ParseError::InvalidToken(
                other.to_string(),
                "expected `.number`".to_owned(),
                self.lexer.span(),
            )),
        }
    }

    fn parse_number_list(&mut self) -> Result<Vec<i16>, ParseError> {
        let mut numbers = Vec::new();

        while let Some(Token::Number) = self.peek_token() {
            numbers.push(self.parse_number()?);
        }

        Ok(numbers)
    }

    fn parse_data(&mut self) -> Result<(), ParseError> {
        loop {
            match self.next_token_opt() {
                Some(Token::Label) => {
                    self.add_data_label()?;
                    for number in self.parse_number_list()? {
                        self.add_data(number)?;
                    }
                }
                Some(Token::Text) => return self.parse_text(),
                Some(other) => {
                    return Err(ParseError::InvalidToken(
                        other.to_string(),
                        "expected `.label`".to_owned(),
                        self.lexer.span(),
                    ))
                }
                None => break,
            }
        }

        Ok(())
    }

    fn current_text(&self) -> u8 {
        self.text.len() as u8
    }

    fn current_data(&self) -> u8 {
        self.data.len() as u8
    }

    fn add_instr(&mut self, instr: Instruction<'a>) -> Result<(), ParseError> {
        if self.text.len() == 255 {
            Err(ParseError::InstructionOverflow(
                format!("{:?}", instr),
                self.lexer.span(),
            ))
        } else {
            self.text.push(instr);
            Ok(())
        }
    }

    fn add_data(&mut self, data: i16) -> Result<(), ParseError> {
        if self.data.len() == 255 {
            Err(ParseError::DataOverflow(
                format!("{}", data),
                self.lexer.span(),
            ))
        } else {
            self.data.push(data);
            Ok(())
        }
    }
}
