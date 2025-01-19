use super::argument::CallArgument;
use crate::ast::expression_typed::ExpressionTyped;
use crate::{ast::location::Location, type_::Type};
use ecow::EcoString;
use vec1::Vec1;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    IntLiteral {
        location: Location,
        value: EcoString,
    },
    FloatLiteral {
        location: Location,
        value: EcoString,
    },
    StringLiteral {
        location: Location,
        value: EcoString,
    },
    CharLiteral {
        location: Location,
        value: EcoString,
    },
    VariableValue {
        location: Location,
        name: EcoString,
    },
    FunctionCall {
        location: Location,
        function_name: EcoString,
        arguments: Option<Vec1<CallArgument<Self>>>,
    },
    StructFieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
    },
    ArrayElementAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<Self>,
    },
    ArrayInitialization {
        location: Location,
        type_annotation: Type,
        elements: Option<Vec1<Self>>,
    },
    StructInitialization {
        location: Location,
        type_annotation: Type,
        fields: Option<Vec1<StructFieldValue>>,
    },
}

impl Expression {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Expression::IntLiteral { location, .. }
            | Expression::FloatLiteral { location, .. }
            | Expression::CharLiteral { location, .. }
            | Expression::StringLiteral { location, .. }
            | Expression::VariableValue { location, .. }
            | Expression::FunctionCall { location, .. }
            | Expression::StructFieldAccess { location, .. }
            | Expression::ArrayElementAccess { location, .. }
            | Expression::ArrayInitialization { location, .. }
            | Expression::StructInitialization { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldValue {
    pub name: EcoString,
    pub value: Expression,
}

impl Expression {
    pub fn translate_to_typed_expression(&self) -> ExpressionTyped {
        match self {
            Expression::IntLiteral { location, value } => ExpressionTyped::IntLiteral {
                location: *location,
                type_: Type::Int,
                value: value.clone(),
            },
            Expression::FloatLiteral { location, value } => ExpressionTyped::FloatLiteral {
                location: *location,
                type_: Type::Float,
                value: value.clone(),
            },
            Expression::StringLiteral { location, value } => ExpressionTyped::StringLiteral {
                location: *location,
                type_: Type::String,
                value: value.clone(),
            },
            Expression::CharLiteral { location, value } => ExpressionTyped::CharLiteral {
                location: *location,
                type_: Type::Char,
                value: value.clone(),
            },
            Expression::VariableValue { location, name } => ExpressionTyped::VariableValue {
                location: *location,
                name: name.clone(),
            },
            Expression::FunctionCall {
                location,
                function_name,
                arguments,
            } => ExpressionTyped::FunctionCall {
                location: *location,
                function_name: function_name.clone(),
                type_: Type::FunctionCall,
                arguments: todo!(),
            },
            Expression::StructFieldAccess {
                location,
                struct_name,
                field_name,
            } => ExpressionTyped::StructFieldAccess {
                location: *location,
                type_: Type::StructFieldAccess,
                struct_name: struct_name.clone(),
                field_name: field_name.clone(),
            },
            Expression::ArrayElementAccess {
                location,
                array_name,
                index_expression,
            } => ExpressionTyped::ArrayElementAccess {
                location: *location,
                type_: Type::ArrayElementAccess,
                array_name: array_name.clone(),
                index_expression: todo!(),
            },
            Expression::ArrayInitialization {
                location,
                type_annotation,
                elements,
            } => ExpressionTyped::ArrayInitialization {
                location: *location,
                type_: Type::ArrayInitialization,
                elements: todo!(),
                type_annotation: type_annotation.clone(),
            },
            Expression::StructInitialization {
                location,
                type_annotation,
                fields,
            } => ExpressionTyped::StructInitialization {
                location: *location,
                type_: Type::StructInitialization,
                fields: todo!(),
                type_annotation: type_annotation.clone(),
            },
        }
    }
}
