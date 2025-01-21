use std::collections::HashMap;

use ecow::EcoString;
pub mod instruction;

mod gc;
#[cfg(test)]
pub mod tests;

use gc::{Object, GC};
use instruction::{Bytecode, Handle, Instruction, Value};

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
    pub fn new(input: Bytecode) -> Self {
        Self {
            input,
            program_counter: 0,
            stack: Vec::new(),
            global_variables: HashMap::new(),
            environments_stack: Vec::new(),
            structures: HashMap::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
            gc: GC::new(),
        }
    }

    pub fn run(&mut self) {
        self.preprocess();

        if let Some(&main_address) = self.functions.get("main") {
            self.program_counter = main_address;
        } else {
            panic!("cannot find function `main()`");
        }

        loop {
            if self.program_counter >= self.input.len() {
                break;
            }
            let instr = self.input[self.program_counter].clone();

            match instr {
                Instruction::PushInt(i) => {
                    self.stack.push(Value::Int(i));
                }
                Instruction::PushFloat(f) => {
                    self.stack.push(Value::Float(f));
                }
                Instruction::PushChar(c) => {
                    self.stack.push(Value::Char(c));
                }
                Instruction::PushString(s) => {
                    let h = self.gc.allocate(Object::String(s));
                    self.stack.push(Value::Ref(h));
                    self.maybe_run_gc();
                }
                Instruction::PushSlice(v) => {
                    let h = self.gc.allocate(Object::Slice(v));
                    self.stack.push(Value::Ref(h));
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
                    if let Some(val) = self.lookup_variable(&name) {
                        self.stack.push(val.clone());
                    } else {
                        panic!("undefined variable: {}", name);
                    }
                }

                Instruction::AddInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(x + y));
                }
                Instruction::SubInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(x - y));
                }
                Instruction::MulInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(x * y));
                }
                Instruction::DivInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    if y == 0 {
                        panic!("division by zero");
                    }
                    self.stack.push(Value::Int(x / y));
                }
                Instruction::Mod => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    if y == 0 {
                        panic!("mod by zero");
                    }
                    self.stack.push(Value::Int(x % y));
                }

                Instruction::AddFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Float(x + y));
                }
                Instruction::SubFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Float(x - y));
                }
                Instruction::MulFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Float(x * y));
                }
                Instruction::DivFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    if y == 0.0 {
                        panic!("division by zero (float)");
                    }
                    self.stack.push(Value::Float(x / y));
                }

                Instruction::Append(val) => {
                    let arr = self.stack.pop().expect("stack underflow");
                    if let Value::Ref(h) = arr {
                        if let Object::Slice(ref mut vs) = self.gc.get_mut(h) {
                            vs.push(val);
                        } else {
                            panic!("Append to non-slice object");
                        }
                        self.stack.push(Value::Ref(h));
                    } else {
                        panic!("Append expects Ref");
                    }
                }
                Instruction::GetByIndex(i) => {
                    let arr = self.stack.pop().expect("stack underflow");
                    if let Value::Ref(h) = arr {
                        if let Object::Slice(vs) = self.gc.get(h) {
                            if i < 0 || (i as usize) >= vs.len() {
                                panic!("index out of range");
                            }
                            self.stack.push(vs[i as usize].clone());
                        } else {
                            panic!("getByIndex on non-slice object");
                        }
                    } else {
                        panic!("getByIndex expects Ref");
                    }
                }
                Instruction::SetByIndex(i) => {
                    let arr = self.stack.pop().expect("stack underflow");
                    let val = self.stack.pop().expect("stack underflow");
                    if let Value::Ref(h) = arr {
                        if let Object::Slice(vs) = self.gc.get_mut(h) {
                            if i < 0 || (i as usize) >= vs.len() {
                                panic!("index out of range");
                            }
                            vs[i as usize] = val;
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
                    let eq = self.eq_values(a, b);
                    self.stack.push(Value::Int(if eq { 1 } else { 0 }));
                }
                Instruction::NotEqual => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let eq = self.eq_values(a, b);
                    self.stack.push(Value::Int(if eq { 0 } else { 1 }));
                }

                Instruction::LessInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(if x < y { 1 } else { 0 }));
                }
                Instruction::LessEqualInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(if x <= y { 1 } else { 0 }));
                }
                Instruction::GreaterInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(if x > y { 1 } else { 0 }));
                }
                Instruction::GreaterEqualInt => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_int(a), self.get_int(b));
                    self.stack.push(Value::Int(if x >= y { 1 } else { 0 }));
                }

                Instruction::LessFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Int(if x < y { 1 } else { 0 }));
                }
                Instruction::LessEqualFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Int(if x <= y { 1 } else { 0 }));
                }
                Instruction::GreaterFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Int(if x > y { 1 } else { 0 }));
                }
                Instruction::GreaterEqualFloat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let (x, y) = (self.get_float(a), self.get_float(b));
                    self.stack.push(Value::Int(if x >= y { 1 } else { 0 }));
                }

                Instruction::Concat => {
                    let b = self.stack.pop().expect("stack underflow");
                    let a = self.stack.pop().expect("stack underflow");
                    let s1 = self.get_string(a);
                    let s2 = self.get_string(b);
                    let r = s1 + s2;
                    let h = self.gc.allocate(Object::String(r.into()));
                    self.stack.push(Value::Ref(h));
                    self.maybe_run_gc();
                }

                Instruction::Jump(addr) => {
                    if addr >= self.input.len() {
                        panic!("jump out of range");
                    }
                    self.program_counter = addr;
                    continue;
                }
                Instruction::JumpIfTrue(addr) => {
                    let c = self.stack.pop().expect("stack underflow");
                    if self.is_true(c) {
                        if addr >= self.input.len() {
                            panic!("jump out of range");
                        }
                        self.program_counter = addr;
                        continue;
                    }
                }
                Instruction::JumpIfFalse(addr) => {
                    let c = self.stack.pop().expect("stack underflow");
                    if !self.is_true(c) {
                        if addr >= self.input.len() {
                            panic!("jump out of range");
                        }
                        self.program_counter = addr;
                        continue;
                    }
                }

                Instruction::Func(_) => {
                    panic!("Func encountered at runtime");
                }
                Instruction::EndFunc => {
                    panic!("EndFunc encountered at runtime");
                }
                Instruction::Call(name) => {
                    if let Some(&addr) = self.functions.get(&name) {
                        self.environments_stack.push(HashMap::new());
                        self.call_stack.push(self.program_counter + 1);
                        self.program_counter = addr;
                        continue;
                    } else {
                        panic!("call to undefined function {}", name);
                    }
                }
                Instruction::Return => {
                    self.environments_stack.pop();

                    if let Some(addr) = self.call_stack.pop() {
                        self.program_counter = addr;
                        continue;
                    } else {
                        break;
                    }
                }

                Instruction::Struct(_) | Instruction::EndStruct => {
                    panic!("struct definition in runtime");
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
                        panic!("unknown struct {}", sname);
                    }
                }
                Instruction::Field(_, _) => {
                    panic!("Field encountered at runtime");
                }
                Instruction::SetField(fname) => {
                    let s = self.stack.pop().expect("stack underflow");
                    let v = self.stack.pop().expect("stack underflow");
                    if let Value::Ref(h) = s {
                        if let Object::Struct { fields, .. } = self.gc.get_mut(h) {
                            if fields.contains_key(&fname) {
                                fields.insert(fname.clone(), v);
                            } else {
                                panic!("no such field {}", fname);
                            }
                        } else {
                            panic!("setField on non-struct object");
                        }
                        self.stack.push(Value::Ref(h));
                    } else {
                        panic!("setField expects a struct ref");
                    }
                }
                Instruction::GetField(fname) => {
                    let s = self.stack.pop().expect("stack underflow");
                    if let Value::Ref(h) = s {
                        if let Object::Struct { fields, .. } = self.gc.get(h) {
                            if let Some(val) = fields.get(&fname) {
                                self.stack.push(val.clone());
                            } else {
                                panic!("no such field {}", fname);
                            }
                        } else {
                            panic!("getField on non-struct");
                        }
                    } else {
                        panic!("getField expects a struct ref");
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

                Instruction::Halt => {
                    break;
                }
            }

            self.program_counter += 1;
        }
    }

    pub fn preprocess(&mut self) {
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
                    if let Some(en) = end {
                        self.functions.insert(name.clone(), start);
                        i = en + 1;
                        continue;
                    } else {
                        panic!("Func without EndFunc");
                    }
                }
                Instruction::Struct(sname) => {
                    let mut fields = HashMap::new();
                    i += 1;
                    while i < self.input.len() {
                        match &self.input[i] {
                            Instruction::Field(k, v) => {
                                fields.insert(k.clone(), v.clone());
                            }
                            Instruction::EndStruct => {
                                break;
                            }
                            _ => {
                                panic!("unexpected token in struct");
                            }
                        }
                        i += 1;
                    }
                    if i >= self.input.len() {
                        panic!("Struct without EndStruct");
                    }
                    self.structures.insert(sname.clone(), fields);
                }
                _ => {}
            }
            i += 1;
        }
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

    fn lookup_variable(&self, name: &EcoString) -> Option<&Value> {
        for env in self.environments_stack.iter().rev() {
            if let Some(val) = env.get(name) {
                return Some(val);
            }
        }
        self.global_variables.get(name)
    }

    fn get_int(&self, v: Value) -> i64 {
        match v {
            Value::Int(i) => i,
            Value::Ref(_) => {
                panic!("expected int, found Ref");
            }
            _ => panic!("expected int"),
        }
    }

    fn get_float(&self, v: Value) -> f64 {
        match v {
            Value::Float(f) => f,
            Value::Ref(_) => panic!("expected float, found Ref"),
            _ => panic!("expected float"),
        }
    }

    fn get_string(&self, v: Value) -> EcoString {
        match v {
            Value::String(s) => s,
            Value::Ref(h) => match self.gc.get(h) {
                Object::String(ss) => ss.clone(),
                _ => panic!("expected string object"),
            },
            _ => panic!("expected string"),
        }
    }

    fn eq_values(&self, a: Value, b: Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Char(x), Value::Char(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Ref(r1), Value::Ref(r2)) => {
                let o1 = self.gc.get(r1);
                let o2 = self.gc.get(r2);
                match (o1, o2) {
                    (Object::String(s1), Object::String(s2)) => s1 == s2,
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
                    _ => todo!(),
                }
            }
            _ => false,
        }
    }

    fn is_true(&self, v: Value) -> bool {
        match v {
            Value::Int(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Char(c) => c != '\0',
            Value::String(s) => !s.is_empty(),
            Value::Slice(slc) => !slc.is_empty(),
            Value::Struct { .. } => true,
            Value::Ref(_) => true,
            Value::Nil => false,
        }
    }

    fn print_value(&self, v: &Value) {
        match v {
            Value::Int(i) => print!("{}", i),
            Value::Float(f) => print!("{}", f),
            Value::Char(c) => print!("{}", c),
            Value::String(s) => print!("{}", s),
            Value::Slice(vs) => {
                print!("[");
                for (idx, val) in vs.iter().enumerate() {
                    if idx > 0 {
                        print!(", ");
                    }
                    self.print_value(val);
                }
                print!("]");
            }
            Value::Struct { name, fields } => {
                print!("Struct {} {{", name);
                let mut first = true;
                for (k, val) in fields {
                    if !first {
                        print!(", ");
                    }
                    print!("{}: ", k);
                    self.print_value(val);
                    first = false;
                }
                print!("}}");
            }
            Value::Ref(h) => {
                let obj = self.gc.get(*h);
                match obj {
                    Object::String(s) => {
                        print!("{}", s);
                    }
                    Object::Slice(vs) => {
                        print!("[");
                        for (idx, val) in vs.iter().enumerate() {
                            if idx > 0 {
                                print!(", ");
                            }
                            self.print_value(val);
                        }
                        print!("]");
                    }
                    Object::Struct { name, fields } => {
                        print!("Struct {} {{", name);
                        let mut first = true;
                        for (k, val) in fields {
                            if !first {
                                print!(", ");
                            }
                            print!("{}: ", k);
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
}
