use super::expression::TypedExpression;
use super::expression::UntypedExpression;
use crate::type_::Type;
use crate::{ast::location::Location, type_::UntypedType};
use ecow::EcoString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntypedAssignment {
    pub location: Location,
    pub variable_name: EcoString,
    pub value: Box<UntypedExpression>,
    pub type_annotation: UntypedType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedAssignment {
    pub location: Location,
    pub variable_name: EcoString,
    pub value: Box<TypedExpression>,
    pub type_: Type,
}
