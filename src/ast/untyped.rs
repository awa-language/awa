use crate::{ast::location::Location, type_::Type};
use ecow::EcoString;
use vec1::Vec1;

use super::{argument, statement};

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
        arguments: Vec<argument::Untyped>,
        body: Vec1<statement::Untyped>,
        return_annotation: Option<Type>,
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
