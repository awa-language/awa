use itertools::Itertools;
use pretty_assertions::assert_eq;

use crate::lex::{
    lexer::{lex, TokenSpan},
    token::Token,
};

use super::{
    error::{LexicalError, Type},
    location::Location,
};

struct TestCase<'a> {
    input: &'a str,
    expected: Vec<Result<TokenSpan, LexicalError>>,
}

#[test]
fn test_var_assignment() {
    let cases = vec![TestCase {
        input: "var name int =",
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
                token: Token::Int,
                start: 9,
                end: 12,
            }),
            Ok(TokenSpan {
                token: Token::Equal,
                start: 13,
                end: 13,
            }),
        ],
    }];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(case.expected, lex);
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
        assert_eq!(case.expected, lex);
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
        assert_eq!(case.expected, lex);
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
        assert_eq!(case.expected, lex);
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
        assert_eq!(case.expected, lex, "Test failed for input: {}", case.input);
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
        assert_eq!(case.expected, lex, "Test failed for input: {}", case.input);
    }
}

#[test]
fn test_comment() {
    let cases = vec![TestCase {
        input: "// comment \n// comment\n",
        expected: vec![
            Ok(TokenSpan {
                token: Token::Comment,
                start: 2,
                end: 11,
            }),
            Ok(TokenSpan {
                token: Token::NewLine,
                start: 11,
                end: 12,
            }),
            Ok(TokenSpan {
                token: Token::Comment,
                start: 14,
                end: 22,
            }),
            Ok(TokenSpan {
                token: Token::NewLine,
                start: 22,
                end: 22,
            }),
        ],
    }];

    for case in cases {
        let lex = lex(&case.input).collect_vec();
        assert_eq!(case.expected, lex);
    }
}
