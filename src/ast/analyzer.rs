use std::collections::HashMap;

use super::argument::{CallArgumentTyped, CallArgumentUntyped};
use super::module;
use super::reassignment::{TypedReassignment, TypedReassignmentTarget};
use crate::ast;
use crate::ast::argument::{ArgumentTyped, ArgumentUntyped};
use crate::ast::assignment::TypedAssignment;
use crate::ast::definition::{DefinitionTyped, DefinitionUntyped, StructField, StructFieldTyped};
use crate::ast::expression::{
    StructFieldValue, StructFieldValueTyped, TypedExpression, UntypedExpression,
};
use crate::ast::module::Module;
use crate::ast::operator::BinaryOperator;
use crate::ast::reassignment::UntypedReassignmentTarget;
use crate::ast::statement::{TypedStatement, UntypedStatement};
use crate::lex::location::Location;
use crate::parse::error::{ConvertingError, ConvertingErrorType};
use crate::parse::parse_module;
use crate::type_::{Type, UntypedType};
use ecow::EcoString;
use vec1::Vec1;

/// Analyzes input code for type correctness.
///
/// # Panics
///
/// - When parser fails to parse input string (unwrap on `parse_module`)
///
/// # Errors
///
/// Returns `ConvertingError` if type analysis fails
pub fn analyze_input(input: &str) -> Result<module::Typed, ConvertingError> {
    let module = parse_module(input).unwrap();

    let mut analyzer = TypeAnalyzer::new();
    let typed_module = analyzer.convert_ast_to_tast(&module)?;

    Ok(typed_module)
}

#[derive(Debug)]
pub struct TypeAnalyzer {
    program_state: ProgramState,
}

impl Default for TypeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        TypeAnalyzer {
            program_state: ProgramState::new(),
        }
    }

    /// Converts AST to typed AST
    ///
    /// # Errors
    /// Returns `ConvertingError` if:
    /// - Type checking fails
    /// - Unknown variable/function reference
    /// - Type mismatch
    pub fn convert_ast_to_tast(
        &mut self,
        untyped_ast: &Module<DefinitionUntyped>,
    ) -> Result<Module<DefinitionTyped>, ConvertingError> {
        let mut typed_definitions = None;

        if let Some(definitions) = &untyped_ast.definitions {
            for definition in definitions {
                self.program_state.clear_variables();
                let typed_definition = match definition {
                    DefinitionUntyped::Function {
                        location,
                        name,
                        arguments,
                        body,
                        return_type_annotation,
                    } => {
                        self.program_state.set_current_function_name(name);

                        let typed_args = arguments
                            .as_ref()
                            .map(|args| args.clone().try_mapped(|arg| self.convert_argument(&arg)))
                            .transpose()?;

                        if let Some(args) = typed_args.as_ref() {
                            for arg in args {
                                self.program_state
                                    .add_variable(arg.name.clone(), arg.type_.clone());
                            }
                        }

                        let return_type = return_type_annotation
                            .as_ref()
                            .map(|type_| {
                                self.convert_untyped_to_typed(type_, location.start, location.end)
                            })
                            .transpose()?
                            .unwrap_or(Type::Void);

                        let typed_function_without_body = DefinitionTyped::Function {
                            name: name.clone(),
                            location: *location,
                            arguments: typed_args.clone(),
                            body: None,
                            return_type: return_type.clone(),
                        };

                        self.program_state
                            .add_function(name.clone(), typed_function_without_body.clone());

                        let typed_body = body
                            .as_ref()
                            .map(|statements| {
                                statements.clone().try_mapped(|statement| {
                                    self.convert_statement_to_typed(&statement)
                                })
                            })
                            .transpose()?;

                        DefinitionTyped::Function {
                            name: name.clone(),
                            location: *location,
                            arguments: typed_args,
                            body: typed_body,
                            return_type,
                        }
                    }
                    DefinitionUntyped::Struct {
                        location,
                        name,
                        fields,
                    } => {
                        let typed_fields = fields
                            .as_ref()
                            .map(|fields| {
                                fields
                                    .clone()
                                    .try_mapped(|field| self.convert_struct_field(&field))
                            })
                            .transpose()?;

                        let typed_struct = DefinitionTyped::Struct {
                            location: *location,
                            name: name.clone(),
                            fields: typed_fields,
                        };

                        self.program_state
                            .add_struct(name.clone(), typed_struct.clone());

                        typed_struct
                    }
                };

                match &mut typed_definitions {
                    None => typed_definitions = Some(Vec1::new(typed_definition)),
                    Some(defs) => defs.push(typed_definition),
                }
            }
        }

        Ok(Module {
            name: untyped_ast.name.clone(),
            definitions: typed_definitions,
        })
    }

    fn convert_argument(
        &mut self,
        argument_untyped: &ArgumentUntyped,
    ) -> Result<ArgumentTyped, ConvertingError> {
        let typed_type = self.convert_untyped_to_typed(
            &argument_untyped.type_annotation,
            argument_untyped.location.start,
            argument_untyped.location.end,
        )?;

        Ok(ArgumentTyped {
            name: argument_untyped.name.clone(),
            location: argument_untyped.location,
            type_: typed_type,
        })
    }

    fn convert_struct_field(
        &mut self,
        field: &StructField,
    ) -> Result<StructFieldTyped, ConvertingError> {
        let resolved_type = self.convert_untyped_to_typed(&field.type_annotation, 0, 0)?;
        Ok(StructFieldTyped {
            name: field.name.clone(),
            type_: resolved_type,
        })
    }

    fn convert_struct_field_value(
        &mut self,
        struct_field_value: &StructFieldValue,
        struct_name: &EcoString,
    ) -> Result<StructFieldValueTyped, ConvertingError> {
        let typed_value = self.convert_expression_to_typed(&struct_field_value.value)?;
        let location = typed_value.get_location();

        let field_type = self.resolve_struct_field_type(struct_name, &struct_field_value.name)?;

        if !Self::compare_types(&field_type, typed_value.get_type()) {
            return Err(ConvertingError {
                error: ConvertingErrorType::TypeMismatch {
                    expected: field_type.clone(),
                    found: typed_value.get_type().clone(),
                },
                location: crate::lex::location::Location {
                    start: location.start,
                    end: location.end,
                },
            });
        }

        Ok(StructFieldValueTyped {
            name: struct_field_value.name.clone(),
            value: typed_value,
            type_: field_type,
        })
    }

    /// Converts untyped statement to typed statement
    ///
    /// # Panics
    /// # Errors
    /// Returns `ConvertingError` if:
    /// - Type mismatch between variable declaration and value
    /// - Unknown variable reference in reassignment
    /// - Array index not an integer type
    /// - Struct field access on invalid type
    /// - Variable used before declaration
    pub fn convert_statement_to_typed(
        &mut self,
        stmt: &UntypedStatement,
    ) -> Result<TypedStatement, ConvertingError> {
        match stmt {
            UntypedStatement::Expression(expression) => {
                let typed_expression = self.convert_expression_to_typed(expression)?;
                Ok(TypedStatement::Expression(typed_expression))
            }
            UntypedStatement::Assignment(assignment) => {
                let typed_value = self.convert_expression_to_typed(&assignment.value)?;
                let resolved_type = self.convert_untyped_to_typed(
                    &assignment.type_annotation,
                    assignment.location.start,
                    assignment.location.end,
                )?;

                if !Self::compare_types(&resolved_type, typed_value.get_type()) {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: resolved_type.clone(),
                            found: typed_value.get_type().clone(),
                        },
                        location: crate::lex::location::Location {
                            start: assignment.location.start,
                            end: assignment.location.end,
                        },
                    });
                }

                self.program_state
                    .add_variable(assignment.variable_name.clone(), resolved_type.clone());

                Ok(TypedStatement::Assignment(TypedAssignment {
                    location: assignment.location,
                    variable_name: assignment.variable_name.clone(),
                    value: Box::new(typed_value),
                    type_: resolved_type.clone(),
                }))
            }
            UntypedStatement::Reassignment(reassignment) => {
                let typed_new_value = self.convert_expression_to_typed(&reassignment.new_value)?;

                let typed_target = match &reassignment.target {
                    UntypedReassignmentTarget::Variable { location, name } => {
                        let var_type =
                            self.program_state
                                .get_variable_type(name)
                                .ok_or(ConvertingError {
                                    error: ConvertingErrorType::UnsupportedType,
                                    location: crate::lex::location::Location {
                                        start: location.start,
                                        end: location.end,
                                    },
                                })?;
                        TypedReassignmentTarget::Variable {
                            location: *location,
                            name: name.clone(),
                            type_: var_type.clone(),
                        }
                    }
                    UntypedReassignmentTarget::FieldAccess {
                        location,
                        struct_name,
                        field_name,
                    } => {
                        let struct_def_name =
                            match self.program_state.get_variable_type(struct_name) {
                                Some(Type::Custom { name }) => Ok(name),
                                _ => Err(ConvertingError {
                                    error: ConvertingErrorType::UnsupportedType,
                                    location: crate::lex::location::Location {
                                        start: location.start,
                                        end: location.end,
                                    },
                                }),
                            };
                        let field_type =
                            self.resolve_struct_field_type(struct_def_name.unwrap(), field_name)?;
                        TypedReassignmentTarget::FieldAccess {
                            location: *location,
                            struct_name: struct_name.clone(),
                            field_name: field_name.clone(),
                            type_: field_type.clone(),
                        }
                    }
                    UntypedReassignmentTarget::ArrayAccess {
                        location,
                        array_name,
                        index_expression,
                    } => {
                        let element_type = self.resolve_array_element_type(
                            array_name,
                            location.start,
                            location.end,
                        )?;
                        let typed_index = self.convert_expression_to_typed(index_expression)?;

                        if *typed_index.get_type() != Type::Int {
                            return Err(ConvertingError {
                                error: ConvertingErrorType::TypeMismatch {
                                    expected: Type::Int,
                                    found: typed_index.get_type().clone(),
                                },
                                location: crate::lex::location::Location {
                                    start: location.start,
                                    end: location.end,
                                },
                            });
                        }

                        TypedReassignmentTarget::ArrayAccess {
                            location: *location,
                            array_name: array_name.clone(),
                            index_expression: Box::new(typed_index),
                            type_: element_type.clone(),
                        }
                    }
                };

                if !Self::compare_types(&typed_target.get_type(), typed_new_value.get_type()) {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: typed_target.get_type(),
                            found: typed_new_value.get_type().clone(),
                        },
                        location: crate::lex::location::Location {
                            start: reassignment.location.start,
                            end: reassignment.location.end,
                        },
                    });
                }

                Ok(TypedStatement::Reassignment(TypedReassignment {
                    location: reassignment.location,
                    target: typed_target.clone(),
                    new_value: Box::new(typed_new_value),
                    type_: typed_target.get_type(),
                }))
            }
            UntypedStatement::Loop { body, location } => {
                let saved_scope = self.program_state.create_scope();
                let typed_body = body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;
                self.program_state.restore_scope(saved_scope);

                Ok(TypedStatement::Loop {
                    body: typed_body,
                    location: *location,
                })
            }
            UntypedStatement::If {
                condition,
                if_body,
                else_body,
                location,
            } => {
                let typed_condition = self.convert_expression_to_typed(condition)?;

                if *typed_condition.get_type() != Type::Boolean {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: Type::Boolean,
                            found: typed_condition.get_type().clone(),
                        },
                        location: Location {
                            start: location.start,
                            end: location.end,
                        },
                    });
                }

                let saved_scope = self.program_state.create_scope();
                let typed_if_body = if_body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;
                self.program_state.restore_scope(saved_scope);

                let saved_scope = self.program_state.create_scope();
                let typed_else_body = else_body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;
                self.program_state.restore_scope(saved_scope);

                Ok(TypedStatement::If {
                    condition: Box::new(typed_condition),
                    if_body: typed_if_body,
                    else_body: typed_else_body,
                    location: *location,
                })
            }
            UntypedStatement::Break { location } => Ok(TypedStatement::Break {
                location: *location,
            }),
            UntypedStatement::Return { location, value } => {
                let typed_value = value
                    .as_ref()
                    .map(|value| self.convert_expression_to_typed(value))
                    .transpose()?;

                let function_name = self.program_state.get_current_function_name();

                if let Some(function_def) = self.program_state.get_function(&function_name) {
                    let return_type = function_def.get_return_type()?;
                    if let Some(typed_value) = &typed_value {
                        if !Self::compare_types(&return_type, &typed_value.get_type()) {
                            return Err(ConvertingError {
                                error: ConvertingErrorType::TypeMismatch {
                                    expected: return_type.clone(),
                                    found: typed_value.get_type().clone(),
                                },
                                location: crate::lex::location::Location {
                                    start: location.start,
                                    end: location.end,
                                },
                            });
                        }
                    }
                }

                Ok(TypedStatement::Return {
                    location: *location,
                    value: typed_value.map(Box::new),
                })
            }
            UntypedStatement::Todo { location } => Ok(TypedStatement::Todo {
                location: *location,
            }),
            UntypedStatement::Panic { location } => Ok(TypedStatement::Panic {
                location: *location,
            }),
            UntypedStatement::Exit { location } => Ok(TypedStatement::Exit {
                location: *location,
            }),
        }
    }

    fn convert_expression_to_typed(
        &mut self,
        expr: &UntypedExpression,
    ) -> Result<TypedExpression, ConvertingError> {
        let start_expression_location = expr.get_location().start;
        let end_expression_location = expr.get_location().end;
        match expr {
            UntypedExpression::IntLiteral { location, value } => Ok(TypedExpression::IntLiteral {
                location: *location,
                value: value.parse::<i64>().map_err(|_| ConvertingError {
                    error: ConvertingErrorType::InvalidIntLiteral,
                    location: crate::lex::location::Location {
                        start: start_expression_location,
                        end: end_expression_location,
                    },
                })?,
                type_: Type::Int,
            }),
            UntypedExpression::FloatLiteral { location, value } => {
                Ok(TypedExpression::FloatLiteral {
                    location: *location,
                    value: value.parse::<f64>().map_err(|_| ConvertingError {
                        error: ConvertingErrorType::InvalidFloatLiteral,
                        location: crate::lex::location::Location {
                            start: start_expression_location,
                            end: end_expression_location,
                        },
                    })?,
                    type_: Type::Float,
                })
            }
            UntypedExpression::StringLiteral { location, value } => {
                Ok(TypedExpression::StringLiteral {
                    location: *location,
                    value: value.clone(),
                    type_: Type::String,
                })
            }
            UntypedExpression::CharLiteral { location, value } => {
                let char_value = value.chars().next().ok_or(ConvertingError {
                    error: ConvertingErrorType::InvalidCharLiteral,
                    location: crate::lex::location::Location {
                        start: start_expression_location,
                        end: end_expression_location,
                    },
                })?;
                Ok(TypedExpression::CharLiteral {
                    location: *location,
                    value: char_value,
                    type_: Type::Char,
                })
            }
            UntypedExpression::VariableValue { location, name } => {
                let resolved_type = self.resolve_variable_type(name)?;
                Ok(TypedExpression::VariableValue {
                    location: *location,
                    name: name.clone(),
                    type_: resolved_type,
                })
            }
            UntypedExpression::BinaryOperation {
                location,
                operator,
                left,
                right,
            } => {
                let typed_left = self.convert_expression_to_typed(left)?;
                let typed_right = self.convert_expression_to_typed(right)?;

                let result_type = TypeAnalyzer::check_type_of_binary_operation(
                    typed_left.get_type(),
                    typed_right.get_type(),
                    *operator,
                    start_expression_location,
                    end_expression_location,
                )?;
                Ok(TypedExpression::BinaryOperation {
                    location: *location,
                    operator: *operator,
                    left: Box::new(typed_left),
                    right: Box::new(typed_right),
                    type_: result_type,
                })
            }
            UntypedExpression::FunctionCall {
                location,
                function_name,
                arguments,
            } => {
                if let Some(function_def) = self.program_state.get_function(function_name) {
                    if let Some(expected_args) = function_def.get_arguments() {
                        if arguments.clone().unwrap().len() > expected_args.len() {
                            return Err(ConvertingError {
                                error: ConvertingErrorType::NotTheRightAmountOfArguments {
                                    expected: expected_args.len(),
                                    found: arguments.clone().unwrap().len(),
                                },
                                location: Location {
                                    start: location.start,
                                    end: location.end,
                                },
                            });
                        }
                    }
                }

                let typed_args = arguments
                    .as_ref()
                    .map(|args| {
                        args.clone()
                            .iter()
                            .enumerate()
                            .map(|(i, arg)| {
                                self.convert_call_argument_to_typed(
                                    function_name,
                                    arg,
                                    *location,
                                    i,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?;

                let function_type =
                    self.resolve_function_return_type(function_name, location.start, location.end)?;

                Ok(TypedExpression::FunctionCall {
                    location: *location,
                    function_name: function_name.clone(),
                    arguments: typed_args,
                    type_: function_type,
                })
            }
            UntypedExpression::StructFieldAccess {
                location,
                struct_name,
                field_name,
            } => {
                let field_type = self.resolve_struct_field_type(struct_name, field_name)?;
                Ok(TypedExpression::StructFieldAccess {
                    location: *location,
                    struct_name: struct_name.clone(),
                    field_name: field_name.clone(),
                    type_: field_type,
                })
            }
            UntypedExpression::ArrayElementAccess {
                location,
                array_name,
                index_expression,
            } => {
                let typed_index = self.convert_expression_to_typed(index_expression)?;

                if *typed_index.get_type() != Type::Int {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: Type::Int,
                            found: typed_index.get_type().clone(),
                        },
                        location: crate::lex::location::Location {
                            start: location.start,
                            end: location.end,
                        },
                    });
                }

                let element_type =
                    self.resolve_array_element_type(array_name, location.start, location.end)?;
                Ok(TypedExpression::ArrayElementAccess {
                    location: *location,
                    array_name: array_name.clone(),
                    index_expression: Box::new(typed_index),
                    type_: element_type,
                })
            }
            UntypedExpression::ArrayInitialization {
                location,
                type_annotation,
                elements,
            } => {
                let resolved_type = self.convert_untyped_to_typed(
                    type_annotation,
                    start_expression_location,
                    end_expression_location,
                )?;

                let inner_type = match resolved_type {
                    Type::Array { ref type_ } => Ok(type_.clone()),
                    _ => Err(ConvertingError {
                        error: ConvertingErrorType::UnsupportedType,
                        location: crate::lex::location::Location {
                            start: start_expression_location,
                            end: end_expression_location,
                        },
                    }),
                }?;

                let typed_elements = elements
                    .as_ref()
                    .map(|expressions| {
                        expressions.clone().try_mapped(|expression| {
                            let typed_expr = self.convert_expression_to_typed(&expression)?;
                            if !Self::compare_types(&inner_type, typed_expr.get_type()) {
                                return Err(ConvertingError {
                                    error: ConvertingErrorType::TypeMismatch {
                                        expected: resolved_type.clone(),
                                        found: typed_expr.get_type().clone(),
                                    },
                                    location: crate::lex::location::Location {
                                        start: start_expression_location,
                                        end: end_expression_location,
                                    },
                                });
                            }
                            Ok(typed_expr)
                        })
                    })
                    .transpose()?;

                Ok(TypedExpression::ArrayInitialization {
                    location: *location,
                    elements: typed_elements,
                    type_: resolved_type,
                })
            }
            UntypedExpression::StructInitialization {
                location,
                type_annotation,
                fields,
            } => {
                let resolved_type = self.convert_untyped_to_typed(
                    type_annotation,
                    start_expression_location,
                    end_expression_location,
                )?;

                let Type::Custom { name: type_name } = &resolved_type else {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::UnsupportedType,
                        location: crate::lex::location::Location {
                            start: start_expression_location,
                            end: end_expression_location,
                        },
                    });
                };

                let typed_fields = fields
                    .as_ref()
                    .map(|fields| {
                        fields
                            .clone()
                            .try_mapped(|field| self.convert_struct_field_value(&field, type_name))
                    })
                    .transpose()?;

                Ok(TypedExpression::StructInitialization {
                    location: *location,
                    type_: resolved_type.clone(),
                    fields: typed_fields.clone(),
                })
            }
        }
    }

    fn resolve_array_element_type(
        &mut self,
        array_name: &EcoString,
        start_location: u32,
        end_location: u32,
    ) -> Result<Type, ConvertingError> {
        let array_type =
            self.program_state
                .get_variable_type(array_name)
                .ok_or(ConvertingError {
                    error: ConvertingErrorType::UnsupportedType,
                    location: crate::lex::location::Location {
                        start: start_location,
                        end: end_location,
                    },
                })?;
        match array_type {
            Type::Array { type_ } => Ok(*type_.clone()),
            _ => Err(ConvertingError {
                error: ConvertingErrorType::UnsupportedType,
                location: crate::lex::location::Location {
                    start: start_location,
                    end: end_location,
                },
            }),
        }
    }

    fn resolve_struct_field_type(
        &self,
        struct_name: &EcoString,
        field_name: &EcoString,
    ) -> Result<Type, ConvertingError> {
        let struct_def = self
            .program_state
            .get_struct(struct_name)
            .ok_or(ConvertingError {
                error: ConvertingErrorType::StructNotFound,
                location: crate::lex::location::Location { start: 0, end: 0 },
            })?;

        if let DefinitionTyped::Struct { fields, .. } = struct_def {
            let fields = fields.as_ref().ok_or(ConvertingError {
                error: ConvertingErrorType::EmptyStruct,
                location: crate::lex::location::Location { start: 0, end: 0 },
            })?;

            let field = fields
                .iter()
                .find(|field| field.name == *field_name)
                .ok_or(ConvertingError {
                    error: ConvertingErrorType::FieldNotFound,
                    location: crate::lex::location::Location { start: 0, end: 0 },
                })?;

            Ok(field.type_.clone())
        } else {
            Err(ConvertingError {
                error: ConvertingErrorType::StructNotFound,
                location: crate::lex::location::Location { start: 0, end: 0 },
            })
        }
    }

    fn resolve_function_return_type(
        &mut self,
        function_name: &EcoString,
        start_location: u32,
        end_location: u32,
    ) -> Result<Type, ConvertingError> {
        let function_def =
            self.program_state
                .get_function(function_name)
                .ok_or(ConvertingError {
                    error: ConvertingErrorType::UndefinedFunction,
                    location: crate::lex::location::Location {
                        start: start_location,
                        end: end_location,
                    },
                })?;
        function_def.get_return_type()
    }

    fn resolve_variable_type(
        &mut self,
        variable_name: &EcoString,
    ) -> Result<Type, ConvertingError> {
        let variable_type =
            self.program_state
                .get_variable_type(variable_name)
                .ok_or(ConvertingError {
                    error: ConvertingErrorType::UnsupportedType,
                    location: crate::lex::location::Location { start: 0, end: 0 },
                })?;
        Ok(variable_type.clone())
    }

    fn check_type_of_binary_operation(
        left_type: &Type,
        right_type: &Type,
        operator: BinaryOperator,
        start_location: u32,
        end_location: u32,
    ) -> Result<Type, ConvertingError> {
        let err_location = crate::lex::location::Location {
            start: start_location,
            end: end_location,
        };

        match operator {
            BinaryOperator::And | BinaryOperator::Or => match (left_type, right_type) {
                (Type::Boolean, Type::Boolean) => Ok(Type::Boolean),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::InvalidBooleanOperation,
                    location: err_location,
                }),
            },

            BinaryOperator::LessInt
            | BinaryOperator::LessEqualInt
            | BinaryOperator::GreaterInt
            | BinaryOperator::GreaterEqualInt => match (left_type, right_type) {
                (Type::Int, Type::Int) => Ok(Type::Boolean),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::IntOperationInvalidType,
                    location: err_location,
                }),
            },

            BinaryOperator::LessFloat
            | BinaryOperator::LessEqualFloat
            | BinaryOperator::GreaterFloat
            | BinaryOperator::GreaterEqualFloat => match (left_type, right_type) {
                (Type::Float, Type::Float) => Ok(Type::Boolean),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::FloatOperationInvalidType,
                    location: err_location,
                }),
            },

            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if left_type == right_type {
                    Ok(Type::Boolean)
                } else {
                    Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: left_type.clone(),
                            found: right_type.clone(),
                        },
                        location: err_location,
                    })
                }
            }

            BinaryOperator::AdditionInt
            | BinaryOperator::SubtractionInt
            | BinaryOperator::MultipicationInt
            | BinaryOperator::DivisionInt
            | BinaryOperator::Modulo => match (left_type, right_type) {
                (Type::Int, Type::Int) => Ok(Type::Int),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::IntOperationInvalidType,
                    location: err_location,
                }),
            },

            BinaryOperator::AdditionFloat
            | BinaryOperator::SubtractionFloat
            | BinaryOperator::MultipicationFloat
            | BinaryOperator::DivisionFloat => match (left_type, right_type) {
                (Type::Float, Type::Float) => Ok(Type::Float),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::FloatOperationInvalidType,
                    location: err_location,
                }),
            },

            BinaryOperator::Concatenation => match (left_type, right_type) {
                (Type::String, Type::String) => Ok(Type::String),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::StringOperationInvalidType,
                    location: err_location,
                }),
            },
        }
    }

    fn convert_call_argument_to_typed(
        &mut self,
        function_name: &EcoString,
        argument: &CallArgumentUntyped,
        location: ast::location::Location,
        i: usize,
    ) -> Result<CallArgumentTyped, ConvertingError> {
        let typed_argument = self.convert_expression_to_typed(&argument.value)?;
        let function_def =
            self.program_state
                .functions
                .get(function_name)
                .ok_or(ConvertingError {
                    error: ConvertingErrorType::UndefinedFunction,
                    location: Location {
                        start: location.start,
                        end: location.end,
                    },
                })?;

        if let Some(expected_args) = function_def.get_arguments() {
            if expected_args.len() <= i {
                return Err(ConvertingError {
                    error: ConvertingErrorType::NotTheRightAmountOfArguments {
                        expected: expected_args.len(),
                        found: i + 1,
                    },
                    location: Location {
                        start: location.start,
                        end: location.end,
                    },
                });
            }

            if expected_args[i].type_ != *typed_argument.clone().get_type() {
                return Err(ConvertingError {
                    error: ConvertingErrorType::TypeMismatch {
                        expected: expected_args[i].type_.clone(),
                        found: typed_argument.get_type().clone(),
                    },
                    location: Location {
                        start: location.start,
                        end: location.end,
                    },
                });
            }
        }

        Ok(CallArgumentTyped {
            value: typed_argument.clone(),
            location,
            type_: typed_argument.get_type().clone(),
        })
    }

    fn convert_untyped_to_typed(
        &mut self,
        untyped_type: &UntypedType,
        start_location: u32,
        end_location: u32,
    ) -> Result<Type, ConvertingError> {
        match untyped_type {
            UntypedType::Int => Ok(Type::Int),
            UntypedType::Float => Ok(Type::Float),
            UntypedType::String => Ok(Type::String),
            UntypedType::Char => Ok(Type::Char),
            UntypedType::Custom { name } => {
                if self.program_state.get_struct(name).is_none() {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::StructNotFound,
                        location: Location {
                            start: start_location,
                            end: end_location,
                        },
                    });
                }
                Ok(Type::Custom { name: name.clone() })
            }
            UntypedType::Array { type_ } => {
                let element_type =
                    self.convert_untyped_to_typed(type_, start_location, end_location)?;
                Ok(Type::Array {
                    type_: Box::new(element_type),
                })
            }
            UntypedType::Boolean => Ok(Type::Boolean),
        }
    }

    fn compare_types(expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Custom { name: name1 }, Type::Custom { name: name2 }) => name1 == name2,
            (Type::Array { type_: type1 }, Type::Array { type_: type2 }) => {
                Self::compare_types(type1, type2)
            }
            (expected_type, found_type) => expected_type == found_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramState {
    variables: HashMap<EcoString, Type>,
    functions: HashMap<EcoString, DefinitionTyped>,
    structs: HashMap<EcoString, DefinitionTyped>,
    current_function_name: EcoString,
}

impl Default for ProgramState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgramState {
    #[must_use]
    pub fn new() -> Self {
        ProgramState {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            current_function_name: "".into(),
        }
    }

    fn add_variable(&mut self, name: EcoString, type_: Type) {
        self.variables.insert(name, type_);
    }

    fn get_variable_type(&self, name: &EcoString) -> Option<&Type> {
        self.variables.get(name)
    }

    fn add_function(&mut self, name: EcoString, definition: DefinitionTyped) {
        if let DefinitionTyped::Function { .. } = definition {
            self.functions.insert(name, definition);
        }
    }

    fn get_function(&self, name: &EcoString) -> Option<&DefinitionTyped> {
        self.functions.get(name)
    }

    fn add_struct(&mut self, name: EcoString, definition: DefinitionTyped) {
        if let DefinitionTyped::Struct { .. } = definition {
            self.structs.insert(name, definition);
        }
    }

    fn get_struct(&self, name: &EcoString) -> Option<&DefinitionTyped> {
        self.structs.get(name)
    }

    fn clear_variables(&mut self) {
        self.variables.clear();
    }

    fn create_scope(&self) -> HashMap<EcoString, Type> {
        self.variables.clone()
    }

    fn restore_scope(&mut self, saved_variables: HashMap<EcoString, Type>) {
        self.variables = saved_variables;
    }

    fn set_current_function_name(&mut self, name: &EcoString) {
        self.current_function_name = name.clone();
    }

    fn get_current_function_name(&mut self) -> EcoString {
        self.current_function_name.clone()
    }
}
