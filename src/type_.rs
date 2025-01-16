use ecow::EcoString;
use vec1::Vec1;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
        fields: Option<Vec1<Box<Type>>>,
    },
    Array {
        type_: Box<Type>, // Needed for empty array
    },
}
