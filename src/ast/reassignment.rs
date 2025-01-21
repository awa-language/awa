use ecow::EcoString;

use crate::ast::location::Location;

use super::expression::{TypedExpression, UntypedExpression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reassignment<ExpressionT> {
    pub location: Location,
    pub target: ReassignmentTarget<ExpressionT>,
    pub new_value: Box<ExpressionT>,
}

pub type TypedReassignment = Reassignment<TypedExpression>;
pub type UntypedReassignment = Reassignment<UntypedExpression>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReassignmentTarget<ExpressionT> {
    Variable {
        location: Location,
        name: EcoString,
    },
    FieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
    },
    ArrayAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<ExpressionT>,
    },
}
