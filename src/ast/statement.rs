use super::{assignment::Assignment, location::Location, typed, untyped};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<ExpressionT>),
}

pub type Typed = Statement<typed::Expression>;
pub type Untyped = Statement<untyped::Expression>;

impl Untyped {
    pub fn get_location(&self) -> Location {
        match self {
            Statement::Expression(expression) => expression.get_location(),
            Statement::Assignment(assignment) => assignment.location,
        }
    }
}
