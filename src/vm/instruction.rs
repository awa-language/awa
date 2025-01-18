use ecow::EcoString;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    PushInt(i64),
    PushFloat(f64),
    PushStr(EcoString),
    PushChar(char),

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

    Call(EcoString),
    Return,

    NewStruct(EcoString),
    SetField(EcoString),
    GetField(EcoString),

    Print,
    Println,

    Halt, // end the program??
    Nop,  // is it needed?
    STW,  // stop-the-world for hot swaps??
}

pub type Bytecode = Vec<Instruction>;
