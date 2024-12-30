use ecow::EcoString;

use crate::lex::{error::LexicalError, location::Location, token::Token};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsingError {
    pub error: Type,
    pub location: Location,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    LexicalError {
        error: LexicalError,
    },
    UnexpectedToken {
        token: Token,
        expected: Vec<EcoString>,
    },
    NoVarBinding {
        token: Token,
    },
    // remove?
    UnrecognizedToken {
        token: char,
    },
    // TODO: rename
    OperatorNakedRight,
    UnexpectedEof,
    InvalidName {
        token: Token,
    },
}
