#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexicalError {
    pub error: LexicalErrorType,
    // TODO: add error location
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexicalErrorType {
    UnrecognizedToken { token: char },
    InvalidTripleEqual,
    UnexpectedStringEnd,
    BadEscapeCharacter,
    InvalidUnicodeEscape,
    InvalidNumberFormat,
    UnexpectedNumberEnd,
    UnexpectedCharEnd,
}
