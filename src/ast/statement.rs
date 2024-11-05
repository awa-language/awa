use crate::type_::Type;

use super::{assignment::Assignment, typed, untyped};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<TypeT, ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<TypeT, ExpressionT>),
}

pub type Typed = Statement<std::sync::Arc<Type>, typed::Expression>;
pub type Untyped = Statement<(), untyped::Expression>;
