use ecow::EcoString;
use crate::ast::location::Location;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
        fields: Vec<Box<Type>>,
    },
    Array {
        type_: Box<Type>, // Needed for empty array
        values: Vec<Box<Type>>,
    },
}
