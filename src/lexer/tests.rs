use crate::lexer::{
    lexer::{get_lexer, TokenSpan},
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
            token: Token::Name {
                value: "a".into(),
            },
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
            token: Token::Name {
                value: "b".into(),
            },
        }),
    ];

    assert_eq!(lex, expected);
}
