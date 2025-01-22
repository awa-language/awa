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
use crate::type_::{Type, UntypedType};
use ecow::EcoString;
use vec1::Vec1;

use super::argument::CallArgument;
use super::expression::CallArgumentTyped;
use super::statement;

fn convert_ast_to_tast(
    untyped_ast: &Module<DefinitionUntyped>,
) -> Result<Module<DefinitionTyped>, String> {
    let mut typed_definitions = None;

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
                        .map(|args| args.clone().try_mapped(|arg| convert_arguments(&arg)))
                        .transpose()?;

                    let typed_body = body
                        .as_ref()
                        .map(|statements| {
                            statements
                                .clone()
                                .try_mapped(|statement| convert_statement_to_typed(&statement))
                        })
                        .transpose()?;

                    let return_type = return_type_annotation
                        .as_ref()
                        .map(|t| convert_untyped_to_typed(t))
                        .transpose()?;

                    DefinitionTyped::Function {
                        name: name.clone(),
                        location: location.clone(),
                        arguments: typed_args,
                        body: typed_body,
                        return_type_annotation: return_type,
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
                                .try_mapped(|field| convert_struct_field(&field))
                        })
                        .transpose()?;

                    DefinitionTyped::Struct {
                        location: location.clone(),
                        name: name.clone(),
                        fields: typed_fields,
                    }
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

fn convert_arguments(p0: &ArgumentUntyped) -> Result<ArgumentTyped, String> {
    todo!()
}

pub fn convert_struct_field(field: &StructField) -> Result<StructFieldTyped, String> {
    let resolved_type = convert_untyped_to_typed(&field.type_annotation)?;
    Ok(StructFieldTyped {
        name: field.name.clone(),
        type_annotation: resolved_type,
    })
}

pub fn convert_statement_to_typed(stmt: &UntypedStatement) -> Result<TypedStatement, String> {
    match stmt {
        UntypedStatement::Expression(expression) => {
            let typed_expression = convert_expression_to_typed(expression)?;
            Ok(TypedStatement::Expression(typed_expression))
        }
        UntypedStatement::Assignment(assignment) => {
            let typed_value = convert_expression_to_typed(&assignment.value)?;
            Ok(TypedStatement::Assignment(TypedAssignment {
                location: assignment.location,
                variable_name: assignment.variable_name.clone(),
                value: Box::new(typed_value),
                type_annotation: assignment.type_annotation.clone(),
            }))
        }
        UntypedStatement::Reassignment(reassignment) => {
            let typed_new_value = convert_expression_to_typed(&reassignment.new_value)?;
            let typed_target = match &reassignment.target {
                ReassignmentTarget::Variable { location, name } => ReassignmentTarget::Variable {
                    location: location.clone(),
                    name: name.clone(),
                },
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
                    index_expression: Box::new(convert_expression_to_typed(index_expression)?),
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
                        .try_mapped(|statement| convert_statement_to_typed(&statement))
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
            let typed_condition = convert_expression_to_typed(condition)?;

            let typed_if_body = if_body
                .as_ref()
                .map(|statements| {
                    statements
                        .clone()
                        .try_mapped(|statement| convert_statement_to_typed(&statement))
                })
                .transpose()?;

            let typed_else_body = else_body
                .as_ref()
                .map(|statements| {
                    statements
                        .clone()
                        .try_mapped(|statement| convert_statement_to_typed(&statement))
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
                .map(|v| convert_expression_to_typed(v))
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

fn convert_expression_to_typed(expr: &UntypedExpression) -> Result<TypedExpression, String> {
    match expr {
        UntypedExpression::IntLiteral { location, value } => Ok(TypedExpression::IntLiteral {
            location: location.clone(),
            value: value
                .parse::<i64>()
                .map_err(|_| "Invalid integer literal")?,
            type_: Type::Int,
        }),
        UntypedExpression::FloatLiteral { location, value } => Ok(TypedExpression::FloatLiteral {
            location: location.clone(),
            value: value.parse::<f64>().map_err(|_| "Invalid float literal")?,
            type_: Type::Float,
        }),
        UntypedExpression::StringLiteral { location, value } => {
            Ok(TypedExpression::StringLiteral {
                location: location.clone(),
                value: value.clone(),
                type_: Type::String,
            })
        }
        UntypedExpression::CharLiteral { location, value } => {
            let char_value = value.chars().next().ok_or("Invalid char literal")?;
            Ok(TypedExpression::CharLiteral {
                location: location.clone(),
                value: char_value,
                type_: Type::Char,
            })
        }
        UntypedExpression::VariableValue { location, name } => {
            let resolved_type = resolve_variable_type(name)?;
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
            let typed_left = convert_expression_to_typed(left)?;
            let typed_right = convert_expression_to_typed(right)?;
            if typed_left != typed_right {}
            let result_type = check_type_of_binary_operation(
                &typed_left.get_type(),
                &typed_right.get_type(),
                operator,
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
                        .try_mapped(|arg| convert_call_argument_to_typed(&arg))
                })
                .transpose()?;

            let function_type = resolve_function_type(function_name)?;
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
            let field_type = resolve_struct_field_type(struct_name, field_name)?;
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
            let typed_index = convert_expression_to_typed(index_expression)?;
            let element_type = resolve_array_element_type(array_name)?;
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
                        .try_mapped(|expression| convert_expression_to_typed(&expression))
                })
                .transpose()?;

            let resolved_type = convert_untyped_to_typed(type_annotation)?;
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
            let typed_fields = fields
                .as_ref()
                .map(|fields| {
                    fields
                        .clone()
                        .try_mapped(|field| convert_struct_field_value(&field))
                })
                .transpose()?;

            let resolved_type = convert_untyped_to_typed(type_annotation)?;
            Ok(TypedExpression::StructInitialization {
                location: location.clone(),
                type_annotation: resolved_type.clone(),
                fields: typed_fields.clone(),
                type_: vec1::vec1![resolved_type],
            })
        }
    }
}

fn convert_struct_field_value(
    struct_field_value: &StructFieldValue,
) -> Result<StructFieldValueTyped, String> {
    todo!()
}

fn resolve_array_element_type(str: &EcoString) -> Result<Type, String> {
    todo!()
}

fn resolve_struct_field_type(str: &EcoString, str1: &EcoString) -> Result<Type, String> {
    todo!()
}

fn resolve_function_type(function_name: &EcoString) -> Result<Type, String> {
    todo!()
}

fn resolve_variable_type(variable_name: &EcoString) -> Result<Type, String> {
    todo!()
}

fn check_type_of_binary_operation(
    left_type: &Type,
    right_type: &Type,
    operator: &BinaryOperator,
) -> Result<Type, String> {
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
            _ => Err("Integer operations require integer expressions in both sides".into()),
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
            _ => Err("Float operations require float expressions in both sides".into()),
        },

        BinaryOperator::Concatenation => match (left_type, right_type) {
            (Type::String, Type::String) => Ok(Type::String),
            _ => Err("String operations requires string expressions in both sides".into()),
        },

        _ => Err("Unsupported binary operation".into()),
    }
}

fn convert_call_argument_to_typed(
    arg: &argument::CallArgument<UntypedExpression>,
) -> Result<CallArgumentTyped<TypedExpression>, String> {
    todo!()
}

fn convert_untyped_to_typed(untyped_type: &UntypedType) -> Result<Type, String> {
    match untyped_type {
        UntypedType::Int => Ok(Type::Int),
        UntypedType::Float => Ok(Type::Float),
        UntypedType::String => Ok(Type::String),
        UntypedType::Char => Ok(Type::Char),
        UntypedType::Custom { name } => Ok(Type::Custom { name: name.clone() }),
        UntypedType::Array { type_ } => {
            let element_type = convert_untyped_to_typed(type_)?;
            Ok(Type::Array {
                type_: Box::new(element_type),
            })
        }
        UntypedType::Boolean => Ok(Type::Boolean),
        _ => Err("Unsupported type".into()),
    }
}
