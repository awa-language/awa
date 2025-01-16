use vec1::Vec1;

use super::{assignment::Assignment, expression::Expression, location::Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<ExpressionT>),
    Loop {
        body: Vec1<Statement<ExpressionT>>,
        location: Location,
    },
    If {
        condition: Box<ExpressionT>,
        if_body: Vec1<Statement<ExpressionT>>,
        else_body: Option<Vec1<Statement<ExpressionT>>>,
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<ExpressionT>>,
    },
}

pub type Typed = Statement<Expression>;
pub type Untyped = Statement<Expression>;

impl Untyped {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Statement::Expression(expression) => expression.get_location(),
            Statement::Assignment(assignment) => assignment.location,
            Statement::Loop { location, .. } => *location,
            Statement::If { location, .. } => *location,
            Statement::Return { location, .. } => *location,
        }
    }
}
