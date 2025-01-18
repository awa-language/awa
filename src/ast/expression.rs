use crate::{ast::location::Location, type_::Type};
use ecow::EcoString;
use vec1::Vec1;

use super::argument::CallArgument;

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
