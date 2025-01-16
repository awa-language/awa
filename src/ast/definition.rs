use ecow::EcoString;
use vec1::Vec1;

use crate::type_::Type;

use super::{argument, location::Location, statement};

#[derive(Debug)]
pub enum Untyped {
    Struct {
        location: Location,
        name: EcoString,
        fields: Option<Vec1<StructField>>,
    },
    Function {
        location: Location,
        name: EcoString,
        arguments: Vec<argument::Untyped>,
        body: Option<Vec1<statement::Untyped>>,
        return_type_annotation: Option<Type>,
    },
}

#[derive(Debug)]
pub struct StructField {
    name: EcoString,
    type_: Type,
}
