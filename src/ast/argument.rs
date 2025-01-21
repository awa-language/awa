use crate::type_::Type;
use crate::type_::UntypedType;
use ecow::EcoString;

use super::location::Location;

pub type ArgumentTyped = Argument<Type>;
pub type ArgumentUntyped = Argument<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument<T> {
    pub name: Name,
    pub location: Location,
    pub type_annotation: UntypedType,
    pub type_: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Named { name: EcoString, location: Location },
}

// TODO: perhaps move somewhere else?
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgument<A> {
    pub location: Location,
    pub value: A,
}
