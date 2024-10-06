use crate::ast::location::Location;
use vec1::Vec1;
use ecow::EcoString;


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UntypedExpression{
    Int32 {
        location: Location,
        value: EcoString,
    },

    Int64 {
        location: Location,
        value: EcoString,
    },

    Float32 {
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
    Float64 {
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

