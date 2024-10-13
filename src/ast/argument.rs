use ecow::EcoString;

use super::{assignment::TypeAst, location::Location};

// pub type TypedArgument = Arg<Arc<Type>>;
pub type UntypedArgument = Argument<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Argument<T> {
    pub name: ArgName,
    pub location: Location,
    pub annotation: Option<TypeAst>,
    pub type_: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgName {
    Named { name: EcoString, location: Location },
}
