use super::{assignment::Assignment, expression::Expression, location::Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<ExpressionT>),
}

pub type Typed = Statement<Expression>;
pub type Untyped = Statement<Expression>;

impl Untyped {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Statement::Expression(expression) => expression.get_location(),
            Statement::Assignment(assignment) => assignment.location,
        }
    }
}
