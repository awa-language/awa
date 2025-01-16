use crate::{
    error::Error,
    lex::{error::LexicalError, location::Location},
    parse::error::ParsingError,
};

use super::{parse_module, parse_statement_sequence};

macro_rules! assert_error {
    ($src:expr, $error:expr $(,)?) => {
        let result = crate::parse::parse_statement_sequence($src).expect_err("should not parse");
        assert_eq!(($src, $error), ($src, result),);
    };
}

macro_rules! assert_parse {
    ($src:expr $(,)?) => {
        let _result = crate::parse::parse_statement_sequence($src).expect("should parse");
    };
}

macro_rules! assert_parse_module {
    ($src:expr $(,)?) => {
        let _result = crate::parse::parse_module($src).expect("should parse");
    };
}

pub fn expect_module_error(src: &str) -> String {
    let result = parse_module(src).expect_err("should not parse");
    let error = Error::Parsing {
        path: "test/path.awa".into(),
        src: src.into(),
        error: result,
    };

    error.to_pretty_string()
}

pub fn expect_error(src: &str) -> String {
    let result = parse_statement_sequence(src).expect_err("should not parse");
    let error = crate::error::Error::Parsing {
        src: src.into(),
        path: "test/path.awa".into(),
        error: result,
    };

    error.to_pretty_string()
}

#[test]
fn test_string_literals() {
    assert_error!(
        "\"abc",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnexpectedStringEnd,
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
    assert_error!(
        "\"\\a\"",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::BadEscapeCharacter,
                    location: Location { start: 2, end: 2 },
                }
            },
            location: Location { start: 2, end: 2 },
        }
    );
    assert_error!(
        "\"\\a",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::BadEscapeCharacter,
                    location: Location { start: 2, end: 2 },
                }
            },
            location: Location { start: 2, end: 2 },
        }
    );
    assert_error!(
        "\"\\u123\"",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidUnicodeEscape,
                    location: Location { start: 3, end: 3 },
                }
            },
            location: Location { start: 3, end: 3 },
        }
    );
    assert_error!(
        "\"\\u{123\"",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidUnicodeEscape,
                    location: Location { start: 3, end: 7 },
                }
            },
            location: Location { start: 3, end: 7 },
        }
    );
}

#[test]
fn test_invalid_characters() {
    assert_error!(
        "🫧",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnrecognizedToken { token: '🫧' },
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
    assert_error!(
        "ава",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnrecognizedToken { token: 'а' },
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
    assert_error!(
        "洗脳塾",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnrecognizedToken { token: '洗' },
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
    assert_error!(
        "ඞ",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnrecognizedToken { token: 'ඞ' },
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
}

#[test]
fn test_char_literals() {
    assert_error!(
        "'abc",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnexpectedCharEnd,
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
    assert_error!(
        "'a",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnexpectedCharEnd,
                    location: Location { start: 0, end: 0 },
                }
            },
            location: Location { start: 0, end: 0 },
        }
    );
}

#[test]
fn test_int_literals() {
    assert_error!(
        "123a456",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::UnexpectedNumberEnd,
                    location: Location { start: 4, end: 4 },
                }
            },
            location: Location { start: 4, end: 4 },
        }
    );
}

#[test]
fn test_float_literals() {
    assert_error!(
        "123.",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidNumberFormat,
                    location: Location { start: 3, end: 3 },
                }
            },
            location: Location { start: 3, end: 3 },
        }
    );
    assert_error!(
        "123..",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidNumberFormat,
                    location: Location { start: 0, end: 4 },
                }
            },
            location: Location { start: 0, end: 4 },
        }
    );
    assert_error!(
        "123.123.123",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidNumberFormat,
                    location: Location { start: 0, end: 8 },
                }
            },
            location: Location { start: 0, end: 8 },
        }
    );
}

#[test]
fn test_triple_equal() {
    assert_error!(
        "===",
        ParsingError {
            error: crate::parse::error::Type::LexicalError {
                error: LexicalError {
                    error: crate::lex::error::Type::InvalidTripleEqual,
                    location: Location { start: 0, end: 2 },
                }
            },
            location: Location { start: 0, end: 2 },
        }
    );
}

#[test]
fn test_uno() {
    assert_parse_module!("");
}

