use crate::ast::location::Location;
use ecow::EcoString;

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
        arguments: Vec<CallArgument<Self>>,
    },
    StructFieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
    },
    ArrayIndexAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<Self>,
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
            | Expression::ArrayIndexAccess { location, .. } => *location,
        }
    }
}
