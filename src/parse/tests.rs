use crate::error::Error;

use super::{parse_module, parse_statement_sequence};

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
