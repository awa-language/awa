use ecow::EcoString;
use vec1::Vec1;

use crate::type_::Type;

use super::{argument, location::Location, statement};

#[derive(Debug)]
pub enum Untyped {
    Struct {
        location: Location,
        name: EcoString,
    },
    Function {
        location: Location,
        name: EcoString,
        arguments: Option<Vec1<argument::Untyped>>,
        body: Option<Vec1<statement::Untyped>>,
        return_type_annotation: Option<Type>,
    },
}
