use ecow::EcoString;
use vec1::Vec1;
use crate::type_::Type;
use crate::type_::UntypedType;

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
        arguments: Option<Vec1<argument::ArgumentUntyped>>,
        body: Option<Vec1<statement::UntypedStatement>>,
        return_type_annotation: Option<UntypedType>,
    },
}

#[derive(Debug)]
pub struct StructField {
    pub name: EcoString,
    pub type_annotation: UntypedType,
}

#[derive(Debug)]
pub enum Typed {
    Struct {
        location: Location,
        name: EcoString,
        fields: Option<Vec1<StructFieldTyped>>,
    },
    Function {
        location: Location,
        name: EcoString,
        arguments: Option<Vec1<argument::ArgumentTyped>>,
        body: Option<Vec1<statement::TypedStatement>>,
        return_type_annotation: Option<Type>,
    },
}


#[derive(Debug)]
pub struct StructFieldTyped {
    pub name: EcoString,
    pub type_annotation: Type,
}