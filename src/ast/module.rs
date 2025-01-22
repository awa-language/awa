use std::fmt;

use ecow::EcoString;
use vec1::Vec1;

use super::{definition, print::print_parse_tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Option<Vec1<Definitions>>,
}

pub type Untyped = Module<definition::Untyped>;

impl fmt::Display for Module<definition::Untyped> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\nAST structure:")?;
        writeln!(formatter, "-------------")?;

        print_parse_tree(self, formatter)
    }
}
