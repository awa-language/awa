use ecow::EcoString;

use super::definition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Vec<Definitions>,
}

pub type Untyped = Module<definition::Untyped>;
