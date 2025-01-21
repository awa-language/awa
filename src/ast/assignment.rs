use ecow::EcoString;
use crate::{ast::location::Location, type_::UntypedType};
use super::expression::TypedExpression;
use super::expression::UntypedExpression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<ExpressionT> {
    pub location: Location,
    pub variable_name: EcoString,
    pub value: Box<ExpressionT>,
    pub type_annotation: UntypedType,
}

pub type TypedAssignment = Assignment<TypedExpression>;
pub type UntypedAssignment = Assignment<UntypedExpression>;
