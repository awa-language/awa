use crate::ast::expression::Expression;
use crate::{ast::location::Location, type_::Type};
use ecow::EcoString;
use vec1::Vec1;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ExpressionTyped {
    IntLiteral {
        location: Location,
        value: EcoString,
        type_: Type,
    },
    FloatLiteral {
        location: Location,
        value: EcoString,
        type_: Type,
    },
    StringLiteral {
        location: Location,
        value: EcoString,
        type_: Type,
    },
    CharLiteral {
        location: Location,
        value: EcoString,
        type_: Type,
    },
    VariableValue {
        location: Location,
        name: EcoString,
    },
    FunctionCall {
        location: Location,
        function_name: EcoString,
        arguments: Option<Vec1<CallArgumentTyped<Self>>>,
        type_: Type,
    },
    StructFieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
        type_: Type,
    },
    ArrayElementAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<Self>,
        type_: Type,
    },
    ArrayInitialization {
        location: Location,
        type_annotation: Type,
        elements: Option<Vec1<Self>>,
        type_: Type,
    },
    StructInitialization {
        location: Location,
        type_annotation: Type,
        fields: Option<Vec1<StructFieldValueTyped>>,
        type_: Type,
    },
}

impl ExpressionTyped {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            ExpressionTyped::IntLiteral { location, .. }
            | ExpressionTyped::FloatLiteral { location, .. }
            | ExpressionTyped::CharLiteral { location, .. }
            | ExpressionTyped::StringLiteral { location, .. }
            | ExpressionTyped::VariableValue { location, .. }
            | ExpressionTyped::FunctionCall { location, .. }
            | ExpressionTyped::StructFieldAccess { location, .. }
            | ExpressionTyped::ArrayElementAccess { location, .. }
            | ExpressionTyped::ArrayInitialization { location, .. }
            | ExpressionTyped::StructInitialization { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldValueTyped {
    pub name: EcoString,
    pub value: Expression,
    pub type_: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgumentTyped<A> {
    pub location: Location,
    pub value: A,
    pub _type: Type,
}
