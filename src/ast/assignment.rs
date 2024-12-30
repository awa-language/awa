use crate::{ast::location::Location, type_::Type};

use super::{typed, untyped};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub annotation: Type,
}

pub type Typed = Assignment<typed::Expression>;
pub type Untyped = Assignment<untyped::Expression>;
