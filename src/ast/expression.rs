use crate::ast::location::Location;
use crate::lex::error::Type;
use ecow::EcoString;
use vec1::Vec1;
use crate::ast::argument::CallArgument;
use crate::ast::operator::BinaryOperator;
use crate::type_::UntypedType;

#[derive(Debug, Clone)]
pub enum TypedExpression {
    IntLiteral {
        location: Location,
        value: i64,
        type_: Type,
    },
    FloatLiteral {
        location: Location,
        value: f64,
        type_: Type,
    },
    StringLiteral {
        location: Location,
        value: EcoString,
        type_: Type,
    },
    CharLiteral {
        location: Location,
        value: char,
        type_: Type,
    },
    VariableValue {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    FunctionCall {
        location: Location,
        function_name: EcoString,
        arguments: Option<Vec1<CallArgumentTyped<Self>>>,
        type_: Vec1<Type>,
    },
    StructFieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
        type_: Type,
    },
    ArrayElementAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<Self>,
        type_: Type,
    },
    ArrayInitialization {
        location: Location,
        type_annotation: Type,
        elements: Option<Vec1<Self>>,
        type_: Type,
    },
    StructInitialization {
        location: Location,
        type_annotation: Type,
        fields: Option<Vec1<StructFieldValueTyped>>,
        type_: Vec1<Type>,
    },
}

impl TypedExpression {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            TypedExpression::IntLiteral { location, .. }
            | TypedExpression::FloatLiteral { location, .. }
            | TypedExpression::CharLiteral { location, .. }
            | TypedExpression::StringLiteral { location, .. }
            | TypedExpression::VariableValue { location, .. }
            | TypedExpression::FunctionCall { location, .. }
            | TypedExpression::StructFieldAccess { location, .. }
            | TypedExpression::ArrayElementAccess { location, .. }
            | TypedExpression::ArrayInitialization { location, .. }
            | TypedExpression::StructInitialization { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldValueTyped {
    pub name: EcoString,
    pub value: TypedExpression,
    pub type_: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgumentTyped<A> {
    pub location: Location,
    pub value: A,
    pub _type: Type,
}

impl PartialEq for TypedExpression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                TypedExpression::IntLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                TypedExpression::IntLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                TypedExpression::FloatLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                TypedExpression::FloatLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && (v1.is_nan() && v2.is_nan() || v1 == v2) && t1 == t2,
            (
                TypedExpression::StringLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                TypedExpression::StringLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                TypedExpression::CharLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                TypedExpression::CharLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                TypedExpression::VariableValue {
                    location: l1,
                    name: n1,
                    type_: t1,
                },
                TypedExpression::VariableValue {
                    location: l2,
                    name: n2,
                    type_: t2,
                },
            ) => l1 == l2 && n1 == n2 && t1 == t2,
            (
                TypedExpression::FunctionCall {
                    location: l1,
                    function_name: fn1,
                    arguments: a1,
                    type_: t1,
                },
                TypedExpression::FunctionCall {
                    location: l2,
                    function_name: fn2,
                    arguments: a2,
                    type_: t2,
                },
            ) => l1 == l2 && fn1 == fn2 && a1 == a2 && t1 == t2,
            (
                TypedExpression::StructFieldAccess {
                    location: l1,
                    struct_name: sn1,
                    field_name: fn1,
                    type_: t1,
                },
                TypedExpression::StructFieldAccess {
                    location: l2,
                    struct_name: sn2,
                    field_name: fn2,
                    type_: t2,
                },
            ) => l1 == l2 && sn1 == sn2 && fn1 == fn2 && t1 == t2,
            (
                TypedExpression::ArrayElementAccess {
                    location: l1,
                    array_name: an1,
                    index_expression: ie1,
                    type_: t1,
                },
                TypedExpression::ArrayElementAccess {
                    location: l2,
                    array_name: an2,
                    index_expression: ie2,
                    type_: t2,
                },
            ) => l1 == l2 && an1 == an2 && ie1 == ie2 && t1 == t2,
            (
                TypedExpression::ArrayInitialization {
                    location: l1,
                    type_annotation: ta1,
                    elements: e1,
                    type_: t1,
                },
                TypedExpression::ArrayInitialization {
                    location: l2,
                    type_annotation: ta2,
                    elements: e2,
                    type_: t2,
                },
            ) => l1 == l2 && ta1 == ta2 && e1 == e2 && t1 == t2,
            (
                TypedExpression::StructInitialization {
                    location: l1,
                    type_annotation: ta1,
                    fields: f1,
                    type_: t1,
                },
                TypedExpression::StructInitialization {
                    location: l2,
                    type_annotation: ta2,
                    fields: f2,
                    type_: t2,
                },
            ) => l1 == l2 && ta1 == ta2 && f1 == f2 && t1 == t2,
            _ => false,
        }
    }
}

impl Eq for TypedExpression {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UntypedExpression {
    IntLiteral {
        location: Location,
        value: EcoString,
    },
    FloatLiteral {
        location: Location,
        value: EcoString,
    },
    StringLiteral {
        location: Location,
        value: EcoString,
    },
    CharLiteral {
        location: Location,
        value: EcoString,
    },
    VariableValue {
        location: Location,
        name: EcoString,
    },
    BinaryOperation {
        location: Location,
        operator: BinaryOperator,
        left: Box<Self>,
        right: Box<Self>,
    },
    FunctionCall {
        location: Location,
        function_name: EcoString,
        arguments: Option<Vec1<CallArgument<Self>>>,
    },
    StructFieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
    },
    ArrayElementAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<Self>,
    },
    ArrayInitialization {
        location: Location,
        type_annotation: UntypedType,
        elements: Option<Vec1<Self>>,
    },
    StructInitialization {
        location: Location,
        type_annotation: UntypedType,
        fields: Option<Vec1<StructFieldValue>>,
    },
}

impl UntypedExpression {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            UntypedExpression::IntLiteral { location, .. }
            | UntypedExpression::FloatLiteral { location, .. }
            | UntypedExpression::CharLiteral { location, .. }
            | UntypedExpression::StringLiteral { location, .. }
            | UntypedExpression::VariableValue { location, .. }
            | UntypedExpression::BinaryOperation { location, .. }
            | UntypedExpression::FunctionCall { location, .. }
            | UntypedExpression::StructFieldAccess { location, .. }
            | UntypedExpression::ArrayElementAccess { location, .. }
            | UntypedExpression::ArrayInitialization { location, .. }
            | UntypedExpression::StructInitialization { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldValue {
    pub name: EcoString,
    pub value: UntypedExpression,
}