use crate::type_::Type;
use crate::type_::UntypedType;
use ecow::EcoString;
use vec1::Vec1;

use super::{argument, location::Location, statement};

#[derive(Debug)]
pub enum DefinitionUntyped {
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

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: EcoString,
    pub type_annotation: UntypedType,
}

#[derive(Debug, Clone)]
pub enum DefinitionTyped {
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
        return_type: Option<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct StructFieldTyped {
    pub name: EcoString,
    pub type_annotation: Type,
}

impl DefinitionTyped {
    pub fn get_arguments(&self) -> Option<Vec1<argument::ArgumentTyped>> {
        match self {
            DefinitionTyped::Function { arguments, .. } => arguments.clone(),
            DefinitionTyped::Struct { .. } => None,
        }
    }
}

