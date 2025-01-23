use crate::type_::Type;
use crate::type_::UntypedType;
use ecow::EcoString;

use super::expression::TypedExpression;
use super::expression::UntypedExpression;
use super::location::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentUntyped {
    pub name: EcoString,
    pub location: Location,
    pub type_annotation: UntypedType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentTyped {
    pub name: EcoString,
    pub location: Location,
    pub type_: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgumentUntyped {
    pub location: Location,
    pub value: UntypedExpression,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgumentTyped {
    pub location: Location,
    pub value: TypedExpression,
    pub type_: Type,
}
