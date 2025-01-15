use super::{assignment::Assignment, expression::Expression, location::Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<ExpressionT>),
    // TODO: add `Loop`, `If` (as `IfElse` with Option<else>?), `Return`
    // NOTE: we won't support else-if's, only if and else
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
