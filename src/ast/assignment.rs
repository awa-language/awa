use ecow::EcoString;

use crate::ast::location::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<TypeT, ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub pattern: Pattern<TypeT>,
    pub annotation: Option<TypeAst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern<Type> {
    Int {
        location: Location,
        value: EcoString,
    },
    Float {
        location: Location,
        value: EcoString,
    },
    String {
        location: Location,
        value: EcoString,
    },
    Variable {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    Discard {
        name: EcoString,
        location: Location,
        type_: Type,
    },
}

// TODO: move them to some more appropriate location
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAst {
    Func(TypeAstFunc),
    Var(TypeAstVar),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAstFunc {
    pub location: Location,
    pub arguments: Vec<TypeAst>,
    pub returns: Box<TypeAst>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAstVar {
    pub location: Location,
    pub name: EcoString,
}
