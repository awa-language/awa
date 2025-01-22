use ecow::EcoString;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    PushInt(i64),
    PushFloat(f64),
    PushString(EcoString),
    PushChar(char),
    PushArray(Vec<Value>),

    LoadToStack(EcoString),
    StoreInMap(EcoString),

    AddInt,
    SubInt,
    MulInt,
    DivInt,
    Mod,

    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,

    Append(Value),
    GetByIndex(i64),
    SetByIndex(i64),

    Equal,
    NotEqual,

    LessInt,
    LessEqualInt,
    GreaterInt,
    GreaterEqualInt,

    LessFloat,
    LessEqualFloat,
    GreaterFloat,
    GreaterEqualFloat,

    Concat,

    Jump(usize),
    JumpIfTrue(usize),
    JumpIfFalse(usize),

    Func(EcoString),
    EndFunc,
    Call(EcoString),
    Return,

    Struct(EcoString),
    EndStruct,
    NewStruct(EcoString),
    Field(EcoString, Value),
    SetField(EcoString),
    GetField(EcoString),

    Print,
    Println,

    Halt,
}

pub type Bytecode = Vec<Instruction>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Handle(pub usize);

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Char(char),
    String(EcoString),
    Slice(Vec<Value>),
    Struct {
        name: EcoString,
        fields: HashMap<EcoString, Value>,
    },
    Nil,
    Ref(Handle),
}
