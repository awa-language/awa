use ecow::EcoString;

use crate::lex::{error::LexicalError, location::Location, token::Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsingError {
    pub error: Type,
    pub location: Location,
}

impl ParsingError {
    pub fn get_description(&self) -> String {
        match &self.error {
            Type::LexicalError { error } => format!("lexical error: {}", error.get_description()),
            Type::UnexpectedToken { token, expected } => {
                format!("found: {token}, expected: {expected}")
            }
            Type::NoVarBinding { .. } => "missing var binding".to_owned(),
            Type::UnknownType { .. } => "unknown type".to_owned(),
            Type::MissingRightOperand => "operator is missing value on the right".to_owned(),
            Type::UnexpectedEof => "unexpected EOF".to_owned(),
            Type::InvalidName { .. } => "invalid name".to_owned(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    LexicalError { error: LexicalError },
    UnexpectedToken { token: Token, expected: EcoString },
    NoVarBinding { token: Token },
    UnknownType { token: Token },
    MissingRightOperand,
    UnexpectedEof,
    InvalidName { token: Token },
}
