pub mod error;

use error::{ParsingError, Type::OperatorNakedRight};
use itertools::PeekNth;
use vec1::{vec1, Vec1};

use crate::{
    ast::{
        argument, location::Location as AstLocation, operator::BinaryOperator,
        statement::Statement, untyped,
    },
    lex::{
        error::LexicalError,
        lexer::{LexResult, TokenSpan},
        location::Location as LexLocation,
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
                        location: LexLocation {
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
                    // TODO: figure whether to advance here?
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
        let mut expression = match self.current_token.take() {
            Some(token_span) => match token_span.token {
                Token::Name { value } => todo!(),
                Token::IntLiteral { value } => {
                    let _ = self.advance_token();
                    untyped::Expression::Int {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    }
                }
                Token::FloatLiteral { value } => {
                    let _ = self.advance_token();
                    untyped::Expression::Float {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    }
                }
                Token::StringLiteral { value } => {
                    let _ = self.advance_token();
                    untyped::Expression::String {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    }
                }
                Token::CharLiteral { value } => {
                    let _ = self.advance_token();
                    untyped::Expression::Char {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    }
                }
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
                Token::Func => {
                    let _ = self.advance_token();
                    self.parse_function()?
                }
                Token::For => todo!(),
                Token::While => todo!(),
                Token::Return => {
                    let _ = self.advance_token();

                    let mut value = None;
                    let mut end = token_span.end;

                    if let Some(expression) = self.parse_expression_unit()? {
                        end = expression.get_location().end;
                        value = Some(Box::new(expression));
                    }

                    untyped::Expression::Return {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    }
                }
                Token::Exit => {
                    let _ = self.advance_token();
                    untyped::Expression::Exit {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }
                }
                Token::Panic => {
                    let _ = self.advance_token();
                    untyped::Expression::Panic {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }
                }
                Token::Todo => {
                    let _ = self.advance_token();
                    untyped::Expression::Todo {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }
                }
            },
            None => todo!(),
        };

        todo!()
    }

    fn parse_function(&mut self) -> Result<untyped::Expression, ParsingError> {
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let name = if let Token::Name { value } = name_token_span.token {
            value
        } else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: vec!["function name".to_string().into()],
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let _ = self.expect_token(&Token::LeftParenthesis)?;

        let arguments = self.parse_series(&Self::parse_function_parameter, Some(&Token::Comma))?;

        self.expect_token(&Token::RightParenthesis)?;

        let return_annotation = self.parse_type_annotation()?;

        let (body, end) = match self.maybe_token(&Token::LeftBrace) {
            Some(left_brace_token_span) => {
                let some_body = self.parse_statement_sequence()?;

                let right_brace_token_span = self.expect_token(&Token::RightBrace)?;
                let end_location = right_brace_token_span.end;

                let body = match some_body {
                    Some((body, _)) => body,
                    None => {
                        vec1![Statement::Expression(untyped::Expression::Todo {
                            location: AstLocation {
                                start: left_brace_token_span.start,
                                end: end_location
                            },
                        })]
                    }
                };

                (body, end_location)
            }
            None => todo!(),
        };

        Ok(untyped::Expression::Func {
            location: AstLocation {
                start: name_token_span.start,
                end,
            },
            name,
            arguments,
            body,
            return_annotation,
        })
    }

    fn parse_function_parameter(&mut self) -> Result<Option<argument::Untyped>, ParsingError> {
        todo!()
    }

    fn parse_series<A>(
        &mut self,
        parser: &impl Fn(&mut Self) -> Result<Option<A>, ParsingError>,
        separator: Option<&Token>,
    ) -> Result<Vec<A>, ParsingError> {
        let mut results = vec![];

        while let Some(result) = parser(self)? {
            results.push(result);

            if let Some(separator) = separator {
                if self.maybe_token(separator).is_none() {
                    break;
                }
            }
        }

        Ok(results)
    }

    fn parse_type_annotation(&mut self) -> Result<Option<crate::type_::Type>, ParsingError> {
        match self.current_token.take() {
            Some(token_span) => match token_span.token {
                Token::Func => todo!(),
                Token::Var => todo!(),
                _ => todo!(),
            },
            None => todo!(),
        }
    }

    fn expect_token(&mut self, token: &Token) -> Result<TokenSpan, ParsingError> {
        match self.maybe_token(token) {
            Some(token_span) => Ok(token_span),
            None => match self.current_token.take() {
                Some(current_token) => Err(ParsingError {
                    error: error::Type::UnexpectedToken {
                        token: current_token.token,
                        expected: vec![token.to_string().into()],
                    },
                    location: LexLocation {
                        start: current_token.start,
                        end: current_token.end,
                    },
                }),
                None => Err(ParsingError {
                    error: error::Type::UnexpectedEof,
                    location: LexLocation { start: 0, end: 0 },
                }),
            },
        }
    }

    fn maybe_token(&mut self, token: &Token) -> Option<TokenSpan> {
        match self.current_token.take() {
            Some(token_span) if token_span.token == *token => {
                let _ = self.advance_token();
                Some(token_span)
            }
            other => {
                self.current_token = other;
                None
            }
        }
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

    fn parse_statement_sequence(
        &self,
    ) -> Result<Option<(Vec1<crate::ast::statement::Untyped>, u32)>, ParsingError> {
        todo!()
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
                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                    operator_token = Some(rhs)
                }
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
