use crate::{ast::location::Location, type_::Type};

use super::expression::Expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub type_annotation: Type,
}

pub type Typed = Assignment<Expression>;
pub type Untyped = Assignment<Expression>;
