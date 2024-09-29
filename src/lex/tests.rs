use itertools::Itertools;
use pretty_assertions::assert_eq;

use crate::lex::{
    lexer::{lex, TokenSpan},
    token::Token,
};

use super::{
    lexical_error::{LexicalError, Type},
    location::Location,
};

struct TestCase<'a> {
    input: &'a str,
    expected: Vec<Result<TokenSpan, LexicalError>>,
}

#[test]
fn test_int32_var() {
    let cases = vec![TestCase {
        input: "var name int32 =",
        expected: vec![
            Ok(TokenSpan {
                token: Token::Var,
                start: 0,
                end: 3,
            }),
            Ok(TokenSpan {
                token: Token::Name {
                    value: "name".into(),
                },
                start: 4,
                end: 8,
            }),
            Ok(TokenSpan {
                token: Token::Int32,
                start: 9,
                end: 14,
            }),
            Ok(TokenSpan {
                token: Token::Equal,
                start: 15,
                end: 15,
            }),
        ],
    }];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(lex, case.expected);
    }
}

#[test]
fn test_newlines() {
    let cases = vec![TestCase {
        input: "a\r\n\nb\r",
        expected: vec![
            Ok(TokenSpan {
                start: 0,
                end: 1,
                token: Token::Name { value: "a".into() },
            }),
            Ok(TokenSpan {
                start: 1,
                end: 3,
                token: Token::NewLine,
            }),
            Ok(TokenSpan {
                start: 3,
                end: 4,
                token: Token::NewLine,
            }),
            Ok(TokenSpan {
                start: 4,
                end: 5,
                token: Token::Name { value: "b".into() },
            }),
            Ok(TokenSpan {
                start: 5,
                end: 5,
                token: Token::NewLine,
            }),
        ],
    }];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(lex, case.expected);
    }
}

#[test]
fn test_int_literal_lexing() {
    let cases = vec![
        TestCase {
            input: "123",
            expected: vec![Ok(TokenSpan {
                token: Token::IntLiteral {
                    value: "123".into(),
                },
                start: 0,
                end: 2,
            })],
        },
        TestCase {
            input: "-123",
            expected: vec![Ok(TokenSpan {
                token: Token::IntLiteral {
                    value: "-123".into(),
                },
                start: 0,
                end: 3,
            })],
        },
    ];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(lex, case.expected);
    }
}

#[test]
fn test_float_literal_lexing() {
    let cases = vec![
        TestCase {
            input: "123.123",
            expected: vec![Ok(TokenSpan {
                token: Token::FloatLiteral {
                    value: "123.123".into(),
                },
                start: 0,
                end: 6,
            })],
        },
        TestCase {
            input: "-123.123",
            expected: vec![Ok(TokenSpan {
                token: Token::FloatLiteral {
                    value: "-123.123".into(),
                },
                start: 0,
                end: 7,
            })],
        },
    ];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(lex, case.expected);
    }
}

#[test]
fn test_int_literal_lexing_failed() {
    let cases = vec![TestCase {
        input: "123a456",
        expected: vec![
            Err(LexicalError {
                error: Type::UnexpectedNumberEnd,
                location: Location { start: 4, end: 4 },
            }),
            Ok(TokenSpan {
                token: Token::IntLiteral {
                    value: "456".into(),
                },
                start: 4,
                end: 6,
            }),
        ],
    }];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(lex, case.expected, "Test failed for input: {}", case.input);
    }
}

#[test]
fn test_float_literal_lexing_failed() {
    let cases = vec![
        TestCase {
            input: "123.",
            expected: vec![Err(LexicalError {
                error: Type::InvalidNumberFormat,
                location: Location { start: 3, end: 3 },
            })],
        },
        TestCase {
            input: "123..",
            expected: vec![Err(LexicalError {
                error: Type::InvalidNumberFormat,
                location: Location { start: 0, end: 4 },
            })],
        },
        TestCase {
            input: "123.123.123",
            expected: vec![
                Err(LexicalError {
                    error: Type::InvalidNumberFormat,
                    location: Location { start: 0, end: 8 },
                }),
                Ok(TokenSpan {
                    token: Token::IntLiteral {
                        value: "123".into(),
                    },
                    start: 8,
                    end: 10,
                }),
            ],
        },
    ];

    for case in cases {
        let lex: Vec<Result<TokenSpan, LexicalError>> = lex(case.input).collect();
        assert_eq!(lex, case.expected, "Test failed for input: {}", case.input);
    }
}
