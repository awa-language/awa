use crate::ast::{location::Location, statement::UntypedStatement};
use ecow::EcoString;
use vec1::Vec1;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UntypedExpression {
    Int {
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
    Float {
        location: Location,
        value: EcoString,
    },
    Todo {
        location: Location,
        message: Option<Box<Self>>,
    },
    Block {
        location: Location,
        statements: Vec1<UntypedStatement>,
    },
}
