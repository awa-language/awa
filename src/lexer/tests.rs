use crate::lexer::{
    lexer::{get_lexer, TokenSpan},
    lexical_error::{LexicalError, Type},
    location::Location,
    token::Token,
};

use itertools::Itertools;
use pretty_assertions::assert_eq;

#[test]
fn test_int32_var() {
    let input = "var name int32 =";
    let lex = get_lexer(&input).collect_vec();

    let expected = [
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
    ];

    assert_eq!(lex, expected);
}

#[test]
fn test_newlines() {
    let input = "a\r\n\nb";
    let lex = get_lexer(&input).collect_vec();

    let expected = [
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
            end: 4,
            token: Token::Name { value: "b".into() },
        }),
    ];

    assert_eq!(lex, expected);
}

#[test]
fn test_int_lexing() {
    let input = "123";
    let lex = get_lexer(&input).collect_vec();

    let expected = [Ok(TokenSpan {
        token: Token::IntLiteral {
            value: "123".into(),
        },
        start: 0,
        end: 2,
    })];

    assert_eq!(lex, expected);
}

#[test]
fn test_number_lexing_failed() {
    let inputs = ["123.", "123.."];
    let expected_errors = vec![
        vec![Err(LexicalError {
            error: Type::InvalidNumberFormat,
            location: Location { start: 3, end: 3 },
        })],
        vec![
            Err(LexicalError {
                error: Type::InvalidNumberFormat,
                location: Location { start: 0, end: 4 },
            }),
        ],
    ];

    for (i, input) in inputs.iter().enumerate() {
        let lex: Vec<Result<TokenSpan, LexicalError>> = get_lexer(input).collect();
        assert_eq!(lex, expected_errors[i], "Test failed for input: {}", input);
    }
}

#[test]
fn test_float_lexing() {
    let input = "123.123";
    let lex = get_lexer(&input).collect_vec();

    let expected = [Ok(TokenSpan {
        token: Token::FloatLiteral {
            value: "123.123".into(),
        },
        start: 0,
        end: 6,
    })];

    assert_eq!(lex, expected)
}
