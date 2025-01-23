use std::fmt;

use ecow::EcoString;
use vec1::Vec1;

use super::{definition, print::print_parse_tree, typed_print::print_typed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Option<Vec1<Definitions>>,
}

pub type Untyped = Module<definition::DefinitionUntyped>;
pub type Typed = Module<definition::DefinitionTyped>;

impl fmt::Display for Module<definition::DefinitionUntyped> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\nAST structure:")?;
        writeln!(formatter, "-------------")?;

        print_parse_tree(self, formatter)
    }
}

impl fmt::Display for Module<definition::DefinitionTyped> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "\nTyped AST structure:")?;
        writeln!(formatter, "-------------------")?;
        print_typed(self, formatter)
    }
}
