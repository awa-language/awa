use ecow::EcoString;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
    },
    Array {
        type_: Box<Self>, // Needed for empty array
    },
    Boolean,
    Void,
}

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
    },
    Boolean,
}
