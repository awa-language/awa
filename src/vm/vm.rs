use super::instruction::Instruction;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Char(char),
    Struct(HashMap<String, Value>),
}

pub struct VM {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    bytecode: Vec<Instruction>,
    pc: usize,
    call_stack: Vec<usize>,
    functions: HashMap<String, usize>,
}

impl VM {
    pub fn new(bytecode: Vec<Instruction>, functions: HashMap<String, usize>, pc: usize) -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            bytecode,
            pc,
            call_stack: Vec::new(),
            functions,
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.pc >= self.bytecode.len() {
                break;
            }

            let instrt = self.bytecode[self.pc].clone();

            match instrt {
                Instruction::PushInt(val) => {
                    self.stack.push(Value::Int(val));
                }
                Instruction::PushFloat(val) => {
                    self.stack.push(Value::Float(val));
                }
                Instruction::PushStr(val) => {
                    self.stack.push(Value::Str(val));
                }
                Instruction::PushChar(val) => {
                    self.stack.push(Value::Char(val));
                }

                Instruction::Load(var) => {
                    if let Some(val) = self.variables.get(&var) {
                        self.stack.push(val.clone());
                    } else {
                        panic!("Undefined variable: {}", var);
                    }
                }
                Instruction::Store(var) => {
                    let val = self.stack.pop().expect("Stack underflow on Store");
                    self.variables.insert(var, val);
                }

                Instruction::AddInt => {
                    let b = self.stack.pop().expect("Stack underflow on AddInt");
                    let a = self.stack.pop().expect("Stack underflow on AddInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                        _ => panic!("Type mismatch for AddInt"),
                    }
                }

                Instruction::SubInt => {
                    let b = self.stack.pop().expect("Stack underflow on SubInt");
                    let a = self.stack.pop().expect("Stack underflow on SubInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x - y)),
                        _ => panic!("Type mismatch for SubInt"),
                    }
                }

                Instruction::MulInt => {
                    let b = self.stack.pop().expect("Stack underflow on MulInt");
                    let a = self.stack.pop().expect("Stack underflow on MulInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x * y)),
                        _ => panic!("Type mismatch for MulInt"),
                    }
                }

                Instruction::DivInt => {
                    let b = self.stack.pop().expect("Stack underflow on DivInt");
                    let a = self.stack.pop().expect("Stack underflow on DivInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            if y == 0 {
                                panic!("Division by zero");
                            }
                            self.stack.push(Value::Int(x / y))
                        }
                        _ => panic!("Type mismatch for DivInt"),
                    }
                }

                Instruction::Mod => {
                    let b = self.stack.pop().expect("Stack underflow on Mod");
                    let a = self.stack.pop().expect("Stack underflow on Mod");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            if y == 0 {
                                panic!("Modulo by zero");
                            }
                            self.stack.push(Value::Int(x % y))
                        }
                        _ => panic!("Type mismatch for Mod"),
                    }
                }

                Instruction::AddFloat => {
                    let b = self.stack.pop().expect("Stack underflow on AddFloat");
                    let a = self.stack.pop().expect("Stack underflow on AddFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x + y)),
                        _ => panic!("Type mismatch for AddFloat"),
                    }
                }

                Instruction::SubFloat => {
                    let b = self.stack.pop().expect("Stack underflow on SubFloat");
                    let a = self.stack.pop().expect("Stack underflow on SubFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x - y)),
                        _ => panic!("Type mismatch for SubFloat"),
                    }
                }

                Instruction::MulFloat => {
                    let b = self.stack.pop().expect("Stack underflow on MulFloat");
                    let a = self.stack.pop().expect("Stack underflow on MulFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(x * y)),
                        _ => panic!("Type mismatch for MulFloat"),
                    }
                }

                Instruction::DivFloat => {
                    let b = self.stack.pop().expect("Stack underflow on DivFloat");
                    let a = self.stack.pop().expect("Stack underflow on DivFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => {
                            if y == 0.0 {
                                panic!("Division by zero");
                            }
                            self.stack.push(Value::Float(x / y))
                        }
                        _ => panic!("Type mismatch for DivFloat"),
                    }
                }

                Instruction::Equal => {
                    let b = self.stack.pop().expect("Stack underflow on Equal");
                    let a = self.stack.pop().expect("Stack underflow on Equal");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x == y) as i64))
                        }
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x == y) as i64))
                        }
                        (Value::Str(x), Value::Str(y)) => {
                            self.stack.push(Value::Int((x == y) as i64))
                        }
                        (Value::Char(x), Value::Char(y)) => {
                            self.stack.push(Value::Int((x == y) as i64))
                        }
                        _ => panic!("Type mismatch for Equal"),
                    }
                }

                Instruction::NotEqual => {
                    let b = self.stack.pop().expect("Stack underflow on NotEqual");
                    let a = self.stack.pop().expect("Stack underflow on NotEqual");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x != y) as i64))
                        }
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x != y) as i64))
                        }
                        (Value::Str(x), Value::Str(y)) => {
                            self.stack.push(Value::Int((x != y) as i64))
                        }
                        (Value::Char(x), Value::Char(y)) => {
                            self.stack.push(Value::Int((x != y) as i64))
                        }
                        _ => panic!("Type mismatch for NotEqual"),
                    }
                }

                Instruction::LessInt => {
                    let b = self.stack.pop().expect("Stack underflow on LessInt");
                    let a = self.stack.pop().expect("Stack underflow on LessInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x < y) as i64))
                        }
                        _ => panic!("Type mismatch for LessInt"),
                    }
                }

                Instruction::LessEqualInt => {
                    let b = self.stack.pop().expect("Stack underflow on LessEqualInt");
                    let a = self.stack.pop().expect("Stack underflow on LessEqualInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x <= y) as i64))
                        }
                        _ => panic!("Type mismatch for LessEqualInt"),
                    }
                }

                Instruction::GreaterInt => {
                    let b = self.stack.pop().expect("Stack underflow on GreaterInt");
                    let a = self.stack.pop().expect("Stack underflow on GreaterInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x > y) as i64))
                        }
                        _ => panic!("Type mismatch for GreaterInt"),
                    }
                }

                Instruction::GreaterEqualInt => {
                    let b = self
                        .stack
                        .pop()
                        .expect("Stack underflow on GreaterEqualInt");
                    let a = self
                        .stack
                        .pop()
                        .expect("Stack underflow on GreaterEqualInt");
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            self.stack.push(Value::Int((x >= y) as i64))
                        }
                        _ => panic!("Type mismatch for GreaterEqualInt"),
                    }
                }

                Instruction::LessFloat => {
                    let b = self.stack.pop().expect("Stack underflow on LessFloat");
                    let a = self.stack.pop().expect("Stack underflow on LessFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x < y) as i64))
                        }
                        _ => panic!("Type mismatch for LessFloat"),
                    }
                }

                Instruction::LessEqualFloat => {
                    let b = self.stack.pop().expect("Stack underflow on LessEqualFloat");
                    let a = self.stack.pop().expect("Stack underflow on LessEqualFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x <= y) as i64))
                        }
                        _ => panic!("Type mismatch for LessEqualFloat"),
                    }
                }

                Instruction::GreaterFloat => {
                    let b = self.stack.pop().expect("Stack underflow on GreaterFloat");
                    let a = self.stack.pop().expect("Stack underflow on GreaterFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x > y) as i64))
                        }
                        _ => panic!("Type mismatch for GreaterFloat"),
                    }
                }

                Instruction::GreaterEqualFloat => {
                    let b = self
                        .stack
                        .pop()
                        .expect("Stack underflow on GreaterEqualFloat");
                    let a = self
                        .stack
                        .pop()
                        .expect("Stack underflow on GreaterEqualFloat");
                    match (a, b) {
                        (Value::Float(x), Value::Float(y)) => {
                            self.stack.push(Value::Int((x >= y) as i64))
                        }
                        _ => panic!("Type mismatch for GreaterEqualFloat"),
                    }
                }

                Instruction::Concat => {
                    let b = self.stack.pop().expect("Stack underflow on Concat");
                    let a = self.stack.pop().expect("Stack underflow on Concat");
                    match (a, b) {
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(x + &y)),
                        _ => panic!("Type mismatch for Concat"),
                    }
                }

                Instruction::Jump(addr) => {
                    if addr >= self.bytecode.len() {
                        panic!("Jump to invalid address: {}", addr);
                    }
                    self.pc = addr;
                    continue;
                }

                Instruction::JumpIfTrue(addr) => {
                    let condition = self.stack.pop().expect("Stack underflow on JumpIfTrue");
                    let is_true = match condition {
                        Value::Int(v) => v != 0,
                        _ => panic!("Type mismatch for JumpIfTrue"),
                    };
                    if is_true {
                        if addr >= self.bytecode.len() {
                            panic!("JumpIfTrue to invalid address: {}", addr);
                        }
                        self.pc = addr;
                        continue;
                    }
                }

                Instruction::JumpIfFalse(addr) => {
                    let condition = self.stack.pop().expect("Stack underflow on JumpIfFalse");
                    let is_false = match condition {
                        Value::Int(v) => v == 0,
                        _ => panic!("Type mismatch for JumpIfFalse"),
                    };
                    if is_false {
                        if addr >= self.bytecode.len() {
                            panic!("JumpIfFalse to invalid address: {}", addr);
                        }
                        self.pc = addr;
                        continue;
                    }
                }

                Instruction::Call(func_name) => {
                    if let Some(&addr) = self.functions.get(&func_name) {
                        self.call_stack.push(self.pc + 1);
                        self.pc = addr;
                        continue;
                    } else {
                        panic!("Undefined function: {}", func_name);
                    }
                }

                Instruction::Return => {
                    if let Some(return_addr) = self.call_stack.pop() {
                        self.pc = return_addr;
                        continue;
                    } else {
                        println!("Return from main function.");
                        break;
                    }
                }

                Instruction::NewStruct(_struct_name) => {
                    self.stack.push(Value::Struct(HashMap::new()));
                }

                Instruction::SetField(field_name) => {
                    let value = self.stack.pop().expect("Stack underflow on SetField");
                    let mut struct_val = match self.stack.pop().expect("Expected struct on stack") {
                        Value::Struct(map) => map,
                        _ => panic!("SetField expects a struct"),
                    };

                    struct_val.insert(field_name, value);
                    self.stack.push(Value::Struct(struct_val));
                }

                Instruction::GetField(field_name) => {
                    let struct_val = match self.stack.pop().expect("Expected struct on stack") {
                        Value::Struct(map) => map,
                        _ => panic!("GetField expects a struct"),
                    };

                    if let Some(val) = struct_val.get(&field_name) {
                        self.stack.push(val.clone());
                    } else {
                        panic!("Field {} does not exist in struct", field_name);
                    }
                }

                Instruction::Print => {
                    let val = self.stack.last().expect("Stack underflow on Print");
                    print!("{:?}", val);
                }
                Instruction::Println => {
                    let val = self.stack.last().expect("Stack underflow on Println");
                    println!("{:?}", val);
                }

                Instruction::Halt => {
                    println!("Program halted.");
                    break;
                }

                Instruction::Nop => {}

                Instruction::STW => {
                    // TODO
                    println!("STW (Stop The World) executed.");
                }
            }

            self.pc += 1;
        }
    }
}
