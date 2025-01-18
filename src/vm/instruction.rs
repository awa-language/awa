#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    PushInt(i64),
    PushFloat(f64),
    PushStr(String),
    PushChar(char),

    Load(String),  //val to stack
    Store(String), //val from stack

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

    Call(string), //func call
    Return,

    NewStruct(String),
    SetField(String),
    GetField(String),

    Print,
    Println,

    Halt, //end program ??
    Nop,  //nothing ??
    STW,  //stop the world for hot swap ??
}
