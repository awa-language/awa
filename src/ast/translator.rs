use crate::ast::argument::{ArgumentTyped};
use crate::ast::definition::Untyped;
use crate::ast::expression::{
    StructFieldValue, StructFieldValueTyped, TypedExpression, UntypedExpression,
};
use crate::ast::module::Module;
use crate::ast::operator::BinaryOperator;
use crate::ast::{argument, definition};
use crate::type_::{Type, UntypedType};
use ecow::EcoString;
use vec1::Vec1;

#[must_use]
pub fn translate_to_tast(
    module: &Module<definition::Untyped>,
) -> Result<Module<definition::Typed>, String> {
    let mut result = Vec::new();

    for (i, definition) in module.definitions.iter().enumerate() {
        let has_next = i < module.definitions.len() - 1;

        let typed_module = convert_definition_tdefinition(definition, &[has_next]);
        result.push((typed_module));
    }

    return todo!();
}

fn convert_definition_tdefinition(p0: &Untyped, p1: &[bool; 1]) -> definition::Typed {
    todo!()
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
            let typed_args = match arguments {
                Some(args) => Some(
                    args.iter()
                        .map(|arg| convert_call_argument_to_typed(arg))
                        .collect::<Result<Vec1<_>, _>>()?,
                ),
                None => None,
            };
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
            let typed_elements = match elements {
                Some(el) => Some(
                    el.iter()
                        .map(|e| convert_expression_to_typed(e))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                None => None,
            };
            let resolved_type = convert_untyped_to_typed(type_annotation)?;
            Ok(TypedExpression::ArrayInitialization {
                location: location.clone(),
                type_annotation: resolved_type.clone(),
                elements: typed_elements.clone()[0],
                type_: resolved_type,
            })
        }
        UntypedExpression::StructInitialization {
            location,
            type_annotation,
            fields,
        } => {
            let typed_fields = match fields {
                Some(f) => Some(
                    f.iter()
                        .map(|field| convert_struct_field_value(field))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                None => None,
            };
            let resolved_type = convert_untyped_to_typed(type_annotation)?;
            Ok(TypedExpression::StructInitialization {
                location: location.clone(),
                type_annotation: resolved_type.clone(),
                fields: typed_fields.clone()[0],
                type_: vec1::vec1![resolved_type],
            })
        }
    }
}

fn convert_struct_field_value(
    structFieldValue: &StructFieldValue,
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
) -> ArgumentTyped {
    todo!()
}

fn convert_untyped_to_typed(untyped_type: &UntypedType) -> Result<Type, String> {
    match untyped_type {
        UntypedType::Int => Ok(Type::Int),
        UntypedType::Float => Ok(Type::Float),
        UntypedType::String => Ok(Type::String),
        UntypedType::Char => Ok(Type::Char),
        UntypedType::Custom => Ok(Type::Custom), // TODO
        UntypedType::Array => Ok(Type::Array),   // TODO
        UntypedType::Boolean => Ok(Type::Boolean),
        _ => Err("Unsupported type".into()),
    }
}
