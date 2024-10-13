use ecow::EcoString;

use super::location::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Func(Func),
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub location: Location,
    pub arguments: Vec<Type>,
    pub returns: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub location: Location,
    pub name: EcoString,
}
