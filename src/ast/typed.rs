use ecow::EcoString;
use vec1::Vec1;

use crate::type_::Type;

use super::{argument, location::Location, statement};

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
    CharLiteral {
        location: Location,
        value: EcoString,
    },
    StringLiteral {
        location: Location,
        value: EcoString,
    },
    VariableValue {
        location: Location,
        name: EcoString,
    },
    FunctionCall {
        location: Location,
        arguments: Vec<argument::Typed>,
        body: Vec1<statement::Typed>,
        return_type_annotation: Option<Type>,
    },
    // TODO: Probably remove??
    Todo {
        location: Location,
        message: Option<Box<Self>>,
    },
    Panic {
        location: Location,
        message: Option<Box<Self>>,
    },
    Exit {
        location: Location,
        code: i32,
    },
}
