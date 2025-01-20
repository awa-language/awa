use ecow::EcoString;

use crate::{ast::location::Location, untyped_type::UntypedType};

use super::expression_untyped::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<ExpressionT> {
    pub location: Location,
    pub variable_name: EcoString,
    pub value: Box<ExpressionT>,
    pub type_annotation: UntypedType,
}

pub type Typed = Assignment<Expression>;
pub type Untyped = Assignment<Expression>;
