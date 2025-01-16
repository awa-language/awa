use super::location::Location;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexicalError {
    pub error: Type,
    pub location: Location,
}

impl LexicalError {
    #[must_use] pub fn get_description(&self) -> &'static str {
        match &self.error {
            Type::UnrecognizedToken { .. } => "unrecognized token",
            Type::InvalidTripleEqual => "invalid `===`",
            Type::UnexpectedStringEnd => "unexpected string end",
            Type::BadEscapeCharacter => "bad escape character",
            Type::InvalidUnicodeEscape => "invalid unicode escape",
            Type::InvalidNumberFormat => "invalid number format",
            Type::UnexpectedNumberEnd => "unexpected number end",
            Type::UnexpectedCharEnd => "unexpected char end",
        }
    }
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
