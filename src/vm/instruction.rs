use std::collections::HashMap;

use ecow::EcoString;
#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    PushInt(i64),
    PushFloat(f64),
    PushStr(EcoString),
    PushChar(char),
    PushSlice(Vec<Value>),

    Load(EcoString),  // value from map in stack
    Store(EcoString), // value from stack in map

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
    SetByIndex(i64, Value),

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
    SetField(EcoString, Value),
    GetField(EcoString),

    Print,
    Println,

    Halt,
}

pub type Bytecode = Vec<Instruction>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(EcoString),
    Char(char),
    Slice(Vec<Value>),
    Struct(HashMap<EcoString, Value>),
}
