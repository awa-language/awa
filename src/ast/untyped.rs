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
    Int {
        location: Location,
        value: EcoString,
    },
    Float {
        location: Location,
        value: EcoString,
    },
    String {
        location: Location,
        value: EcoString,
    },
    Char {
        location: Location,
        value: EcoString,
    },
    Var {
        location: Location,
        name: EcoString,
    },
    Func {
        location: Location,
        name: EcoString,
        arguments: Vec<argument::Untyped>,
        body: Vec1<statement::Untyped>,
        return_type_annotation: Option<Type>,
    },
    Call {
        location: Location,
        function: Box<Self>,
        arguments: Vec<CallArgument<Self>>,
    },
    BinaryOperator {
        location: Location,
        operator: BinaryOperator,
        lhs: Box<Self>,
        rhs: Box<Self>,
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
            Expression::Int { location, .. }
            | Expression::Float { location, .. }
            | Expression::Char { location, .. }
            | Expression::String { location, .. }
            | Expression::Var { location, .. }
            | Expression::Func { location, .. }
            | Expression::Todo { location, .. }
            | Expression::Panic { location, .. }
            | Expression::Exit { location, .. }
            | Expression::Call { location, .. }
            | Expression::BinaryOperator { location, .. }
            | Expression::Return { location, .. } => *location,
        }
    }
}
