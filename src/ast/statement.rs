use super::{assignment::Assignment, untyped::Expression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<TypeT, ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<TypeT, ExpressionT>),
}

// pub type Typed = Statement<Arc<Type>, TypedExpression>;
pub type Untyped = Statement<(), Expression>;
