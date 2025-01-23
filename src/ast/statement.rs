use super::{
    assignment::{TypedAssignment, UntypedAssignment},
    location::Location,
    reassignment::{TypedReassignment, UntypedReassignment},
};
use crate::ast::expression::{TypedExpression, UntypedExpression};
use vec1::Vec1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStatement {
    Expression(TypedExpression),
    Assignment(TypedAssignment),
    Reassignment(TypedReassignment),
    Loop {
        body: Option<Vec1<Self>>,
        location: Location,
    },
    If {
        condition: Box<TypedExpression>,
        if_body: Option<Vec1<Self>>,
        else_body: Option<Vec1<Self>>,
        location: Location,
    },
    Break {
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<TypedExpression>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UntypedStatement {
    Expression(UntypedExpression),
    Assignment(UntypedAssignment),
    Reassignment(UntypedReassignment),
    Loop {
        body: Option<Vec1<Self>>,
        location: Location,
    },
    If {
        condition: Box<UntypedExpression>,
        if_body: Option<Vec1<Self>>,
        else_body: Option<Vec1<Self>>,
        location: Location,
    },
    Break {
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<UntypedExpression>>,
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

impl UntypedStatement {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            UntypedStatement::Expression(expression) => expression.get_location(),
            UntypedStatement::Assignment(assignment) => assignment.location,
            UntypedStatement::Reassignment(reassignment) => reassignment.location,
            UntypedStatement::Loop { location, .. }
            | UntypedStatement::If { location, .. }
            | UntypedStatement::Return { location, .. }
            | UntypedStatement::Todo { location, .. }
            | UntypedStatement::Panic { location, .. }
            | UntypedStatement::Exit { location, .. }
            | UntypedStatement::Break { location, .. } => *location,
        }
    }
}

impl TypedStatement {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            TypedStatement::Expression(expression) => expression.get_location(),
            TypedStatement::Assignment(assignment) => assignment.location,
            TypedStatement::Reassignment(reassignment) => reassignment.location,
            TypedStatement::Loop { location, .. }
            | TypedStatement::If { location, .. }
            | TypedStatement::Return { location, .. }
            | TypedStatement::Todo { location, .. }
            | TypedStatement::Panic { location, .. }
            | TypedStatement::Exit { location, .. }
            | TypedStatement::Break { location, .. } => *location,
        }
    }
}
