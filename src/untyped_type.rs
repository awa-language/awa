use ecow::EcoString;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UntypedType {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
    },
    Array {
        type_: Box<Self>, // Needed for empty array
    }
}
