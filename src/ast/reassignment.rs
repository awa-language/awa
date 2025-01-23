use ecow::EcoString;

use crate::{ast::location::Location, type_::Type};

use super::expression::{TypedExpression, UntypedExpression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntypedReassignment {
    pub location: Location,
    pub target: UntypedReassignmentTarget,
    pub new_value: Box<UntypedExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedReassignment {
    pub location: Location,
    pub target: TypedReassignmentTarget,
    pub new_value: Box<TypedExpression>,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UntypedReassignmentTarget {
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
        index_expression: Box<UntypedExpression>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedReassignmentTarget {
    Variable {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    FieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
        type_: Type,
    },
    ArrayAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<TypedExpression>,
        type_: Type,
    },
}

impl TypedReassignmentTarget {
    #[must_use]
    pub fn get_type(&self) -> Type {
        match self {
            TypedReassignmentTarget::Variable { type_, .. }
            | TypedReassignmentTarget::FieldAccess { type_, .. }
            | TypedReassignmentTarget::ArrayAccess { type_, .. } => type_.clone(),
        }
    }
}
