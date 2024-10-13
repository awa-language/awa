use ecow::EcoString;

use super::{assignment::TypeAst, location::Location};

// pub type Typed = Arg<Arc<Type>>;
pub type Untyped = Argument<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument<T> {
    pub name: Name,
    pub location: Location,
    pub annotation: Option<TypeAst>,
    pub type_: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Named { name: EcoString, location: Location },
}
