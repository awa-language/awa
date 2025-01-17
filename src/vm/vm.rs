use super::instruction;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Value {
    Int(i32),
    Float(f32),
    Str(String),
    Struct(HashMap<String, Value>),
}

pub struct VM {
    stack: Vec<Value>,
    variables: HashMap<string, Value>,
    bytecode: Vec<Instruction>,
    pc: usize,
    call_stack: Vec<usize>,
    functions: HashMap<String, usize>,
}

impl VM {
    pub fn new(bytecode: Vec<Instruction>, functions: HashMap<String, usize>) -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            bytecode,
            pc: 0,
            call_stack: Vec::new(),
            functions,
        }
    }
    // TODO: all methods
}
