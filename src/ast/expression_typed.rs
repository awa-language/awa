use crate::ast::location::Location;
use crate::lex::error::Type;
use ecow::EcoString;
use vec1::Vec1;

#[derive(Debug, Clone)]
pub enum ExpressionTyped {
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

impl ExpressionTyped {
    #[must_use]
    pub fn get_location(&self) -> Location {
        match self {
            ExpressionTyped::IntLiteral { location, .. }
            | ExpressionTyped::FloatLiteral { location, .. }
            | ExpressionTyped::CharLiteral { location, .. }
            | ExpressionTyped::StringLiteral { location, .. }
            | ExpressionTyped::VariableValue { location, .. }
            | ExpressionTyped::FunctionCall { location, .. }
            | ExpressionTyped::StructFieldAccess { location, .. }
            | ExpressionTyped::ArrayElementAccess { location, .. }
            | ExpressionTyped::ArrayInitialization { location, .. }
            | ExpressionTyped::StructInitialization { location, .. } => *location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldValueTyped {
    pub name: EcoString,
    pub value: ExpressionTyped,
    pub type_: Type,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallArgumentTyped<A> {
    pub location: Location,
    pub value: A,
    pub _type: Type,
}

impl PartialEq for ExpressionTyped {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ExpressionTyped::IntLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                ExpressionTyped::IntLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                ExpressionTyped::FloatLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                ExpressionTyped::FloatLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && (v1.is_nan() && v2.is_nan() || v1 == v2) && t1 == t2,
            (
                ExpressionTyped::StringLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                ExpressionTyped::StringLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                ExpressionTyped::CharLiteral {
                    location: l1,
                    value: v1,
                    type_: t1,
                },
                ExpressionTyped::CharLiteral {
                    location: l2,
                    value: v2,
                    type_: t2,
                },
            ) => l1 == l2 && v1 == v2 && t1 == t2,
            (
                ExpressionTyped::VariableValue {
                    location: l1,
                    name: n1,
                    type_: t1,
                },
                ExpressionTyped::VariableValue {
                    location: l2,
                    name: n2,
                    type_: t2,
                },
            ) => l1 == l2 && n1 == n2 && t1 == t2,
            (
                ExpressionTyped::FunctionCall {
                    location: l1,
                    function_name: fn1,
                    arguments: a1,
                    type_: t1,
                },
                ExpressionTyped::FunctionCall {
                    location: l2,
                    function_name: fn2,
                    arguments: a2,
                    type_: t2,
                },
            ) => l1 == l2 && fn1 == fn2 && a1 == a2 && t1 == t2,
            (
                ExpressionTyped::StructFieldAccess {
                    location: l1,
                    struct_name: sn1,
                    field_name: fn1,
                    type_: t1,
                },
                ExpressionTyped::StructFieldAccess {
                    location: l2,
                    struct_name: sn2,
                    field_name: fn2,
                    type_: t2,
                },
            ) => l1 == l2 && sn1 == sn2 && fn1 == fn2 && t1 == t2,
            (
                ExpressionTyped::ArrayElementAccess {
                    location: l1,
                    array_name: an1,
                    index_expression: ie1,
                    type_: t1,
                },
                ExpressionTyped::ArrayElementAccess {
                    location: l2,
                    array_name: an2,
                    index_expression: ie2,
                    type_: t2,
                },
            ) => l1 == l2 && an1 == an2 && ie1 == ie2 && t1 == t2,
            (
                ExpressionTyped::ArrayInitialization {
                    location: l1,
                    type_annotation: ta1,
                    elements: e1,
                    type_: t1,
                },
                ExpressionTyped::ArrayInitialization {
                    location: l2,
                    type_annotation: ta2,
                    elements: e2,
                    type_: t2,
                },
            ) => l1 == l2 && ta1 == ta2 && e1 == e2 && t1 == t2,
            (
                ExpressionTyped::StructInitialization {
                    location: l1,
                    type_annotation: ta1,
                    fields: f1,
                    type_: t1,
                },
                ExpressionTyped::StructInitialization {
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

impl Eq for ExpressionTyped {}
