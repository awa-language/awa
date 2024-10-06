use crate::ast::location::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<TypeT, ExpressionT> {
    pub location: Location,
    pub value: Box<ExpressionT>,
    pub pattern: Pattern<TypeT>,
    pub kind: AssignmentKind,
    pub annotation: Option<TypeAst>,
}
