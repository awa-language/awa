use core::f64;
use std::collections::HashMap;

use ecow::EcoString;
pub mod instruction;

mod gc;
#[cfg(test)]
pub mod tests;

use gc::{Object, GC};
use instruction::{Bytecode, Instruction, Value};

pub struct VM {
    pub input: Bytecode,
    pub program_counter: usize,
    pub stack: Vec<Value>,

    /// Глобальные переменные (видны везде)
    pub global_variables: HashMap<EcoString, Value>,

    /// Стек окружений для локальных переменных (каждый вызов Func -> push, Return -> pop)
    pub environments_stack: Vec<HashMap<EcoString, Value>>,

    pub structures: HashMap<EcoString, HashMap<EcoString, Value>>,
    pub functions: HashMap<EcoString, usize>,
    pub call_stack: Vec<usize>,

    pub gc: GC,
}

impl VM {
    #[must_use]
    pub fn new(input: Vec<Instruction>) -> Self {
        let mut vm = Self {
            input,
            program_counter: 0,
            stack: Vec::new(),
            environments_stack: Vec::new(),
            global_variables: HashMap::new(),
            structures: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
            gc: GC::new(),
        };
        vm.preprocess_bytecode();

        if let Some(&main_address) = vm.functions.get("main") {
            vm.program_counter = main_address;
        } else {
            panic!("cannot find function `main()`");
        }
        vm
    }

    pub fn run(&mut self) {
        if self.program_counter >= self.input.len() {
            return;
        }

        let instruction = self.input[self.program_counter].clone();
        match instruction {
            Instruction::PushInt(int) => {
                self.stack.push(Value::Int(int));
            }
            Instruction::PushFloat(float) => {
                self.stack.push(Value::Float(float));
            }
            Instruction::PushChar(char) => {
                self.stack.push(Value::Char(char));
            }
            Instruction::PushString(string) => {
                let handle = self.gc.allocate(Object::String(string));

                self.stack.push(Value::Ref(handle));
                self.maybe_run_gc();
            }
            Instruction::PushArray(array) => {
                let handle = self.gc.allocate(Object::Slice(array));

                self.stack.push(Value::Ref(handle));
                self.maybe_run_gc();
            }
            Instruction::StoreInMap(name) => {
                let val = self.stack.pop().expect("stack underflow");
                if let Some(env) = self.environments_stack.last_mut() {
                    env.insert(name, val);
                } else {
                    self.global_variables.insert(name, val);
                }
            }
            Instruction::LoadToStack(name) => {
                if let Some(variable_value) = self.lookup_variable(&name) {
                    self.stack.push(variable_value.clone());
                } else {
                    panic!("undefined variable: {name}");
                }
            }
            Instruction::AddInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");

                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(x + y));
            }
            Instruction::SubInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(x - y));
            }
            Instruction::MulInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(x * y));
            }
            Instruction::DivInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                assert!(y != 0, "division by zero");
                self.stack.push(Value::Int(x / y));
            }
            Instruction::Mod => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                assert!(y != 0, "mod by zero");
                self.stack.push(Value::Int(x % y));
            }
            Instruction::AddFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Float(x + y));
            }
            Instruction::SubFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Float(x - y));
            }
            Instruction::MulFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Float(x * y));
            }
            Instruction::DivFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                assert!(!(y == 0.0), "division by zero");
                self.stack.push(Value::Float(x / y));
            }
            Instruction::Append(val) => {
                let arr = self.stack.pop().expect("stack underflow");

                if let Value::Ref(h) = arr {
                    if let Object::Slice(ref mut vs) = self.gc.get_mut(h) {
                        vs.push(val);
                    } else {
                        panic!("append to non-slice");
                    }
                    self.stack.push(Value::Ref(h));
                } else {
                    panic!("append expects Ref");
                }
            }
            Instruction::GetByIndex(i) => {
                let arr = self.stack.pop().expect("stack underflow");

                if let Value::Ref(h) = arr {
                    if let Object::Slice(vs) = self.gc.get(h) {
                        assert!(!(i < 0 || (i as usize) >= vs.len()), "index out of range");
                        self.stack.push(vs[i as usize].clone());
                    } else {
                        panic!("getByIndex on non-slice");
                    }
                } else {
                    panic!("getByIndex expects Ref");
                }
            }
            Instruction::SetByIndex(i) => {
                let array = self.stack.pop().expect("stack underflow");
                let value = self.stack.pop().expect("stack underflow");

                if let Value::Ref(h) = array {
                    if let Object::Slice(vs) = self.gc.get_mut(h) {
                        assert!(!(i < 0 || (i as usize) >= vs.len()), "index out of range");
                        vs[i as usize] = value;
                    } else {
                        panic!("setByIndex on non-slice");
                    }

                    self.stack.push(Value::Ref(h));
                } else {
                    panic!("setByIndex expects Ref");
                }
            }
            Instruction::Equal => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let eq = self.is_equal_values(a, b);

                self.stack.push(Value::Int(i64::from(eq)));
            }
            Instruction::NotEqual => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let eq = self.is_equal_values(a, b);

                self.stack.push(Value::Int(i64::from(!eq)));
            }
            Instruction::LessInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(i64::from(x < y)));
            }
            Instruction::LessEqualInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(i64::from(x <= y)));
            }
            Instruction::GreaterInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(i64::from(x > y)));
            }
            Instruction::GreaterEqualInt => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_int(&a), VM::get_int(&b));

                self.stack.push(Value::Int(i64::from(x >= y)));
            }
            Instruction::LessFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Int(i64::from(x < y)));
            }
            Instruction::LessEqualFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Int(i64::from(x <= y)));
            }
            Instruction::GreaterFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Int(i64::from(x > y)));
            }
            Instruction::GreaterEqualFloat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");
                let (x, y) = (VM::get_float(&a), VM::get_float(&b));

                self.stack.push(Value::Int(i64::from(x >= y)));
            }
            Instruction::Concat => {
                let b = self.stack.pop().expect("stack underflow");
                let a = self.stack.pop().expect("stack underflow");

                let s1 = self.get_string(a);
                let s2 = self.get_string(b);

                let r = s1 + s2;
                let h = self.gc.allocate(Object::String(r));

                self.stack.push(Value::Ref(h));
                self.maybe_run_gc();
            }
            Instruction::Jump(address) => {
                assert!(address < self.input.len(), "jump out of range");
                self.program_counter = address;

                return;
            }
            Instruction::JumpIfTrue(addr) => {
                let c = self.stack.pop().expect("stack underflow");

                if VM::is_true(c) {
                    assert!(addr < self.input.len(), "jump out of range");
                    self.program_counter = addr;
                    return;
                }
            }
            Instruction::JumpIfFalse(addr) => {
                let c = self.stack.pop().expect("stack underflow");
                if !VM::is_true(c) {
                    assert!(addr < self.input.len(), "jump out of range");
                    self.program_counter = addr;
                    return;
                }
            }
            Instruction::Call(name) => {
                if let Some(&addr) = self.functions.get(&name) {
                    self.environments_stack.push(HashMap::new());
                    self.call_stack.push(self.program_counter + 1);
                    self.program_counter = addr;
                    return;
                }
                panic!("call to undefined function {name}");
            }
            Instruction::Return => {
                self.environments_stack.pop();
                if let Some(addr) = self.call_stack.pop() {
                    self.program_counter = addr;
                    return;
                }
                return;
            }
            Instruction::Struct(_) | Instruction::EndStruct => {
                panic!("struct definition in main block");
            }
            Instruction::NewStruct(sname) => {
                if let Some(fields) = self.structures.get(&sname) {
                    let mut map = HashMap::new();
                    for (k, v) in fields {
                        map.insert(k.clone(), v.clone());
                    }
                    let h = self.gc.allocate(Object::Struct {
                        name: sname.clone(),
                        fields: map,
                    });
                    self.stack.push(Value::Ref(h));
                    self.maybe_run_gc();
                } else {
                    panic!("unknown struct {sname}");
                }
            }
            Instruction::Field(_, _) => {
                panic!("Field encountered in main block");
            }
            Instruction::SetField(fname) => {
                let r#struct = self.stack.pop().expect("stack underflow");
                let value = self.stack.pop().expect("stack underflow");
                if let Value::Ref(heap) = r#struct {
                    if let Object::Struct { fields, .. } = self.gc.get_mut(heap) {
                        if fields.contains_key(&fname) {
                            fields.insert(fname.clone(), value);
                        } else {
                            panic!("no such field {fname}");
                        }
                    } else {
                        panic!("setField on non-struct");
                    }
                    self.stack.push(Value::Ref(heap));
                } else {
                    panic!("setField expects struct ref");
                }
            }
            Instruction::GetField(fname) => {
                let s = self.stack.pop().expect("stack underflow");
                if let Value::Ref(h) = s {
                    if let Object::Struct { fields, .. } = self.gc.get(h) {
                        if let Some(val) = fields.get(&fname) {
                            self.stack.push(val.clone());
                        } else {
                            panic!("no such field {fname}");
                        }
                    } else {
                        panic!("getField on non-struct");
                    }
                } else {
                    panic!("getField expects struct ref");
                }
            }
            Instruction::Print => {
                let top = self.stack.last().expect("stack underflow");
                self.print_value(top);
            }
            Instruction::Println => {
                let top = self.stack.last().expect("stack underflow");
                self.print_value(top);
                println!();
            }
            Instruction::Func(_) | Instruction::EndFunc => {
                panic!("function definition in main block");
            }
            Instruction::Halt => {
                return;
            }
        }

        self.program_counter += 1;
    }

    pub fn preprocess_bytecode(&mut self) {
        let mut i = 0;

        while i < self.input.len() {
            match &self.input[i] {
                Instruction::Func(name) => {
                    let start = i + 1;
                    let mut end = None;
                    let mut j = start;

                    while j < self.input.len() {
                        if let Instruction::EndFunc = self.input[j] {
                            end = Some(j);
                            break;
                        }
                        j += 1;
                    }

                    if let Some(end) = end {
                        self.functions.insert(name.clone(), start);
                        i = end + 1;

                        continue;
                    }
                    panic!("Func without EndFunc");
                }
                Instruction::Struct(struct_name) => {
                    let mut fields = HashMap::new();
                    i += 1;

                    while i < self.input.len() {
                        match &self.input[i] {
                            Instruction::Field(key, value) => {
                                fields.insert(key.clone(), value.clone());
                            }
                            Instruction::EndStruct => {
                                break;
                            }
                            _ => panic!("unexpected token in struct"),
                        }

                        i += 1;
                    }

                    assert!(i < self.input.len(), "Struct without EndStruct");
                    self.structures.insert(struct_name.clone(), fields);
                }
                _ => {}
            }

            i += 1;
        }
    }

    fn lookup_variable(&self, name: &EcoString) -> Option<&Value> {
        for environment in self.environments_stack.iter().rev() {
            if let Some(value) = environment.get(name) {
                return Some(value);
            }
        }

        self.global_variables.get(name)
    }

    fn maybe_run_gc(&mut self) {
        if self.gc.alloc_count > self.gc.threshold {
            self.gc.collect_garbage(
                &self.stack,
                &self.environments_stack,
                &self.global_variables,
            );
        }
    }

    fn get_int(value: &Value) -> i64 {
        match value {
            Value::Int(int) => *int,
            Value::Ref(_) => panic!("expected int, found Ref"),
            _ => panic!("expected int"),
        }
    }

    fn get_float(value: &Value) -> f64 {
        match value {
            Value::Float(float) => *float,
            Value::Ref(_) => panic!("expected float, found Ref"),
            _ => panic!("expected float"),
        }
    }

    fn get_string(&self, value: Value) -> EcoString {
        match value {
            Value::String(string) => string,
            Value::Ref(handle) => match self.gc.get(handle) {
                Object::String(string) => string.clone(),
                _ => panic!("expected string object"),
            },
            _ => panic!("expected string"),
        }
    }

    fn is_equal_values(&self, value1: Value, value2: Value) -> bool {
        match (value1, value2) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Ref(r1), Value::Ref(r2)) => {
                let o1 = self.gc.get(r1);
                let o2 = self.gc.get(r2);

                match (o1, o2) {
                    (Object::String(ss1), Object::String(ss2)) => ss1 == ss2,
                    (Object::Slice(v1), Object::Slice(v2)) => v1 == v2,
                    (
                        Object::Struct {
                            name: n1,
                            fields: f1,
                        },
                        Object::Struct {
                            name: n2,
                            fields: f2,
                        },
                    ) => n1 == n2 && f1 == f2,
                    _ => panic!("Non comparable"),
                }
            }
            _ => false,
        }
    }

    fn is_true(value: Value) -> bool {
        match value {
            Value::Int(int) => int != 0,
            Value::Float(float) => float != 0.0,
            Value::Char(char) => char != '\0',
            Value::String(string) => !string.is_empty(),
            Value::Slice(slice) => !slice.is_empty(),
            Value::Struct { .. } | Value::Ref(_) => true,
            Value::Nil => false,
        }
    }

    fn print_value(&self, value: &Value) {
        match value {
            Value::Int(int) => print!("{int}"),
            Value::Float(float) => print!("{float}"),
            Value::Char(char) => print!("{char}"),
            Value::String(string) => print!("{string}"),
            Value::Slice(arr) => {
                print!("[");

                for (i, value) in arr.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }

                    self.print_value(value);
                }

                print!("]");
            }
            Value::Struct { name, fields } => {
                print!("Struct {name} {{");
                let mut first = true;
                for (name, val) in fields {
                    if !first {
                        print!(", ");
                    }
                    print!("{name}: ");
                    self.print_value(val);
                    first = false;
                }
                print!("}}");
            }
            Value::Ref(heap) => {
                let object = self.gc.get(*heap);

                match object {
                    Object::String(string) => print!("{string}"),
                    Object::Slice(array) => {
                        print!("[");
                        for (i, value) in array.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }

                            self.print_value(value);
                        }

                        print!("]");
                    }
                    Object::Struct { name, fields } => {
                        print!("Struct {name} {{");
                        let mut first = true;
                        for (name, val) in fields {
                            if !first {
                                print!(", ");
                            }
                            print!("{name}: ");
                            self.print_value(val);
                            first = false;
                        }
                        print!("}}");
                    }
                }
            }
            Value::Nil => print!("nil"),
        }
    }

    // ========================
    //     HOTSWAP - МЕТОД
    // ========================
    /// (1) Ищет Func(name) ... `EndFunc` в новом фрагменте
    /// (2) Смещает Jump/JumpIfTrue/JumpIfFalse на offset = текущая длина self.input
    /// (3) Добавляет в self.input: Func(name), [тело], `EndFunc`
    /// (4) Обновляет functions[name] на начало вставленного тела
    pub fn hotswap_function(&mut self, new_code: &[Instruction]) {
        let (function_name, body) = VM::extract_func_block(new_code);
        let offset = self.input.len();

        let body_fixed = VM::adjust_jumps(body, offset);

        self.input.push(Instruction::Func(function_name.clone()));
        let start_addr = self.input.len();

        for instr in body_fixed {
            self.input.push(instr);
        }

        self.input.push(Instruction::EndFunc);
        self.functions.insert(function_name, start_addr);
    }

    fn extract_func_block(code: &[Instruction]) -> (EcoString, Vec<Instruction>) {
        let mut name = EcoString::new();
        let mut start = None;
        let mut end = None;

        for (i, instruction) in code.iter().enumerate() {
            match instruction {
                Instruction::Func(n) => {
                    name = n.clone();
                    start = Some(i);
                }
                Instruction::EndFunc => {
                    if start.is_some() && end.is_none() {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let start = start.expect("No Func(...) in new_code");
        let end = end.expect("No EndFunc after Func(...)");
        let body = code[start + 1..end].to_vec();

        (name, body)
    }

    fn adjust_jumps(body: Vec<Instruction>, offset: usize) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(body.len());

        for instruction in body {
            let x = match instruction {
                Instruction::Jump(index) => Instruction::Jump(index + offset),
                Instruction::JumpIfTrue(index) => Instruction::JumpIfTrue(index + offset),
                Instruction::JumpIfFalse(index) => Instruction::JumpIfFalse(index + offset),
                other => other,
            };

            result.push(x);
        }

        result
    }
}
