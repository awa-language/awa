pub mod instruction;

#[cfg(test)]
pub mod tests;
use std::collections::HashMap;

use ecow::EcoString;
use instruction::{Instruction, Value};

#[derive(Debug, PartialEq, Clone)]

pub struct VM {
    input: Vec<Instruction>,
    program_counter: usize,
    stack: Vec<Value>,
    variables: HashMap<EcoString, Value>,
    structures: HashMap<EcoString, HashMap<EcoString, Value>>,
    functions: HashMap<EcoString, usize>,
    call_stack: Vec<usize>,
}

impl VM {
    #[must_use]
    pub fn new(bytecode: Vec<Instruction>) -> Self {
        VM {
            input: bytecode,
            program_counter: 0,
            stack: Vec::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
            structures: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        self.preprocess();
        if let Some(&main_addr) = self.functions.get("main") {
            self.program_counter = main_addr;
        } else {
            panic!("cannot find function `main()`");
        }

        loop {
            if self.program_counter >= self.input.len() {
                break;
            }

            let current_instruction = self.input[self.program_counter].clone();

            match current_instruction {
                Instruction::Func(_) | Instruction::EndFunc => {
                    panic!("declare func into main")
                }

                Instruction::Struct(_) | Instruction::EndStruct | Instruction::Field(_, _) => {
                    panic!("declare struct into main")
                }

                Instruction::PushInt(value) => {
                    self.stack.push(Value::Int(value));
                }
                Instruction::PushFloat(value) => {
                    self.stack.push(Value::Float(value));
                }
                Instruction::PushStr(value) => {
                    self.stack.push(Value::String(value));
                }
                Instruction::PushChar(value) => {
                    self.stack.push(Value::Char(value));
                }
                Instruction::PushSlice(value) => {
                    self.stack.push(Value::Slice(value));
                }

                Instruction::Load(variable) => {
                    if let Some(value) = self.variables.get(&variable) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("undefined variable: {variable}");
                    }
                }
                Instruction::Store(variable) => {
                    let value = self.stack.pop().expect("stack underflow on Store");
                    self.variables.insert(variable, value);
                }

                Instruction::AddInt => {
                    let rhs = self.stack.pop().expect("stack underflow on AddInt");
                    let lhs = self.stack.pop().expect("stack underflow on AddInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(lhs + rhs));
                        }
                        _ => panic!("type mismatch for AddInt"),
                    }
                }

                Instruction::SubInt => {
                    let rhs = self.stack.pop().expect("stack underflow on SubInt");
                    let lhs = self.stack.pop().expect("stack underflow on SubInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(lhs - rhs));
                        }
                        _ => panic!("type mismatch for SubInt"),
                    }
                }

                Instruction::MulInt => {
                    let rhs = self.stack.pop().expect("stack underflow on MulInt");
                    let lhs = self.stack.pop().expect("stack underflow on MulInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(lhs * rhs));
                        }
                        _ => panic!("type mismatch for MulInt"),
                    }
                }

                Instruction::DivInt => {
                    let rhs = self.stack.pop().expect("stack underflow on DivInt");
                    let lhs = self.stack.pop().expect("stack underflow on DivInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            assert!(rhs != 0, "division by zero");
                            self.stack.push(Value::Int(lhs / rhs));
                        }
                        _ => panic!("type mismatch for DivInt"),
                    }
                }

                Instruction::Mod => {
                    let rhs = self.stack.pop().expect("stack underflow on Mod");
                    let lhs = self.stack.pop().expect("stack underflow on Mod");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            assert!(rhs != 0, "modulo by zero");
                            self.stack.push(Value::Int(lhs % rhs));
                        }
                        _ => panic!("type mismatch for Mod"),
                    }
                }

                Instruction::AddFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on AddFloat");
                    let lhs = self.stack.pop().expect("stack underflow on AddFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Float(lhs + rhs));
                        }
                        _ => panic!("type mismatch for AddFloat"),
                    }
                }

                Instruction::SubFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on SubFloat");
                    let lhs = self.stack.pop().expect("stack underflow on SubFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Float(lhs - rhs));
                        }
                        _ => panic!("type mismatch for SubFloat"),
                    }
                }

                Instruction::MulFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on MulFloat");
                    let lhs = self.stack.pop().expect("stack underflow on MulFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Float(lhs * rhs));
                        }
                        _ => panic!("type mismatch for MulFloat"),
                    }
                }

                Instruction::DivFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on DivFloat");
                    let lhs = self.stack.pop().expect("stack underflow on DivFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            assert!(!(rhs == 0.0), "division by zero");
                            self.stack.push(Value::Float(lhs / rhs));
                        }
                        _ => panic!("type mismatch for DivFloat"),
                    }
                }

                Instruction::Append(value) => {
                    let slice = self.stack.pop().expect("something");

                    match slice {
                        Value::Slice(mut slice) => {
                            slice.push(value);
                            self.stack.push(Value::Slice(slice));
                        }
                        _ => {}
                    }
                }

                Instruction::GetByIndex(index) => {
                    let slice = self.stack.pop().expect("something");

                    match slice {
                        Value::Slice(slice) => {
                            self.stack.push(slice[index as usize].clone());
                        }
                        _ => {}
                    }
                }

                Instruction::SetByIndex(index, value) => {
                    let slice = self.stack.pop().expect("something");

                    match slice {
                        Value::Slice(mut slice) => {
                            slice[index as usize] = value;
                            self.stack.push(Value::Slice(slice));
                        }
                        _ => {}
                    }
                }

                Instruction::Equal => {
                    let rhs = self.stack.pop().expect("stack underflow on Equal");
                    let lhs = self.stack.pop().expect("stack underflow on Equal");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs == rhs)));
                        }
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs == rhs)));
                        }
                        (Value::String(lhs), Value::String(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs == rhs)));
                        }
                        (Value::Char(lhs), Value::Char(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs == rhs)));
                        }
                        _ => panic!("type mismatch for Equal"),
                    }
                }

                Instruction::NotEqual => {
                    let rhs = self.stack.pop().expect("stack underflow on NotEqual");
                    let lhs = self.stack.pop().expect("stack underflow on NotEqual");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs != rhs)));
                        }
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs != rhs)));
                        }
                        (Value::String(lhs), Value::String(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs != rhs)));
                        }
                        (Value::Char(lhs), Value::Char(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs != rhs)));
                        }
                        _ => panic!("type mismatch for NotEqual"),
                    }
                }

                Instruction::LessInt => {
                    let rhs = self.stack.pop().expect("stack underflow on LessInt");
                    let lhs = self.stack.pop().expect("stack underflow on LessInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs < rhs)));
                        }
                        _ => panic!("type mismatch for LessInt"),
                    }
                }

                Instruction::LessEqualInt => {
                    let rhs = self.stack.pop().expect("stack underflow on LessEqualInt");
                    let lhs = self.stack.pop().expect("stack underflow on LessEqualInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs <= rhs)));
                        }
                        _ => panic!("type mismatch for LessEqualInt"),
                    }
                }

                Instruction::GreaterInt => {
                    let rhs = self.stack.pop().expect("stack underflow on GreaterInt");
                    let lhs = self.stack.pop().expect("stack underflow on GreaterInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs > rhs)));
                        }
                        _ => panic!("type mismatch for GreaterInt"),
                    }
                }

                Instruction::GreaterEqualInt => {
                    let rhs = self
                        .stack
                        .pop()
                        .expect("stack underflow on GreaterEqualInt");
                    let lhs = self
                        .stack
                        .pop()
                        .expect("stack underflow on GreaterEqualInt");

                    match (lhs, rhs) {
                        (Value::Int(lhs), Value::Int(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs >= rhs)));
                        }
                        _ => panic!("type mismatch for GreaterEqualInt"),
                    }
                }

                Instruction::LessFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on LessFloat");
                    let lhs = self.stack.pop().expect("stack underflow on LessFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs < rhs)));
                        }
                        _ => panic!("type mismatch for LessFloat"),
                    }
                }

                Instruction::LessEqualFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on LessEqualFloat");
                    let lhs = self.stack.pop().expect("stack underflow on LessEqualFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs <= rhs)));
                        }
                        _ => panic!("type mismatch for LessEqualFloat"),
                    }
                }

                Instruction::GreaterFloat => {
                    let rhs = self.stack.pop().expect("stack underflow on GreaterFloat");
                    let lhs = self.stack.pop().expect("stack underflow on GreaterFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs > rhs)));
                        }
                        _ => panic!("type mismatch for GreaterFloat"),
                    }
                }

                Instruction::GreaterEqualFloat => {
                    let rhs = self
                        .stack
                        .pop()
                        .expect("stack underflow on GreaterEqualFloat");
                    let lhs = self
                        .stack
                        .pop()
                        .expect("stack underflow on GreaterEqualFloat");

                    match (lhs, rhs) {
                        (Value::Float(lhs), Value::Float(rhs)) => {
                            self.stack.push(Value::Int(i64::from(lhs >= rhs)));
                        }
                        _ => panic!("type mismatch for GreaterEqualFloat"),
                    }
                }

                Instruction::Concat => {
                    let rhs = self.stack.pop().expect("stack underflow on Concat");
                    let lhs = self.stack.pop().expect("stack underflow on Concat");

                    match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => {
                            self.stack.push(Value::String(lhs + rhs));
                        }
                        _ => panic!("type mismatch for Concat"),
                    }
                }

                Instruction::Jump(address) => {
                    assert!(
                        address < self.input.len(),
                        "jump to invalid address: {address}"
                    );
                    self.program_counter = address;

                    continue;
                }

                Instruction::JumpIfTrue(address) => {
                    let condition = self.stack.pop().expect("stack underflow on JumpIfTrue");

                    let is_true = match condition {
                        Value::Int(value) => value != 0,
                        _ => panic!("type mismatch for JumpIfTrue"),
                    };

                    if is_true {
                        assert!(
                            address < self.input.len(),
                            "jumpIfTrue to invalid address: {address}"
                        );
                        self.program_counter = address;

                        continue;
                    }
                }

                Instruction::JumpIfFalse(address) => {
                    let condition = self.stack.pop().expect("stack underflow on JumpIfFalse");

                    let is_false = match condition {
                        Value::Int(value) => value == 0,
                        _ => panic!("type mismatch for JumpIfFalse"),
                    };

                    if is_false {
                        assert!(
                            address < self.input.len(),
                            "JumpIfFalse to invalid address: {address}"
                        );
                        self.program_counter = address;

                        continue;
                    }
                }

                Instruction::Call(function_name) => {
                    if let Some(&address) = self.functions.get(&function_name) {
                        self.call_stack.push(self.program_counter + 1);
                        self.program_counter = address;

                        continue;
                    } else {
                        panic!("undefined function: {function_name}");
                    }
                }

                Instruction::Return => {
                    if let Some(return_address) = self.call_stack.pop() {
                        self.program_counter = return_address;
                        continue;
                    } else {
                        println!("Return from main function.");
                        break;
                    }
                }

                Instruction::NewStruct(struct_type) => {
                    if let Some(fields) = self.structures.get(&struct_type.clone()) {
                        self.stack.push(Value::Struct(fields.clone()))
                    } else {
                        panic!("undefined structer: {struct_type}")
                    }
                }

                Instruction::SetField(field_name, field_value) => {
                    let struct_value = match self.stack.pop() {
                        Some(value) => value,
                        None => {
                            panic!("stack is empty while setting the field '{}'", field_name);
                        }
                    };

                    //TODO may require type mismatch checking
                    match struct_value {
                        Value::Struct(mut map) => {
                            if let Some(_) = map.get(&field_name.clone()) {
                                map.insert(field_name.clone(), field_value.clone());
                                self.stack.push(Value::Struct(map));
                            } else {
                                panic!("unknown struct field");
                            }
                        }
                        _ => {}
                    }
                }

                Instruction::GetField(field_name) => {
                    let struct_value = match self.stack.pop().expect("expected struct on stack") {
                        Value::Struct(map) => map,
                        _ => panic!("GetField expects a struct"),
                    };

                    if let Some(value) = struct_value.get(&field_name) {
                        self.stack.push(value.clone());
                    } else {
                        panic!("field {field_name} does not exist in struct");
                    }
                }

                Instruction::Print => {
                    let value = self.stack.last().expect("stack underflow on Print");
                    print!("{value:?}");
                }
                Instruction::Println => {
                    let value = self.stack.last().expect("stack underflow on Println");
                    println!("{value:?}");
                }

                Instruction::Halt => {
                    break;
                }
            }

            self.program_counter += 1;
        }
    }

    pub fn preprocess(&mut self) {
        let mut process_counter = 0;
        let mut inside_function = false;

        while process_counter < self.input.len() {
            match &self.input[process_counter] {
                Instruction::Func(func_name) => {
                    if inside_function {
                        panic!("cannot define function inside function body");
                    }

                    let func_start = process_counter + 1;
                    let mut func_end = None;

                    for i in (process_counter + 1)..self.input.len() {
                        if let Instruction::EndFunc = self.input[i] {
                            func_end = Some(i);
                            break;
                        }
                    }

                    if let Some(end) = func_end {
                        self.functions.insert(func_name.clone(), func_start);
                        process_counter = end + 1;
                        inside_function = false;

                        continue;
                    } else {
                        panic!(
                            "Func without corresponding EndFunc for function {}",
                            func_name
                        );
                    }
                }

                Instruction::Struct(struct_name) => {
                    if inside_function {
                        panic!("cannot define struct inside function body");
                    }

                    let mut fields = HashMap::new();
                    process_counter += 1;

                    while process_counter < self.input.len() {
                        match &self.input[process_counter] {
                            Instruction::Field(field_name, field_type) => {
                                fields.insert(field_name.clone(), field_type.clone());
                            }
                            Instruction::EndStruct => {
                                break;
                            }
                            _ => {
                                panic!("unsupported instruction inside struct definition");
                            }
                        }

                        process_counter += 1;
                    }

                    if process_counter >= self.input.len() {
                        panic!(
                            "Struct without corresponding EndStruct for struct {}",
                            struct_name
                        );
                    }

                    self.structures.insert(struct_name.clone(), fields);
                }

                _ => {
                    process_counter += 1;
                }
            }
        }
    }
}
