pub mod error;

#[cfg(test)]
pub mod tests;

use ecow::EcoString;
use error::{ParsingError, Type::MissingRightOperand};
use itertools::{peek_nth, PeekNth};
use vec1::Vec1;

use crate::ast::definition::StructField;
use crate::ast::location::Location;
use crate::{
    ast::{
        argument,
        assignment::Assignment,
        definition, expression,
        location::Location as AstLocation,
        module,
        operator::BinaryOperator,
        statement::{self, Statement},
    },
    lex::{
        error::LexicalError,
        lexer::{self, LexResult, TokenSpan},
        location::Location as LexLocation,
        token::Token,
    },
    type_::Type,
};

pub fn parse_module(input: &str) -> Result<module::Untyped, ParsingError> {
    let tokens = lexer::lex(input);

    let mut parser = Parser::new(peek_nth(tokens));
    let module = parser.parse_module()?;

    Ok(module)
}

pub fn parse_statement_sequence(input: &str) -> Result<Vec1<statement::Untyped>, ParsingError> {
    let lex = lexer::lex(input);

    let mut parser = Parser::new(peek_nth(lex));
    let statement_sequence = parser.parse_statement_sequence();

    let statement_sequence = parser.ensure_no_errors_or_remaining_tokens(statement_sequence)?;
    if let Some((e, _)) = statement_sequence {
        Ok(e)
    } else {
        Err(ParsingError {
            error: error::Type::ExpectedStatementSequence,
            location: LexLocation { start: 0, end: 0 },
        })
    }
}

pub struct Parser<T: Iterator<Item = LexResult>> {
    input_tokens: PeekNth<T>,
    lexical_errors: Vec<LexicalError>,
    current_token: Option<TokenSpan>,
}

impl<T: Iterator<Item = LexResult>> Parser<T> {
    pub fn new(tokens: PeekNth<T>) -> Self {
        let mut parser = Parser {
            input_tokens: tokens,
            lexical_errors: vec![],
            current_token: None,
        };

        let _ = parser.advance_token();

        parser
    }

    fn parse_module(&mut self) -> Result<module::Untyped, ParsingError> {
        let definitions = self.parse_series(&Self::parse_definition, None);
        let definitions = self.ensure_no_errors_or_remaining_tokens(definitions)?;

        Ok(module::Untyped {
            name: "".into(),
            definitions,
        })
    }

    fn ensure_no_errors_or_remaining_tokens<A>(
        &mut self,
        parse_result: Result<A, ParsingError>,
    ) -> Result<A, ParsingError> {
        let result = self.ensure_no_errors(parse_result)?;

        if let Some(_) = self.current_token {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: self.current_token.clone().unwrap().token,
                    expected: "function or struct definitions".to_string().into(),
                },
                location: LexLocation {
                    start: self.current_token.clone().unwrap().start,
                    end: self.current_token.clone().unwrap().end,
                },
            });
        }

        Ok(result)
    }

    fn ensure_no_errors<A>(&mut self, result: Result<A, ParsingError>) -> Result<A, ParsingError> {
        if let Some(error) = self.lexical_errors.first() {
            Err(ParsingError {
                error: error::Type::LexicalError { error: *error },
                location: error.location,
            })
        } else {
            result
        }
    }

    fn parse_definition(&mut self) -> Result<Option<definition::Untyped>, ParsingError> {
        match self.current_token.take() {
            // NOTE: no global variables because i'm too lazy to add variable definition
            // (now they are available only as an expression)
            Some(token_span) => match token_span.token {
                Token::Struct => match self.parse_struct_defenition() {
                    Ok(definition) => Ok(Some(definition)),
                    Err(parsing_error) => Err(parsing_error),
                },
                Token::Func => match self.parse_function_definition() {
                    Ok(definition) => Ok(Some(definition)),
                    Err(parsing_error) => Err(parsing_error),
                },
                _ => Err(ParsingError {
                    error: error::Type::UnexpectedToken {
                        token: self.current_token.clone().unwrap().token,
                        expected: "either function or struct definition".to_string().into(),
                    },
                    location: LexLocation {
                        start: self.current_token.clone().unwrap().start,
                        end: self.current_token.clone().unwrap().end,
                    },
                }),
            },
            None => Ok(None),
        }
    }

    fn parse_expression(&mut self) -> Result<Option<expression::Expression>, ParsingError> {
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
                        error: MissingRightOperand,
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
                            _token_span: token_span,
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

    fn parse_expression_unit(&mut self) -> Result<Option<expression::Expression>, ParsingError> {
        let expression_unit = match self.current_token.take() {
            Some(token_span) => match token_span.token {
                //   name can be either:
                // - variable value access (varName)
                // - function call (`funcName()` or `funcName(argFirst, argSecond)`)
                // - struct field access (`structName.fieldName`)
                // - array value access (`arrayName[indexVariable]` or `arrayName[1]`)
                Token::Name { value } => {
                    let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
                        error: error::Type::UnexpectedEof,
                        location: LexLocation { start: 0, end: 0 },
                    })?;

                    // function call (`funcName()` or `funcName(argFirst, argSecond)`)
                    if let Some(left_parenthesis_token_span) =
                        self.maybe_token(&Token::LeftParenthesis)
                    {
                        let call_arguments = self.parse_series(
                            &Self::parse_function_call_argument,
                            Some(&Token::Comma),
                        )?;
                        let right_paren_token_span = self.expect_token(&Token::RightParenthesis)?;

                        return Ok(Some(expression::Expression::FunctionCall {
                            location: Location {
                                start: name_token_span.start,
                                end: right_paren_token_span.end,
                            },
                            function_name: value.into(),
                            arguments: call_arguments,
                        }));
                    }

                    // struct field access (`structName.fieldName`)
                    if let Some(field_name_token_span) = self.maybe_token(&Token::Dot) {
                        let Token::Name { value: _field_name } = field_name_token_span.token else {
                            return Err(ParsingError {
                                error: error::Type::UnexpectedToken {
                                    token: field_name_token_span.token,
                                    expected: "field name".to_string().into(),
                                },
                                location: LexLocation {
                                    start: field_name_token_span.start,
                                    end: field_name_token_span.end,
                                },
                            });
                        };

                        return Ok(Some(expression::Expression::StructFieldAccess {
                            location: Location {
                                start: name_token_span.start,
                                end: field_name_token_span.end,
                            },
                            struct_name: value.into(),
                            field_name: _field_name.into(),
                        }));
                    }

                    // array value access (`arrayName[indexVariable]` or `arrayName[1]`)
                    if let Some(left_brace_token_span) = self.maybe_token(&Token::LeftBrace) {
                        let index_expression = self.parse_expression()?;
                        let right_bracket_token_span = self.expect_token(&Token::RightBrace)?;

                        return Ok(Some(expression::Expression::ArrayIndexAccess {
                            location: Location {
                                start: name_token_span.start,
                                end: right_bracket_token_span.end,
                            },
                            array_name: value.into(),
                            index_expression: Box::new(index_expression.unwrap()),
                        }));
                    }

                    // variable value access (varName)
                    return Ok(Some(expression::Expression::VariableValue {
                        location: Location {
                            start: name_token_span.start,
                            end: name_token_span.end,
                        },
                        name: value.into(),
                    }));
                }

                Token::IntLiteral { value } => {
                    let _ = self.advance_token();
                    Some(expression::Expression::IntLiteral {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    })
                }
                Token::FloatLiteral { value } => {
                    let _ = self.advance_token();
                    Some(expression::Expression::FloatLiteral {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    })
                }
                Token::StringLiteral { value } => {
                    let _ = self.advance_token();
                    Some(expression::Expression::StringLiteral {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    })
                }
                Token::CharLiteral { value } => {
                    let _ = self.advance_token();
                    Some(expression::Expression::CharLiteral {
                        location: AstLocation {
                            start: token_span.start,
                            end: token_span.end,
                        },
                        value,
                    })
                }
                _ => {
                    self.current_token = Some(token_span);
                    None
                }
            },
            None => {
                return Err(ParsingError {
                    error: error::Type::UnexpectedEof,
                    location: LexLocation { start: 0, end: 0 },
                })
            }
        };

        Ok(expression_unit)
    }

    fn parse_function_call(
        &mut self,
        function_name: &EcoString,
        start_location: u32,
    ) -> Result<expression::Expression, ParsingError> {
        // TODO: this would most certainly be not here, but on the caller side
        let _ = self.expect_token(&Token::LeftParenthesis)?;

        let call_arguments =
            self.parse_series(&Self::parse_function_call_argument, Some(&Token::Comma))?;

        let right_parenthesis_token_span = self.expect_token(&Token::RightParenthesis)?;

        Ok(expression::Expression::FunctionCall {
            location: AstLocation {
                start: start_location,
                end: right_parenthesis_token_span.end,
            },
            function_name: function_name.clone(),
            arguments: call_arguments,
        })
    }

    fn parse_struct_defenition(&mut self) -> Result<definition::Untyped, ParsingError> {
        let _ = self.advance_token();
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let Token::Name { value: name } = name_token_span.token else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: "struct name".to_string().into(),
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let _ = self.expect_token(&Token::LeftBrace)?;

        let fields = self.parse_series(&Self::parse_struct_field, None)?;

        let right_brace_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let fields = match Vec1::try_from_vec(fields) {
            Ok(fields) => Some(fields),
            Err(_) => None,
        };

        Ok(definition::Untyped::Struct {
            location: AstLocation {
                start: name_token_span.start,
                end: right_brace_token_span.end,
            },
            name,
            fields,
        })
    }

    fn parse_function_definition(&mut self) -> Result<definition::Untyped, ParsingError> {
        let _ = self.advance_token();
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let Token::Name { value: name } = name_token_span.token else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: "function name".to_string().into(),
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let _ = self.expect_token(&Token::LeftParenthesis)?;

        let arguments = match self.maybe_token(&Token::RightParenthesis) {
            Some(_) => None,
            None => {
                let args =
                    self.parse_series(&Self::parse_function_argument, Some(&Token::Comma))?;
                Some(Vec1::try_from_vec(args).unwrap())
            }
        };
        if arguments.is_some() {
            self.expect_token(&Token::RightParenthesis)?;
        }

        let return_type_annotation = self.parse_type_annotation()?;

        let (body, end) = match self.maybe_token(&Token::LeftBrace) {
            Some(_) => {
                let some_body = self.parse_statement_sequence()?;
                let right_brace_token_span = self.expect_token(&Token::RightBrace)?;
                let end_location = right_brace_token_span.end;
                let body = match some_body {
                    Some((body, _)) => Some(body),
                    None => None,
                };
                Ok((body, end_location))
            }
            None => Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: self.current_token.clone().unwrap().token,
                    expected: "opening function brace `{`".to_string().into(),
                },
                location: LexLocation {
                    start: self.current_token.clone().unwrap().start,
                    end: self.current_token.clone().unwrap().end,
                },
            }),
        }?;

        Ok(definition::Untyped::Function {
            location: AstLocation {
                start: name_token_span.start,
                end,
            },
            name,
            arguments,
            body,
            return_type_annotation,
        })
    }

    fn parse_function_call_argument(
        &mut self,
    ) -> Result<Option<argument::CallArgument<expression::Expression>>, ParsingError> {
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let Token::Name {
            value: _argument_name,
        } = name_token_span.token
        else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: "argument name".to_string().into(),
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let expression = self.parse_expression()?.ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        Ok(Some(argument::CallArgument {
            location: Location {
                start: name_token_span.start,
                end: expression.get_location().end,
            },
            value: expression,
        }))
    }

    fn parse_function_argument(&mut self) -> Result<Option<argument::Untyped>, ParsingError> {
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let Token::Name { value: name } = name_token_span.token else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: "function argument name".to_string().into(),
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let type_annotation = self.parse_type_annotation()?.ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedToken {
                token: self.current_token.clone().unwrap().token,
                expected: "function argument type annotation".to_string().into(),
            },
            location: LexLocation {
                start: self.current_token.clone().unwrap().start,
                end: self.current_token.clone().unwrap().end,
            },
        })?;

        Ok(Some(argument::Untyped {
            name: argument::Name::Named {
                name,
                location: Location {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            },
            location: Location {
                start: name_token_span.start,
                end: self.current_token.clone().unwrap().start,
            },
            annotation: Some(type_annotation),
            type_: (),
        }))
    }

    fn parse_struct_field(&mut self) -> Result<Option<StructField>, ParsingError> {
        let name_token_span = self.current_token.clone();
        let name_token_span = match name_token_span {
            Some(name_token_span) => match name_token_span.token {
                Token::Name { .. } => name_token_span,
                _ => return Ok(None),
            },
            None => return Ok(None),
        };

        let _ = self.advance_token();

        let Token::Name { value: name } = name_token_span.token else {
            return Ok(None);
        };

        let type_annotation = self.parse_type_annotation()?.ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedToken {
                token: self.current_token.clone().unwrap().token,
                expected: "struct field type annotation".to_string().into(),
            },
            location: LexLocation {
                start: self.current_token.clone().unwrap().start,
                end: self.current_token.clone().unwrap().end,
            },
        })?;

        Ok(Some(StructField {
            name,
            type_annotation,
        }))
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

    fn parse_statement_sequence(
        &mut self,
    ) -> Result<Option<(Vec1<statement::Untyped>, u32)>, ParsingError> {
        let mut statements = vec![];
        let mut start = None;
        let mut end = 0;

        while let Some(statement) = self.parse_statement()? {
            if start.is_none() {
                start = Some(statement.get_location().start);
            }

            end = statement.get_location().end;
            statements.push(statement);
        }

        match Vec1::try_from_vec(statements) {
            Ok(statements) => Ok(Some((statements, end))),
            Err(_) => Ok(None),
        }
    }

    fn parse_statement(&mut self) -> Result<Option<statement::Untyped>, ParsingError> {
        match self.current_token.take() {
            Some(token_span) => match token_span.token {
                Token::Var => {
                    let _ = self.advance_token();
                    Ok(Some(self.parse_assignment(token_span.start)?))
                }
                Token::Loop => {
                    let _ = self.advance_token();
                    let _ = self.expect_token(&Token::LeftBrace)?;

                    let body = match self.parse_statement_sequence()? {
                        Some((statements, _)) => Some(statements),
                        None => None,
                    };

                    let right_brace_token_span = self.expect_token(&Token::RightBrace)?;
                    let end = right_brace_token_span.end;

                    Ok(Some(Statement::Loop {
                        body,
                        location: Location {
                            start: token_span.start,
                            end,
                        },
                    }))
                }
                Token::If => {
                    let _ = self.advance_token();
                    let _ = self.expect_token(&Token::LeftParenthesis)?;
                    let condition = self.parse_expression()?;
                    let _ = self.expect_token(&Token::RightParenthesis)?;

                    let _ = self.expect_token(&Token::LeftBrace)?;
                    let if_body = match self.parse_statement_sequence()? {
                        Some((statements, _)) => Some(statements),
                        None => None,
                    };

                    let right_brace_token_span = self.expect_token(&Token::RightBrace)?;
                    let mut end = right_brace_token_span.end;

                    let else_body =
                        if let Some(Token::Else) = self.current_token.as_ref().map(|t| &t.token) {
                            self.advance_token();
                            let _ = self.expect_token(&Token::LeftBrace)?;

                            let else_statements = match self.parse_statement_sequence()? {
                                Some((statements, _)) => Some(statements),
                                None => None,
                            };

                            let else_right_brace = self.expect_token(&Token::RightBrace)?;
                            end = else_right_brace.end;

                            else_statements
                        } else {
                            None
                        };

                    Ok(Some(Statement::If {
                        condition: Box::new(condition.unwrap()),
                        if_body,
                        else_body,
                        location: Location {
                            start: token_span.start,
                            end,
                        },
                    }))
                }
                Token::Return => {
                    let _ = self.advance_token();
                    let mut value = None;
                    let mut end = token_span.end;

                    if let Some(expression) = self.parse_expression()? {
                        end = expression.get_location().end;
                        value = Some(Box::new(expression));
                    }

                    Ok(Some(Statement::Return {
                        value,
                        location: Location {
                            start: token_span.start,
                            end,
                        },
                    }))
                }
                Token::Todo => {
                    let _ = self.advance_token();
                    Ok(Some(Statement::Todo {
                        location: Location {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }))
                }
                Token::Break => {
                    let _ = self.advance_token();
                    Ok(Some(Statement::Break {
                        location: Location {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }))
                }
                Token::Panic => {
                    let _ = self.advance_token();
                    Ok(Some(Statement::Panic {
                        location: Location {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }))
                }
                Token::Exit => {
                    let _ = self.advance_token();
                    Ok(Some(Statement::Exit {
                        location: Location {
                            start: token_span.start,
                            end: token_span.end,
                        },
                    }))
                }
                _ => {
                    self.current_token = Some(token_span);
                    let expression = self.parse_expression()?.map(Statement::Expression);
                    Ok(expression)
                }
            },
            None => Ok(None),
        }
    }

    fn parse_assignment(&mut self, start: u32) -> Result<statement::Untyped, ParsingError> {
        let name_token_span = self.advance_token().ok_or_else(|| ParsingError {
            error: error::Type::UnexpectedEof,
            location: LexLocation { start: 0, end: 0 },
        })?;

        let Token::Name { value: ref name } = name_token_span.token else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: name_token_span.token,
                    expected: "variable name".to_string().into(),
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        if let "true" | "false" = name.as_str() {
            return Err(ParsingError {
                error: error::Type::InvalidName {
                    token: name_token_span.token,
                },
                location: LexLocation {
                    start: name_token_span.start,
                    end: name_token_span.end,
                },
            });
        };

        let Some(type_annotation) = self.parse_type_annotation()? else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: self.current_token.clone().unwrap().token,
                    expected: "variable type annotation".to_string().into(),
                },
                location: LexLocation {
                    start: self.current_token.clone().unwrap().start,
                    end: self.current_token.clone().unwrap().end,
                },
            });
        };

        let _ = self.expect_token(&Token::Equal)?;

        let Some(value) = self.parse_expression()? else {
            return Err(ParsingError {
                error: error::Type::UnexpectedToken {
                    token: self.current_token.clone().unwrap().token,
                    expected: "variable assignment expression".to_string().into(),
                },
                location: LexLocation {
                    start: self.current_token.clone().unwrap().start,
                    end: self.current_token.clone().unwrap().end,
                },
            });
        };
        let end = value.get_location().end;

        Ok(Statement::Assignment(Assignment {
            location: AstLocation { start, end },
            value: Box::new(value),
            annotation: type_annotation,
        }))
    }

    fn parse_type_annotation(&mut self) -> Result<Option<Type>, ParsingError> {
        match self.current_token.clone() {
            Some(token_span) => match token_span.token {
                Token::Int => {
                    let _ = self.advance_token();
                    Ok(Some(Type::Int))
                }
                Token::Float => {
                    let _ = self.advance_token();
                    Ok(Some(Type::Float))
                }
                Token::String => {
                    let _ = self.advance_token();
                    Ok(Some(Type::String))
                }
                Token::Char => {
                    let _ = self.advance_token();
                    Ok(Some(Type::Char))
                }
                Token::Name { value } => {
                    let name = value;
                    let _ = self.advance_token();
                    Ok(Some(Type::Custom { name, fields: None }))
                }
                Token::LeftSquare => {
                    self.advance_token();
                    let _ = self.expect_token(&Token::RightSquare)?;

                    let Some(array_type) = self.parse_type_annotation()? else {
                        return Err(ParsingError {
                            error: error::Type::UnexpectedToken {
                                token: self.current_token.clone().unwrap().token,
                                expected: "right square".to_string().into(),
                            },
                            location: LexLocation {
                                start: self.current_token.clone().unwrap().start,
                                end: self.current_token.clone().unwrap().end,
                            },
                        });
                    };

                    Ok(Some(Type::Array {
                        type_: Box::new(array_type),
                    }))
                }
                _ => Ok(None),
            },
            None => Err(ParsingError {
                error: error::Type::UnexpectedEof,
                location: LexLocation { start: 0, end: 0 },
            }),
        }
    }

    fn expect_token(&mut self, token: &Token) -> Result<TokenSpan, ParsingError> {
        match self.maybe_token(token) {
            Some(token_span) => Ok(token_span),
            None => match self.current_token.take() {
                Some(current_token) => Err(ParsingError {
                    error: error::Type::UnexpectedToken {
                        token: current_token.token,
                        expected: token.to_string().into(),
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

        loop {
            match self.input_tokens.next() {
                Some(Ok(TokenSpan {
                    token: Token::Comment | Token::NewLine,
                    ..
                })) => {
                    continue;
                }
                Some(Ok(token)) => {
                    self.current_token = Some(token);
                    break;
                }
                Some(Err(lexical_error)) => {
                    self.lexical_errors.push(lexical_error);
                    self.current_token = None;
                    break;
                }
                None => {
                    self.current_token = None;
                    break;
                }
            }
        }

        token
    }

    fn peek_token(&mut self) -> Option<TokenSpan> {
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
    _token_span: TokenSpan,
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
                    operator_token = Some(rhs);
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
                    }
                } else {
                    return None;
                }
            }
        }
    }

    None
}
