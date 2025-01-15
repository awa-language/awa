use crate::error::Error;

use super::parse_module;

pub fn expect_module_error(src: &str) -> String {
    let result = parse_module(src).expect_err("should not parse");
    let error = Error::Parsing {
        path: "test/path".into(),
        src: src.into(),
        error: result,
    };

    error.pretty_string()
}
