pub mod error;

use error::{ParsingError, Type::OperatorNakedRight};
use itertools::PeekNth;

use crate::{
    ast::{operator::BinaryOperator, untyped},
    lex::{
        error::LexicalError,
        lexer::{LexResult, TokenSpan},
        location::Location,
        token::Token,
    },
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

    pub fn parse_expression(&mut self) -> Result<Option<untyped::Expression>, ParsingError> {
        let mut operator_stack = vec![];
        let mut expression_stack = vec![];
        let mut last_operator_start = 0;
        let mut last_operator_end = 0;

        loop {
            match self.parse_expression_unit()? {
                Some(unit) => expression_stack.push(unit),
                _ if expression_stack.is_empty() => return Ok(None),
                _ => {
                    return Err(ParsingError {
                        error: OperatorNakedRight,
                        location: Location {
                            start: last_operator_start,
                            end: last_operator_end,
                        },
                    });
                }
            }

            if let Some(token_span) = self.current_token.take() {
                // if it has precedence, it is a binary operator
                if let Some(precedence) = get_precedence(&token_span.token) {
                    let _ = self.advance_token();

                    last_operator_start = token_span.start;
                    last_operator_end = token_span.end;

                    let _ = handle_operator(
                        Some(OperatorToken {
                            token_span,
                            precedence,
                        }),
                        &mut operator_stack,
                        &mut expression_stack,
                    );
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(handle_operator(
            None,
            &mut operator_stack,
            &mut expression_stack,
        ))
    }

    fn parse_expression_unit(&mut self) -> Result<Option<untyped::Expression>, ParsingError> {
        let mut expression = match &self.current_token {
            Some(token_span) => match &token_span.token {
                Token::Name { value } => todo!(),
                Token::IntLiteral { value } => todo!(),
                Token::FloatLiteral { value } => todo!(),
                Token::StringLiteral { value } => todo!(),
                Token::CharLiteral { value } => todo!(),
                Token::LeftParenthesis => todo!(),
                Token::RightParenthesis => todo!(),
                Token::LeftSquare => todo!(),
                Token::RightSquare => todo!(),
                Token::LeftBrace => todo!(),
                Token::RightBrace => todo!(),
                Token::Plus => todo!(),
                Token::Minus => todo!(),
                Token::Asterisk => todo!(),
                Token::Slash => todo!(),
                Token::PlusPlus => todo!(),
                Token::MinusMinus => todo!(),
                Token::Less => todo!(),
                Token::Greater => todo!(),
                Token::LessEqual => todo!(),
                Token::GreaterEqual => todo!(),
                Token::Percent => todo!(),
                Token::PlusFloat => todo!(),
                Token::MinusFloat => todo!(),
                Token::AsteriskFloat => todo!(),
                Token::SlashFloat => todo!(),
                Token::LessFloat => todo!(),
                Token::GreaterFloat => todo!(),
                Token::LessEqualFloat => todo!(),
                Token::GreaterEqualFloat => todo!(),
                Token::Concat => todo!(),
                Token::Colon => todo!(),
                Token::Comma => todo!(),
                Token::Bang => todo!(),
                Token::Equal => todo!(),
                Token::EqualEqual => todo!(),
                Token::NotEqual => todo!(),
                Token::Pipe => todo!(),
                Token::PipePipe => todo!(),
                Token::Ampersand => todo!(),
                Token::AmpersandAmpersand => todo!(),
                Token::LessLess => todo!(),
                Token::GreaterGreater => todo!(),
                Token::Dot => todo!(),
                Token::Comment => todo!(),
                Token::EndOfFile => todo!(),
                Token::NewLine => todo!(),
                Token::Int => todo!(),
                Token::Float => todo!(),
                Token::Char => todo!(),
                Token::String => todo!(),
                Token::Var => todo!(),
                Token::If => todo!(),
                Token::Else => todo!(),
                Token::Func => todo!(),
                Token::For => todo!(),
                Token::While => todo!(),
                Token::Return => todo!(),
                Token::Exit => todo!(),
                Token::Panic => todo!(),
                Token::Todo => todo!(),
            },
            None => todo!(),
        };
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
            Some(Err(lexical_error)) => {
                self.lexical_errors.push(*lexical_error);
                None
            }
            None => None,
        }
    }
}

fn get_precedence(token: &Token) -> Option<u8> {
    token_to_binary_operator(token).map(|operator| operator.get_precedence())
}

fn token_to_binary_operator(token: &Token) -> Option<BinaryOperator> {
    match token {
        Token::Plus => Some(BinaryOperator::AdditionInt),
        Token::Minus => Some(BinaryOperator::SubtractionInt),
        Token::Asterisk => Some(BinaryOperator::MultipicationInt),
        Token::Slash => Some(BinaryOperator::DivisionInt),
        Token::Less => Some(BinaryOperator::LessInt),
        Token::Greater => Some(BinaryOperator::GreaterInt),
        Token::LessEqual => Some(BinaryOperator::LessEqualInt),
        Token::GreaterEqual => Some(BinaryOperator::GreaterEqualInt),
        Token::Percent => Some(BinaryOperator::Modulo),
        Token::PlusFloat => Some(BinaryOperator::AdditionFloat),
        Token::MinusFloat => Some(BinaryOperator::SubtractionFloat),
        Token::AsteriskFloat => Some(BinaryOperator::MultipicationFloat),
        Token::SlashFloat => Some(BinaryOperator::DivisionFloat),
        Token::LessFloat => Some(BinaryOperator::LessFloat),
        Token::GreaterFloat => Some(BinaryOperator::GreaterFloat),
        Token::LessEqualFloat => Some(BinaryOperator::LessEqualFloat),
        Token::GreaterEqualFloat => Some(BinaryOperator::GreaterEqualFloat),
        Token::Concat => Some(BinaryOperator::Concatenation),
        Token::EqualEqual => Some(BinaryOperator::Equal),
        Token::NotEqual => Some(BinaryOperator::NotEqual),
        Token::PipePipe => Some(BinaryOperator::Or),
        Token::AmpersandAmpersand => Some(BinaryOperator::And),
        // TODO: add others
        _ => None,
    }
}

struct OperatorToken {
    token_span: TokenSpan,
    precedence: u8,
}

fn handle_operator<T>(
    operator_token: Option<OperatorToken>,
    operator_stack: &mut Vec<OperatorToken>,
    expression_stack: &mut Vec<T>,
) -> Option<T> {
    let mut operator_token = operator_token;

    loop {
        match (operator_stack.pop(), operator_token.take()) {
            (Some(lhs), Some(rhs)) => match lhs.precedence.cmp(&rhs.precedence) {
                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => todo!(),
                std::cmp::Ordering::Less => {
                    operator_stack.push(lhs);
                    operator_stack.push(rhs);

                    break;
                }
            },
            (Some(_), None) => {}
            (None, Some(operator_token)) => {
                operator_stack.push(operator_token);
                break;
            }
            (None, None) => {
                if let Some(expression) = expression_stack.pop() {
                    if expression_stack.is_empty() {
                        return Some(expression);
                    } else {
                        unreachable!();
                    }
                } else {
                    return None;
                }
            }
        }
    }

    None
}
