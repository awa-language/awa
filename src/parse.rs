use itertools::PeekNth;

use crate::lex::{
    lexer::{LexResult, TokenSpan},
    lexical_error::LexicalError,
};

pub struct Parser<T: Iterator<Item = LexResult>> {
    input_tokens: PeekNth<T>,
    lexical_errors: Vec<LexicalError>,
    current_token: Option<TokenSpan>,
}

impl<T: Iterator<Item = LexResult>> Parser<T> {
    pub fn new(
        tokens: PeekNth<T>,
        lexical_errors: Vec<LexicalError>,
        current_char: Option<TokenSpan>,
    ) -> Self {
        let mut parser = Parser {
            input_tokens: tokens,
            lexical_errors,
            current_token: current_char,
        };

        let _ = parser.advance_token();

        parser
    }

    fn advance_token(&mut self) -> Option<TokenSpan> {
        let token = self.current_token.clone();

        match self.input_tokens.next() {
            Some(Ok(token)) => {
                self.current_token = Some(token);
            }
            Some(Err(lexical_error)) => {
                self.lexical_errors.push(lexical_error);
                self.current_token = None;
            }
            None => {
                self.current_token = None;
            }
        }

        token
    }

    fn peek_char(&mut self) -> Option<TokenSpan> {
        match self.input_tokens.peek_nth(0) {
            Some(Ok(token)) => Some(token.clone()),
            // TODO: it may insert the same lexical error twice, need tests
            // it may as well not be an issue
            Some(Err(lexical_error)) => {
                self.lexical_errors.push(*lexical_error);
                None
            }
            None => None,
        }
    }
}
