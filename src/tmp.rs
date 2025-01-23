#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<Definitions> {
    pub name: EcoString,
    pub definitions: Option<Vec1<Definitions>>,
}

pub type Typed = Module<definition::DefinitionTyped>;

pub enum Type {
    Int,
    Float,
    String,
    Char,
    Custom {
        name: EcoString,
    },
    Array {
        type_: Box<Self>, // Needed for empty array
    },
    Boolean,
    Void,
}

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

pub enum TypedStatement {
    Expression(TypedExpression),
    Assignment(TypedAssignment),
    Reassignment(TypedReassignment),
    Loop {
        body: Option<Vec1<Self>>,
        location: Location,
    },
    If {
        condition: Box<TypedExpression>,
        if_body: Option<Vec1<Self>>,
        else_body: Option<Vec1<Self>>,
        location: Location,
    },
    Break {
        location: Location,
    },
    Return {
        location: Location,
        value: Option<Box<TypedExpression>>,
    },
    Todo {
        location: Location,
    },
    Panic {
        location: Location,
    },
    Exit {
        location: Location,
    },
}

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
        arguments: Option<Vec<CallArgumentTyped>>,
        type_: Type,
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
        elements: Option<Vec1<Self>>,
        type_: Type,
    },
    StructInitialization {
        location: Location,
        fields: Option<Vec1<StructFieldValueTyped>>,
        type_: Type,
    },
    BinaryOperation {
        location: Location,
        operator: BinaryOperator,
        left: Box<Self>,
        right: Box<Self>,
        type_: Type,
    },
}

pub struct TypedAssignment {
    pub location: Location,
    pub variable_name: EcoString,
    pub value: Box<TypedExpression>,
    pub type_: Type,
}

pub struct TypedReassignment {
    pub location: Location,
    pub target: TypedReassignmentTarget,
    pub new_value: Box<TypedExpression>,
    pub type_: Type,
}

pub enum TypedReassignmentTarget {
    Variable {
        location: Location,
        name: EcoString,
        type_: Type,
    },
    FieldAccess {
        location: Location,
        struct_name: EcoString,
        field_name: EcoString,
        type_: Type,
    },
    ArrayAccess {
        location: Location,
        array_name: EcoString,
        index_expression: Box<TypedExpression>,
        type_: Type,
    },
}

pub struct ArgumentTyped {
    pub name: EcoString,
    pub location: Location,
    pub type_: Type,
}

pub struct CallArgumentTyped {
    pub location: Location,
    pub value: TypedExpression,
    pub type_: Type,
}

pub struct Location {
    pub start: u32,
    pub end: u32,
}

pub enum BinaryOperator {
    And,
    Or,
    Equal,
    NotEqual,
    LessInt,
    LessEqualInt,
    LessFloat,
    LessEqualFloat,
    GreaterEqualInt,
    GreaterInt,
    GreaterEqualFloat,
    GreaterFloat,
    AdditionInt,
    AdditionFloat,
    SubtractionInt,
    SubtractionFloat,
    MultipicationInt,
    MultipicationFloat,
    DivisionInt,
    DivisionFloat,
    Modulo,
    Concatenation,
}

pub struct StructFieldTyped {
    pub name: EcoString,
    pub type_: Type,
}
