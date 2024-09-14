#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexicalError {
    pub error: Type,
    // TODO: add error location
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
