use ecow::EcoString;

use crate::ast::location::Location;

use super::types::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<TypeT, ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub pattern: Pattern<TypeT>,
    pub annotation: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<Type> {
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
    Variable {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    Discard {
        name: EcoString,
        location: Location,
        type_: Type,
    },
}
