use ecow::EcoString;
use vec1::Vec1;

use crate::type_::Type;

use super::{argument, location::Location, statement};

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
    Char {
        location: Location,
        value: EcoString,
    },
    String {
        location: Location,
        value: EcoString,
    },
    Var {
        location: Location,
        name: EcoString,
    },
    Func {
        location: Location,
        arguments: Vec<argument::Typed>,
        body: Vec1<statement::Typed>,
        return_type_annotation: Option<Type>,
    },
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
