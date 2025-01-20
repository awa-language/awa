use std::fmt;

use ecow::EcoString;

use super::{definition, print::print_parse_tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Vec<Definitions>,
}

pub type Untyped = Module<definition::Untyped>;

impl fmt::Display for Module<definition::Untyped> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\nAST structure:")?;
        writeln!(formatter, "-------------")?;

        print_parse_tree(self, formatter)
    }
}
