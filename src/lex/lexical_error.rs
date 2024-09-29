use super::location::Location;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexicalError {
    pub error: Type,
    pub location: Location,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    UnrecognizedToken { token: char },
    InvalidTripleEqual,
    UnexpectedStringEnd,
    BadEscapeCharacter,
    InvalidUnicodeEscape,
    InvalidNumberFormat,
    UnexpectedNumberEnd,
    UnexpectedCharEnd,
}
