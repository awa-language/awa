use std::collections::HashMap;

use super::argument::CallArgument;
use super::expression::CallArgumentTyped;
use super::statement;
use crate::ast::argument::{ArgumentTyped, ArgumentUntyped};
use crate::ast::assignment::{Assignment, TypedAssignment};
use crate::ast::definition::{DefinitionTyped, DefinitionUntyped, StructField, StructFieldTyped};
use crate::ast::expression::{
    StructFieldValue, StructFieldValueTyped, TypedExpression, UntypedExpression,
};
use crate::ast::module::{Module, Typed};
use crate::ast::operator::BinaryOperator;
use crate::ast::reassignment::{Reassignment, ReassignmentTarget};
use crate::ast::statement::{TypedStatement, UntypedStatement};
use crate::ast::{argument, definition, module};
use crate::parse::error::{ConvertingError, ConvertingErrorType};
use crate::type_::{Type, UntypedType};
use ecow::EcoString;
use vec1::Vec1;

#[derive(Debug, Clone)]
pub struct ProgramState {
    variables: HashMap<EcoString, Type>,
    functions: HashMap<EcoString, DefinitionTyped>,
    structs: HashMap<EcoString, DefinitionTyped>,
}

impl ProgramState {
    pub fn new() -> Self {
        ProgramState {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    fn add_variable(&mut self, name: EcoString, type_: Type) {
        self.variables.insert(name, type_);
    }

    fn get_variable_type(&self, name: &EcoString) -> Option<&Type> {
        self.variables.get(name)
    }

    fn add_function(&mut self, name: EcoString, definition: DefinitionTyped) {
        match definition {
            DefinitionTyped::Function { .. } => {
                self.functions.insert(name, definition);
            }
            _ => {}
        }
    }

    fn get_function(&self, name: &EcoString) -> Option<&DefinitionTyped> {
        self.functions.get(name)
    }

    fn add_struct(&mut self, name: EcoString, definition: DefinitionTyped) {
        match definition {
            DefinitionTyped::Struct { .. } => {
                self.structs.insert(name, definition);
            }
            _ => {}
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

    fn convert_ast_to_tast(
        &mut self,
        untyped_ast: &Module<DefinitionUntyped>,
    ) -> Result<Module<DefinitionTyped>, ConvertingError> {
        let mut typed_definitions = None;
        let mut program_state = ProgramState::new();

        if let Some(definitions) = &untyped_ast.definitions {
            for definition in definitions {
                let typed_definition = match definition {
                    DefinitionUntyped::Function {
                        location,
                        name,
                        arguments,
                        body,
                        return_type_annotation,
                    } => {
                        let typed_args = arguments
                            .as_ref()
                            .map(|args| args.clone().try_mapped(|arg| self.convert_arguments(&arg)))
                            .transpose()?;

                        let typed_body = body
                            .as_ref()
                            .map(|statements| {
                                statements.clone().try_mapped(|statement| {
                                    self.convert_statement_to_typed(&statement)
                                })
                            })
                            .transpose()?;

                        let return_type = return_type_annotation
                            .as_ref()
                            .map(|t| self.convert_untyped_to_typed(t, location.start, location.end))
                            .transpose()?;

                        let typed_function = DefinitionTyped::Function {
                            name: name.clone(),
                            location: location.clone(),
                            arguments: typed_args,
                            body: typed_body,
                            return_type_annotation: return_type,
                        };

                        program_state.add_function(name.clone(), typed_function.clone());

                        typed_function
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
                            location: location.clone(),
                            name: name.clone(),
                            fields: typed_fields,
                        };

                        program_state.add_struct(name.clone(), typed_struct.clone());

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

    fn convert_arguments(
        &mut self,
        p0: &ArgumentUntyped,
    ) -> Result<ArgumentTyped, ConvertingError> {
        todo!()
    }

    fn convert_struct_field(
        &mut self,
        field: &StructField,
    ) -> Result<StructFieldTyped, ConvertingError> {
        let resolved_type = self.convert_untyped_to_typed(&field.type_annotation, 0, 0)?;
        Ok(StructFieldTyped {
            name: field.name.clone(),
            type_annotation: resolved_type,
        })
    }

    fn convert_struct_field_value(
        &mut self,
        struct_field_value: &StructFieldValue,
        struct_name: &EcoString,
    ) -> Result<StructFieldValueTyped, ConvertingError> {
        let typed_value = self.convert_expression_to_typed(&struct_field_value.value)?;
        let location = typed_value.get_location();

        let struct_def = self.get_struct(struct_name).ok_or(ConvertingError {
            error: ConvertingErrorType::StructNotFound,
            location: crate::lex::location::Location {
                start: location.start,
                end: location.end,
            },
        })?;

        match struct_def {
            DefinitionTyped::Struct { fields, .. } => {
                let fields = fields.as_ref().ok_or(ConvertingError {
                    error: ConvertingErrorType::EmptyStruct,
                    location: crate::lex::location::Location {
                        start: location.start,
                        end: location.end,
                    },
                })?;

                let field = fields
                    .iter()
                    .find(|f| f.name == struct_field_value.name)
                    .ok_or(ConvertingError {
                        error: ConvertingErrorType::FieldNotFound,
                        location: crate::lex::location::Location {
                            start: location.start,
                            end: location.end,
                        },
                    })?;

                if Self::compare_types(&field.type_annotation, &typed_value.get_type()) {
                    return Err(ConvertingError {
                        error: ConvertingErrorType::TypeMismatch {
                            expected: field.type_annotation.clone(),
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
                    type_: field.type_annotation.clone(),
                })
            }
            _ => Err(ConvertingError {
                error: ConvertingErrorType::StructNotFound,
                location: crate::lex::location::Location {
                    start: location.start,
                    end: location.end,
                },
            }),
        }
    }

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
                Ok(TypedStatement::Assignment(TypedAssignment {
                    location: assignment.location,
                    variable_name: assignment.variable_name.clone(),
                    value: Box::new(typed_value),
                    type_annotation: assignment.type_annotation.clone(),
                }))
            }
            UntypedStatement::Reassignment(reassignment) => {
                let typed_new_value = self.convert_expression_to_typed(&reassignment.new_value)?;
                let typed_target = match &reassignment.target {
                    ReassignmentTarget::Variable { location, name } => {
                        ReassignmentTarget::Variable {
                            location: location.clone(),
                            name: name.clone(),
                        }
                    }
                    ReassignmentTarget::FieldAccess {
                        location,
                        struct_name,
                        field_name,
                    } => ReassignmentTarget::FieldAccess {
                        location: location.clone(),
                        struct_name: struct_name.clone(),
                        field_name: field_name.clone(),
                    },
                    ReassignmentTarget::ArrayAccess {
                        location,
                        array_name,
                        index_expression,
                    } => ReassignmentTarget::ArrayAccess {
                        location: location.clone(),
                        array_name: array_name.clone(),
                        index_expression: Box::new(
                            self.convert_expression_to_typed(index_expression)?,
                        ),
                    },
                };

                Ok(TypedStatement::Reassignment(Reassignment {
                    location: reassignment.location.clone(),
                    target: typed_target,
                    new_value: Box::new(typed_new_value),
                }))
            }
            UntypedStatement::Loop { body, location } => {
                let typed_body = body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;

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

                let typed_if_body = if_body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;

                let typed_else_body = else_body
                    .as_ref()
                    .map(|statements| {
                        statements
                            .clone()
                            .try_mapped(|statement| self.convert_statement_to_typed(&statement))
                    })
                    .transpose()?;

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
                    .map(|v| self.convert_expression_to_typed(v))
                    .transpose()?;
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
                location: location.clone(),
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
                    location: location.clone(),
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
                    location: location.clone(),
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
                    location: location.clone(),
                    value: char_value,
                    type_: Type::Char,
                })
            }
            UntypedExpression::VariableValue { location, name } => {
                let resolved_type = self.resolve_variable_type(name)?;
                Ok(TypedExpression::VariableValue {
                    location: location.clone(),
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
                if typed_left != typed_right {}
                let result_type = self.check_type_of_binary_operation(
                    &typed_left.get_type(),
                    &typed_right.get_type(),
                    operator,
                    start_expression_location,
                    end_expression_location,
                )?;
                Ok(TypedExpression::BinaryOperation {
                    location: location.clone(),
                    operator: operator.clone(),
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
                let typed_args = arguments
                    .as_ref()
                    .map(|args| {
                        args.clone()
                            .try_mapped(|arg| self.convert_call_argument_to_typed(&arg))
                    })
                    .transpose()?;

                let function_type = self.resolve_function_type(function_name)?;
                Ok(TypedExpression::FunctionCall {
                    location: location.clone(),
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
                    location: location.clone(),
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
                let element_type = self.resolve_array_element_type(array_name)?;
                Ok(TypedExpression::ArrayElementAccess {
                    location: location.clone(),
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
                let typed_elements = elements
                    .as_ref()
                    .map(|expressions| {
                        expressions
                            .clone()
                            .try_mapped(|expression| self.convert_expression_to_typed(&expression))
                    })
                    .transpose()?;

                let resolved_type = self.convert_untyped_to_typed(
                    type_annotation,
                    start_expression_location,
                    end_expression_location,
                )?;
                Ok(TypedExpression::ArrayInitialization {
                    location: location.clone(),
                    type_annotation: resolved_type.clone(),
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

                let type_name = if let Type::Custom { name } = &resolved_type {
                    name
                } else {
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
                    location: location.clone(),
                    type_: resolved_type.clone(),
                    fields: typed_fields.clone(),
                })
            }
        }
    }

    fn resolve_array_element_type(&mut self, str: &EcoString) -> Result<Type, ConvertingError> {
        todo!()
    }

    fn resolve_struct_field_type(
        &mut self,
        str: &EcoString,
        str1: &EcoString,
    ) -> Result<Type, ConvertingError> {
        todo!()
    }

    fn resolve_function_type(
        &mut self,
        function_name: &EcoString,
    ) -> Result<Type, ConvertingError> {
        todo!()
    }

    fn resolve_variable_type(
        &mut self,
        variable_name: &EcoString,
    ) -> Result<Type, ConvertingError> {
        todo!()
    }

    fn check_type_of_binary_operation(
        &mut self,
        left_type: &Type,
        right_type: &Type,
        operator: &BinaryOperator,
        start_location: u32,
        end_location: u32,
    ) -> Result<Type, ConvertingError> {
        match operator {
            /* logical operations in case we will add them
            BinaryOperator::And | BinaryOperator::Or => {
                match (left_type, right_type) {
                    (Type::Bool, Type::Bool) => Ok(Type::Bool),
                    _ => Err("Logical operations require boolean expressions in both sides".into()),
                }
            }
            */
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessInt
            | BinaryOperator::LessEqualInt
            | BinaryOperator::GreaterInt
            | BinaryOperator::GreaterEqualInt
            | BinaryOperator::AdditionInt
            | BinaryOperator::SubtractionInt
            | BinaryOperator::MultipicationInt
            | BinaryOperator::DivisionInt
            | BinaryOperator::Modulo => match (left_type, right_type) {
                (Type::Int, Type::Int) => Ok(Type::Int),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::IntOperationInvalidType,
                    location: crate::lex::location::Location {
                        start: start_location,
                        end: end_location,
                    },
                }),
            },

            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessFloat
            | BinaryOperator::LessEqualFloat
            | BinaryOperator::GreaterFloat
            | BinaryOperator::GreaterEqualFloat
            | BinaryOperator::AdditionFloat
            | BinaryOperator::SubtractionFloat
            | BinaryOperator::MultipicationFloat
            | BinaryOperator::DivisionFloat => match (left_type, right_type) {
                (Type::Float, Type::Float) => Ok(Type::Float),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::FloatOperationInvalidType,
                    location: crate::lex::location::Location {
                        start: start_location,
                        end: end_location,
                    },
                }),
            },

            BinaryOperator::Concatenation => match (left_type, right_type) {
                (Type::String, Type::String) => Ok(Type::String),
                _ => Err(ConvertingError {
                    error: ConvertingErrorType::StringOperationInvalidType,
                    location: crate::lex::location::Location {
                        start: start_location,
                        end: end_location,
                    },
                }),
            },

            _ => Err(ConvertingError {
                error: ConvertingErrorType::UnsupportedBinaryOperation,
                location: crate::lex::location::Location {
                    start: start_location,
                    end: end_location,
                },
            }),
        }
    }

    fn convert_call_argument_to_typed(
        &mut self,
        arg: &argument::CallArgument<UntypedExpression>,
    ) -> Result<CallArgumentTyped<TypedExpression>, ConvertingError> {
        todo!()
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
            UntypedType::Custom { name } => Ok(Type::Custom { name: name.clone() }),
            UntypedType::Array { type_ } => {
                let element_type =
                    self.convert_untyped_to_typed(type_, start_location, end_location)?;
                Ok(Type::Array {
                    type_: Box::new(element_type),
                })
            }
            UntypedType::Boolean => Ok(Type::Boolean),
            _ => Err(ConvertingError {
                error: ConvertingErrorType::UnsupportedBinaryOperation,
                location: crate::lex::location::Location {
                    start: start_location,
                    end: end_location,
                },
            }),
        }
    }

    fn compare_types(expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            (Type::Custom { name: n1 }, Type::Custom { name: n2 }) => n1 == n2,
            (Type::Array { type_: t1 }, Type::Array { type_: t2 }) => Self::compare_types(t1, t2),
            (a, b) => a == b,
        }
    }
}
