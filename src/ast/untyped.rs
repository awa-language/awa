use crate::ast::{location::Location, statement::Untyped};
use ecow::EcoString;
use vec1::Vec1;

use super::{argument::Untyped as UntypedArgument, assignment::TypeAst};

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
        arguments: Vec<UntypedArgument>,
        body: Vec1<Untyped>,
        return_annotation: Option<TypeAst>,
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
