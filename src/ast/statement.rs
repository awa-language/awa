use vec1::Vec1;
use crate::ast::expression::{TypedExpression, UntypedExpression};
use super::{
    assignment::Assignment, location::Location,
    reassignment::Reassignment,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<ExpressionT> {
    Expression(ExpressionT),
    Assignment(Assignment<ExpressionT>),
    Reassignment(Reassignment<ExpressionT>),
    Loop {
        body: Option<Vec1<Statement<ExpressionT>>>,
        location: Location,
    },
    If {
        condition: Box<ExpressionT>,
        if_body: Option<Vec1<Statement<ExpressionT>>>,
        else_body: Option<Vec1<Statement<ExpressionT>>>,
        location: Location,
    },
    Break {
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<ExpressionT>>,
    },
    Todo {
        location: Location,
    },
    Panic {
        location: Location,
    },
    Exit {
        location: Location,
    },
}

pub type TypedStatement = Statement<TypedExpression>;
pub type UntypedStatement = Statement<UntypedExpression>;

impl UntypedStatement {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Statement::Expression(expression) => expression.get_location(),
            Statement::Assignment(assignment) => assignment.location,
            Statement::Reassignment(reassignment) => reassignment.location,
            Statement::Loop { location, .. }
            | Statement::If { location, .. }
            | Statement::Return { location, .. }
            | Statement::Todo { location, .. }
            | Statement::Panic { location, .. }
            | Statement::Exit { location, .. }
            | Statement::Break { location, .. } => *location,
        }
    }
}

impl TypedStatement {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            Statement::Expression(expression) => expression.get_location(),
            Statement::Assignment(assignment) => assignment.location,
            Statement::Reassignment(reassignment) => reassignment.location,
            Statement::Loop { location, .. }
            | Statement::If { location, .. }
            | Statement::Return { location, .. }
            | Statement::Todo { location, .. }
            | Statement::Panic { location, .. }
            | Statement::Exit { location, .. }
            | Statement::Break { location, .. } => *location,
        }
    }
}
