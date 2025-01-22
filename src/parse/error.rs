use ecow::EcoString;

use crate::lex::{error::LexicalError, location::Location, token::Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsingError {
    pub error: Type,
    pub location: Location,
}

impl ParsingError {
    #[must_use] pub fn get_description(&self) -> String {
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
            Type::ExpectedStatementSequence => "expected statement sequence".to_owned(),
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
    ExpectedStatementSequence,
}



#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConvertingError {
    pub error: ConvertingErrorType,
    pub location: Location,
}

impl ConvertingError {
    #[must_use] pub fn get_description(&self) -> String {
        match &self.error {
            ConvertingErrorType::IntOperationInvalidType => "integer operations require integer expressions in both sides".to_owned(),
            ConvertingErrorType::InvalidIntLiteral => "invalid integer literal".to_owned(),
            ConvertingErrorType::FloatOperationInvalidType => "float operations require float expressions in both sides".to_owned(),
            ConvertingErrorType::InvalidFloatLiteral => "invalid float literal".to_owned(),
            ConvertingErrorType::StringOperationInvalidType => "string operations requires string expressions in both sides".to_owned(),
            ConvertingErrorType::InvalidCharLiteral => "invalid char literal".to_owned(),
            ConvertingErrorType::UnsupportedBinaryOperation => "unsupported binary operation".to_owned(),
            ConvertingErrorType::UnsupportedType => "unsupported type".to_owned(),

        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConvertingErrorType {
    IntOperationInvalidType,
    InvalidIntLiteral,
    FloatOperationInvalidType,
    InvalidFloatLiteral,
    StringOperationInvalidType,
    InvalidCharLiteral,
    UnsupportedBinaryOperation,
    UnsupportedType,
}