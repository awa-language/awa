use std::fmt;

use ecow::EcoString;
use vec1::Vec1;
use super::{definition, print::print_parse_tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Vec<Definitions>,
}

pub type Untyped = Module<definition::DefinitionUntyped>;
pub type Typed = Module<definition::DefinitionTyped>;

impl fmt::Display for Module<definition::DefinitionUntyped> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nAST structure:")?;
        writeln!(f, "-------------")?;
        print_parse_tree(self, f)
    }
}
