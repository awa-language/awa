use std::sync::Arc;

use ecow::EcoString;
use vec1::Vec1;

use crate::{
    lex::{location::Location, string::StringSpan},
    type_::Type,
};

use super::{argument::Argument, statement::Statement, typed, untyped};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Definition<T, Expr> {
    Function(Function<T, Expr>),

    CustomType(CustomType<T>),
}

pub type TypedFunction = Function<Arc<Type>, typed::Expression>;
pub type UntypedFunction = Function<(), untyped::Expression>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function<T, Expr> {
    pub location: Location,
    pub end_position: u32,
    pub name: StringSpan,
    pub arguments: Vec<Argument<T>>,
    pub body: Vec1<Statement<T, Expr>>,
    pub return_annotation: Option<Type>,
    pub return_type: T,
}

pub type UntypedCustomType = CustomType<()>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomType<T> {
    pub location: Location,
    pub end_position: u32,
    pub name: EcoString,
    pub name_location: Location,
    pub parameter_names: Vec<StringSpan>,
    pub typed_parameters: Vec<T>,
}
