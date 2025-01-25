use crate::parse::error::ConvertingError;
use crate::parse::error::ConvertingErrorType;
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
        return_type: Type,
    },
}

#[derive(Debug, Clone)]
pub struct StructFieldTyped {
    pub name: EcoString,
    pub type_: Type,
}

impl DefinitionTyped {
    /// Returns the arguments of a definition
    ///
    /// # Errors
    ///
    /// Returns `ConvertingError` if definition is a struct (only functions have arguments)
    pub fn get_arguments(&self) -> Result<Option<Vec1<argument::ArgumentTyped>>, ConvertingError> {
        match self {
            DefinitionTyped::Function { arguments, .. } => Ok(arguments.clone()),
            DefinitionTyped::Struct { .. } => Err(ConvertingError {
                error: ConvertingErrorType::UnsupportedType,
                location: crate::lex::location::Location { start: 0, end: 0 },
            }),
        }
    }

    /// Returns the return type of a definition
    ///
    /// # Errors
    ///
    /// Returns `ConvertingError` if definition is a struct (only functions have return types)
    pub fn get_return_type(&self) -> Result<Type, ConvertingError> {
        match self {
            DefinitionTyped::Function { return_type, .. } => Ok(return_type.clone()),
            DefinitionTyped::Struct { .. } => Err(ConvertingError {
                error: ConvertingErrorType::UnsupportedType,
                location: crate::lex::location::Location { start: 0, end: 0 },
            }),
        }
    }
}
