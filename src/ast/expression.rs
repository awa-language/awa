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
    Todo {
        location: Location,
    },
    Panic {
        location: Location,
    },
    Exit {
        location: Location,
    },
    // TODO: add struct field access
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
            | Expression::Todo { location, .. }
            | Expression::Panic { location, .. }
            | Expression::Exit { location, .. }
            | Expression::FunctionCall { location, .. } => *location,
        }
    }
}
