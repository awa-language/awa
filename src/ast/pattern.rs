use ecow::EcoString;

use crate::{ast::location::Location, type_::Type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<Type> {
    Int {
        location: Location,
        value: EcoString,
    },
    Float {
        location: Location,
        value: EcoString,
    },
    String {
        location: Location,
        value: EcoString,
    },
    Variable {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    Discard {
        name: EcoString,
        location: Location,
        type_: Type,
    },
}

pub type Typed = Pattern<std::sync::Arc<Type>>;
pub type Untyped = Pattern<()>;
