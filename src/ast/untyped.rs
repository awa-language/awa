use crate::{ast::location::Location, type_::Type};
use ecow::EcoString;
use vec1::Vec1;

use super::{
    argument::{self, CallArgument},
    operator::BinaryOperator,
    statement,
};

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
    VarValue {
        location: Location,
        name: EcoString,
    },
    // TODO: remove because Func is not expression
    Func {
        location: Location,
        name: EcoString,
        arguments: Vec<argument::Untyped>,
        body: Vec1<statement::Untyped>,
        return_type_annotation: Option<Type>,
    },
    FunctionCall {
        location: Location,
        function: Box<Self>,
        arguments: Vec<CallArgument<Self>>,
    },
    // TODO: remove because BinaryOperator is not expression
    BinaryOperator {
        location: Location,
        operator: BinaryOperator,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    // TODO: Probably remove??
    Todo {
        location: Location,
    },
    Panic {
        location: Location,
    },
    Exit {
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<Self>>,
    },
    // TODO: add field access
}

impl Expression {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Expression::IntLiteral { location, .. }
            | Expression::FloatLiteral { location, .. }
            | Expression::CharLiteral { location, .. }
            | Expression::StringLiteral { location, .. }
            | Expression::VarValue { location, .. }
            | Expression::Func { location, .. }
            | Expression::Todo { location, .. }
            | Expression::Panic { location, .. }
            | Expression::Exit { location, .. }
            | Expression::FunctionCall { location, .. }
            | Expression::BinaryOperator { location, .. }
            | Expression::Return { location, .. } => *location,
        }
    }
}
