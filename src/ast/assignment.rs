use crate::{ast::location::Location, type_::Type};

use super::{pattern::Pattern, typed, untyped};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<TypeT, ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub pattern: Pattern<TypeT>,
    pub annotation: Option<Type>,
}

pub type Typed = Assignment<std::sync::Arc<Type>, typed::Expression>;
pub type Untyped = Assignment<(), untyped::Expression>;
