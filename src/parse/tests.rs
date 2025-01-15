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
fn test_number_literals() {
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
