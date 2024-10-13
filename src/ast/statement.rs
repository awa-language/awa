use super::{assignment::Assignment, untyped::UntypedExpression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<TypeT, ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<TypeT, ExpressionT>),
}

// pub type TypedStatement = Statement<Arc<Type>, TypedExpression>;
pub type UntypedStatement = Statement<(), UntypedExpression>;
