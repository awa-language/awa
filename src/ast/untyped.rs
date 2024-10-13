use crate::ast::{location::Location, statement::UntypedStatement};
use ecow::EcoString;
use vec1::Vec1;

use super::{argument::UntypedArgument, assignment::TypeAst};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UntypedExpression {
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
        body: Vec1<UntypedStatement>,
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
