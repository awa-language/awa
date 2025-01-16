use ecow::EcoString;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
        fields: Option<Vec<Box<Type>>>,
    },
    Array {
        type_: Box<Type>, // Needed for empty array
    },
}
