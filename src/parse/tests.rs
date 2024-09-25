use crate::parse::{
    lexer::{self, get_lexer},
    token::Token,
};

use itertools::Itertools;
use pretty_assertions::assert_eq;

#[test]
fn test_lex_int32_var() {
    let input = "var name int32 =";
    let lex = get_lexer(&input).collect_vec();

    let expected = [
        Ok(lexer::TokenSpan {
            token: Token::Var,
            start: 0,
            end: 3,
        }),
        Ok(lexer::TokenSpan {
            token: Token::Name {
                value: "name".into(),
            },
            start: 4,
            end: 8,
        }),
        Ok(lexer::TokenSpan {
            token: Token::Int32,
            start: 9,
            end: 14,
        }),
        Ok(lexer::TokenSpan {
            token: Token::Equal,
            start: 15,
            end: 15,
        }),
    ];

    assert_eq!(lex, expected);
}
