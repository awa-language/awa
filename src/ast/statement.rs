#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<TypeT, ExpressionT> {
    /// A bare expression that is not assigned to any variable.
    Expression(ExpressionT),
    /// Assigning an expression to variables using a pattern.
    Assignment(Assignment<TypeT, ExpressionT>),
    /// A `use` expression.
    Use(Use),
}

pub type TypedStatement = Statement<Arc<Type>, TypedExpr>;
pub type UntypedStatement = Statement<(), UntypedExpr>;
