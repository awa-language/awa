use std::collections::HashMap;

use ecow::EcoString;
pub mod instruction;

mod gc;
#[cfg(test)]
pub mod tests;

use gc::{Object, GC};
use instruction::{Bytecode, Instruction, Value};

use crate::optimizer::Optimizer;

pub struct VM {
    pub(crate) input: Bytecode,
    pub(crate) program_counter: usize,
    pub(crate) stack: Vec<Value>,

    /// Environments stack for local variables (each Func call -> push, Return -> pop).
    pub(crate) environments_stack: Vec<HashMap<EcoString, Value>>,

    pub(crate) structures: HashMap<EcoString, HashMap<EcoString, Value>>,
    pub(crate) functions: HashMap<EcoString, usize>,
    pub(crate) call_stack: Vec<usize>,

    pub(crate) gc: GC,

    backup_state: Option<State>,
    execution_stats: ExecutionStats,
    optimization_threshold: u32,

    should_perform_optimizations: bool,
}

#[derive(Debug)]
struct State {
    stack: Vec<Value>,
    program_counter: usize,
}

#[derive(Default)]
struct ExecutionStats {
    function_executions: HashMap<EcoString, u32>,
    loop_iterations: HashMap<usize, u32>,
    optimized_functions: HashMap<EcoString, u32>,
    loop_last_optimization: HashMap<usize, u32>,
    current_execution_time: u32,
}

impl ExecutionStats {
    fn new(functions_count: usize) -> Self {
        Self {
            function_executions: HashMap::with_capacity(functions_count),
            loop_iterations: HashMap::with_capacity(functions_count), // whatever
            optimized_functions: HashMap::with_capacity(functions_count),
            loop_last_optimization: HashMap::with_capacity(functions_count), // whatever
            current_execution_time: 0,
        }
    }

    fn record_function_execution(&mut self, name: &EcoString) {
        *self.function_executions.entry(name.clone()).or_insert(0) += 1;
    }

    fn record_loop_iteration(&mut self, start_pc: usize) {
        *self.loop_iterations.entry(start_pc).or_insert(0) += 1;
    }

    fn should_optimize_function(&self, name: &EcoString, threshold: u32) -> bool {
        if self.optimized_functions.contains_key(name) {
            return false;
        }

        self.function_executions
            .get(name)
            .is_some_and(|&count| count > threshold)
    }

    fn should_optimize_loop(&self, start_pc: usize, threshold: u32) -> bool {
        if self.loop_last_optimization.contains_key(&start_pc) {
            return false;
        }

        self.loop_iterations
            .get(&start_pc)
            .is_some_and(|&count| count > threshold)
    }

    fn record_function_optimization(&mut self, name: &EcoString) {
        self.optimized_functions
            .insert(name.clone(), self.current_execution_time);
    }

    fn record_loop_optimization(&mut self, start_pc: usize) {
        self.loop_last_optimization
            .insert(start_pc, self.current_execution_time);
    }
}

#[derive(Debug, Clone)]
pub enum RunCommunication {
    RequireHotswap(EcoString),
    Finished,
}

impl VM {
    /// Initializes new VM
    ///
    /// # Panics
    ///
    /// Will panic if the provided bytecode does not contain `main()` function.
    #[must_use]
    pub fn new(input: Vec<Instruction>, should_perform_optimizations: bool) -> Self {
        let mut vm = Self {
            input,
            program_counter: 0,
            stack: Vec::with_capacity(100_000),
            environments_stack: Vec::with_capacity(100_000),
            structures: HashMap::with_capacity(10),
            functions: HashMap::with_capacity(10),
            call_stack: Vec::with_capacity(100_000),
            gc: GC::new(),
            backup_state: None,
            execution_stats: ExecutionStats::new(0),
            optimization_threshold: 10000,
            should_perform_optimizations,
        };

        vm.environments_stack.push(HashMap::with_capacity(50));

        let functions_count = vm.preprocess_bytecode();
        vm.execution_stats = ExecutionStats::new(functions_count);

        if let Some(&main_address) = vm.functions.get("main") {
            vm.program_counter = main_address;

            vm.backup_state = Some(State {
                stack: vm.stack.clone(),
                program_counter: vm.program_counter,
            });
        } else {
            panic!("cannot find function `main()` in provided code");
        }

        vm
    }

    /// Runs one current instruction in the VM
    ///
    /// # Panics
    ///
    /// Will panic if cannot recover from user code error, or in case other
    /// interpreter parts do not function as expected.
    #[must_use]
    pub fn run(&mut self) -> Option<RunCommunication> {
        if self.program_counter >= self.input.len() {
            return Some(RunCommunication::Finished);
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
                let handle = self.gc.allocate(Object::Array(array));

                self.stack.push(Value::Ref(handle));
                self.maybe_run_gc();
            }
            Instruction::StoreInMap(name) => {
                let value = self.stack.pop().expect("stack underflow");

                if let Some(environment) = self.environments_stack.last_mut() {
                    environment.insert(name, value);
                } else {
                    panic!("no environment available to store the variable");
                }
            }
            Instruction::LoadToStack(name) => {
                let variable_value = self.lookup_variable(&name);
                self.stack.push(variable_value.clone());
            }
            Instruction::AddInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(lhs + rhs));
            }
            Instruction::SubInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(lhs - rhs));
            }
            Instruction::MulInt | Instruction::And => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(lhs * rhs));
            }
            Instruction::DivInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                if rhs == 0 {
                    return Some(RunCommunication::RequireHotswap(
                        self.perform_backoff("integer division by zero"),
                    ));
                }

                self.stack.push(Value::Int(lhs / rhs));
            }
            Instruction::Mod => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                if rhs == 0 {
                    return Some(RunCommunication::RequireHotswap(
                        self.perform_backoff("modulo by zero"),
                    ));
                }

                self.stack.push(Value::Int(lhs % rhs));
            }
            Instruction::AddFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Float(lhs + rhs));
            }
            Instruction::SubFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Float(lhs - rhs));
            }
            Instruction::MulFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Float(lhs * rhs));
            }
            Instruction::DivFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                if rhs == 0.0 {
                    return Some(RunCommunication::RequireHotswap(
                        self.perform_backoff("floating point division by zero"),
                    ));
                }

                self.stack.push(Value::Float(lhs / rhs));
            }
            Instruction::Append => {
                let value = self.stack.pop().expect("stack underflow");
                let array = self.stack.pop().expect("stack underflow");

                if let Value::Ref(handle) = array {
                    if let Object::Array(ref mut slice) = self.gc.get_mut(handle) {
                        slice.push(value);
                    } else {
                        panic!("Append to non-slice");
                    }
                    self.stack.push(Value::Ref(handle));
                } else {
                    panic!("Append expects Ref");
                }
            }
            Instruction::Pop => {
                let array = self.stack.pop().expect("stack underflow");

                if let Value::Ref(handle) = array {
                    if let Object::Array(ref mut slice) = self.gc.get_mut(handle) {
                        slice.pop();
                    } else {
                        panic!("Append to non-slice");
                    }
                    self.stack.push(Value::Ref(handle));
                } else {
                    panic!("Append expects Ref");
                }
            }
            Instruction::GetByIndex => {
                let index = self.stack.pop().expect("stack underflow");
                let index = VM::get_int(&index);
                let array = self.stack.pop().expect("stack underflow");

                if let Value::Ref(handle) = array {
                    let Object::Array(slice) = self.gc.get(handle) else {
                        panic!("GetByIndex on non-slice");
                    };

                    if index < 0 || (usize::try_from(index).unwrap()) >= slice.len() {
                        return Some(RunCommunication::RequireHotswap(
                            self.perform_backoff("getting from array by index out of range"),
                        ));
                    }

                    self.stack
                        .push(slice[usize::try_from(index).unwrap()].clone());
                } else {
                    panic!("GetByIndex expects Ref");
                }
            }
            Instruction::SetByIndex => {
                let index = self.stack.pop().expect("stack underflow");
                let index = VM::get_int(&index);
                let value = self.stack.pop().expect("stack underflow");
                let array = self.stack.pop().expect("stack underflow");

                if let Value::Ref(handle) = array {
                    let Object::Array(slice) = self.gc.get_mut(handle) else {
                        panic!("SetByIndex on non-slice");
                    };

                    if index < 0 || (usize::try_from(index).unwrap()) >= slice.len() {
                        return Some(RunCommunication::RequireHotswap(
                            self.perform_backoff("setting array value by index out of range"),
                        ));
                    }

                    slice[usize::try_from(index).unwrap()] = value;

                    self.stack.push(Value::Ref(handle));
                } else {
                    panic!("SetByIndex expects Ref");
                }
            }
            Instruction::Equal => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let equal = self.is_equal_values(lhs, rhs);

                self.stack.push(Value::Int(i64::from(equal)));
            }
            Instruction::NotEqual => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let equal = self.is_equal_values(lhs, rhs);

                self.stack.push(Value::Int(i64::from(!equal)));
            }
            Instruction::Or => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(lhs | rhs));
            }
            Instruction::LessInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(i64::from(lhs < rhs)));
            }
            Instruction::LessEqualInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(i64::from(lhs <= rhs)));
            }
            Instruction::GreaterInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(i64::from(lhs > rhs)));
            }
            Instruction::GreaterEqualInt => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_int(&lhs), VM::get_int(&rhs));

                self.stack.push(Value::Int(i64::from(lhs >= rhs)));
            }
            Instruction::LessFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Int(i64::from(lhs < rhs)));
            }
            Instruction::LessEqualFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Int(i64::from(lhs <= rhs)));
            }
            Instruction::GreaterFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Int(i64::from(lhs > rhs)));
            }
            Instruction::GreaterEqualFloat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");
                let (lhs, rhs) = (VM::get_float(&lhs), VM::get_float(&rhs));

                self.stack.push(Value::Int(i64::from(lhs >= rhs)));
            }
            Instruction::Concat => {
                let rhs = self.stack.pop().expect("stack underflow");
                let lhs = self.stack.pop().expect("stack underflow");

                let s1 = self.get_string(lhs);
                let s2 = self.get_string(rhs);

                let result = s1 + s2;
                let handle = self.gc.allocate(Object::String(result));

                self.stack.push(Value::Ref(handle));
                self.maybe_run_gc();
            }
            Instruction::Jump(address) => {
                if self.should_perform_optimizations && address <= self.program_counter {
                    self.execution_stats.record_loop_iteration(address);

                    if self
                        .execution_stats
                        .should_optimize_loop(address, self.optimization_threshold)
                    {
                        self.optimize_loop(address, self.program_counter);
                        self.execution_stats.record_loop_optimization(address);
                    }
                }

                assert!(address < self.input.len(), "jump out of range");
                self.program_counter = address;

                return None;
            }
            Instruction::JumpIfTrue(address) => {
                let condition = self.stack.pop().expect("stack underflow");

                if VM::is_true(condition) {
                    assert!(address < self.input.len(), "jump out of range");
                    self.program_counter = address;

                    return None;
                }
            }
            Instruction::JumpIfFalse(address) => {
                let condition = self.stack.pop().expect("stack underflow");
                if !VM::is_true(condition) {
                    assert!(address < self.input.len(), "jump out of range");
                    self.program_counter = address;

                    return None;
                }
            }
            Instruction::Call(name) => {
                if self.should_perform_optimizations {
                    self.execution_stats.record_function_execution(&name);

                    if self
                        .execution_stats
                        .should_optimize_function(&name, self.optimization_threshold)
                    {
                        self.optimize_function(&name);
                        self.execution_stats.record_function_optimization(&name);
                    }
                }

                if let Some(&address) = self.functions.get(&name) {
                    self.backup_state = Some(State {
                        stack: self.stack.clone(),
                        program_counter: self.program_counter,
                    });

                    self.environments_stack.push(HashMap::with_capacity(10));
                    self.call_stack.push(self.program_counter + 1);
                    self.program_counter = address;

                    return None;
                }

                panic!("Call to undefined function `{name}`");
            }
            Instruction::Return => {
                self.environments_stack.pop();

                if let Some(address) = self.call_stack.pop() {
                    self.program_counter = address;
                    self.backup_state = None;

                    return None;
                }
                return None;
            }
            Instruction::Struct(_) | Instruction::EndStruct => {
                panic!("Struct definition in `main()` body");
            }
            Instruction::NewStruct(struct_name) => {
                if let Some(fields) = self.structures.get(&struct_name) {
                    let mut map = HashMap::with_capacity(fields.len());

                    for (key, value) in fields {
                        map.insert(key.clone(), value.clone());
                    }

                    let handle = self.gc.allocate(Object::Struct {
                        name: struct_name.clone(),
                        fields: map,
                    });

                    self.stack.push(Value::Ref(handle));
                    self.maybe_run_gc();
                } else {
                    panic!("unknown struct `{struct_name}`");
                }
            }
            Instruction::Field(_, _) => {
                panic!("Field encountered in `main()` body");
            }
            Instruction::SetField(field_name) => {
                let value = self.stack.pop().expect("stack underflow");
                let struct_value = self.stack.pop().expect("stack underflow");
                if let Value::Ref(handle) = struct_value {
                    if let Object::Struct { fields, .. } = self.gc.get_mut(handle) {
                        if fields.contains_key(&field_name) {
                            fields.insert(field_name.clone(), value);
                        } else {
                            panic!("no such field: `{field_name}`");
                        }
                    } else {
                        panic!("SetField on non-struct");
                    }
                    self.stack.push(Value::Ref(handle));
                } else {
                    panic!("SetField expects struct ref");
                }
            }
            Instruction::GetField(field_name) => {
                let struct_value = self.stack.pop().expect("stack underflow");
                match struct_value {
                    Value::Ref(handle) => {
                        if let Object::Struct { fields, .. } = self.gc.get(handle) {
                            if let Some(value) = fields.get(&field_name) {
                                self.stack.push(value.clone());
                            } else {
                                panic!("no such field: `{field_name}`");
                            }
                        } else {
                            panic!("GetField on non-struct Ref");
                        }
                    }
                    Value::Struct { name, .. } => {
                        if let Some(fields) = self.structures.get(&name) {
                            if let Some(value) = fields.get(&field_name) {
                                self.stack.push(value.clone());
                            } else {
                                panic!("no such field: `{field_name}`");
                            }
                        } else {
                            panic!("struct `{name}` not found in structures map");
                        }
                    }
                    _ => {
                        panic!("GetField expects a Struct or Struct Ref");
                    }
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
                return Some(RunCommunication::Finished);
            }
            Instruction::Backoff(reason) => {
                return Some(RunCommunication::RequireHotswap(
                    self.perform_backoff(&reason),
                ));
            }
        }

        self.program_counter += 1;

        None
    }

    fn preprocess_bytecode(&mut self) -> usize {
        let mut i = 0;

        let mut functions_count = 0;

        while i < self.input.len() {
            match &self.input[i] {
                Instruction::Func(name) => {
                    functions_count += 1;

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
                    let mut fields = HashMap::with_capacity(10);
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

        functions_count
    }

    fn optimize_function(&mut self, name: &EcoString) {
        if let Some(&start) = self.functions.get(name) {
            let mut end = start;
            while end < self.input.len() {
                if matches!(self.input[end], Instruction::EndFunc) {
                    break;
                }
                end += 1;
            }

            let code_to_optimize = self.input[start..end].to_vec();
            let optimized_code = Optimizer::optimize_function(code_to_optimize.clone(), start);

            self.replace_code_region(start, end - 1, optimized_code);
        }
    }

    fn optimize_loop(&mut self, start: usize, end: usize) {
        let mut func_start = start;
        while func_start > 0 {
            if let Instruction::Func(_) = &self.input[func_start - 1] {
                break;
            }
            func_start -= 1;
        }

        let mut func_end = end;
        while func_end < self.input.len() {
            if let Instruction::EndFunc = &self.input[func_end] {
                break;
            }
            func_end += 1;
        }

        let function_code = self.input[func_start..func_end].to_vec();
        let loop_start = start - func_start;
        let loop_end = end - func_start;

        let optimized_loop =
            Optimizer::optimize_loop(function_code, loop_start, loop_end, func_start);
        self.replace_code_region(start, end, optimized_loop);
    }

    fn replace_code_region(&mut self, start: usize, end: usize, new_code: Vec<Instruction>) {
        let old_size = end - start + 1;
        let new_size = new_code.len();
        let size_diff = isize::try_from(new_size).unwrap() - isize::try_from(old_size).unwrap();

        self.input.splice(start..=end, new_code);

        if self.program_counter > start {
            self.program_counter =
                usize::try_from(isize::try_from(self.program_counter).unwrap() + size_diff)
                    .unwrap();
        }

        for address in &mut self.call_stack {
            if *address > start {
                *address = usize::try_from(isize::try_from(*address).unwrap() + size_diff).unwrap();
            }
        }

        for i in 0..self.input.len() {
            match &mut self.input[i] {
                Instruction::Jump(target)
                | Instruction::JumpIfTrue(target)
                | Instruction::JumpIfFalse(target) => {
                    if *target > end {
                        *target =
                            usize::try_from(isize::try_from(*target).unwrap() + size_diff).unwrap();
                    }
                }
                _ => {}
            }
        }

        for address in self.functions.values_mut() {
            if *address > start {
                *address = usize::try_from(isize::try_from(*address).unwrap() + size_diff).unwrap();
            }
        }
    }

    fn perform_backoff(&mut self, reason: &str) -> EcoString {
        match &self.backup_state {
            Some(backup_state) => {
                let call_instruction = match self.call_stack.pop() {
                    Some(address) => self.input[address - 1].clone(),
                    None => unreachable!(),
                };
                let Instruction::Call(name) = call_instruction else {
                    unreachable!();
                };

                self.program_counter = backup_state.program_counter;
                self.stack.clone_from(&backup_state.stack);
                let _ = self.environments_stack.pop();

                self.backup_state = None;

                name
            }
            None => panic!("cannot recover from: {reason}"),
        }
    }

    fn lookup_variable(&self, name: &EcoString) -> &Value {
        for environment in self.environments_stack.iter().rev() {
            if let Some(value) = environment.get(name) {
                return value;
            }
        }
        panic!("stack underflow")
    }

    fn maybe_run_gc(&mut self) {
        if self.gc.alloc_count > self.gc.threshold {
            self.gc
                .collect_garbage(&mut self.stack, &mut self.environments_stack);
        }
    }

    fn get_int(value: &Value) -> i64 {
        match value {
            Value::Int(int) => *int,
            Value::Ref(_) => panic!("expected Int, found Ref"),
            _ => panic!("expected Int"),
        }
    }

    fn get_float(value: &Value) -> f64 {
        match value {
            Value::Float(float) => *float,
            Value::Ref(_) => panic!("expected Float, found Ref"),
            _ => panic!("expected Float"),
        }
    }

    fn get_string(&self, value: Value) -> EcoString {
        match value {
            Value::String(string) => string,
            Value::Ref(handle) => match self.gc.get(handle) {
                Object::String(string) => string.clone(),
                _ => panic!("expected String object"),
            },
            _ => panic!("expected String"),
        }
    }

    fn is_equal_values(&self, lhs: Value, rhs: Value) -> bool {
        match (lhs, rhs) {
            (Value::Int(lhs), Value::Int(rhs)) => lhs == rhs,
            (Value::Float(lhs), Value::Float(rhs)) => (lhs - rhs).abs() < f64::EPSILON,
            (Value::Char(lhs), Value::Char(rhs)) => lhs == rhs,
            (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
            (Value::Ref(lhs), Value::Ref(rhs)) => {
                let lhs = self.gc.get(lhs);
                let rhs = self.gc.get(rhs);

                match (lhs, rhs) {
                    (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
                    (Object::Array(lhs), Object::Array(rhs)) => lhs == rhs,
                    (
                        Object::Struct {
                            name: name1,
                            fields: fields1,
                        },
                        Object::Struct {
                            name: name2,
                            fields: fields2,
                        },
                    ) => name1 == name2 && fields1 == fields2,
                    _ => panic!("non comparable: {lhs:?}, {rhs:?}"),
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
            Value::Ref(handle) => {
                let object = self.gc.get(*handle);

                match object {
                    Object::String(string) => print!("{string}"),
                    Object::Array(array) => {
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

    /// Perform function hotswap
    ///
    /// # Panics
    ///
    /// Will panic if the function to be swapped out is not present in the code
    ///
    /// 1. Finds `Func(name)` ... `EndFunc` in the new fragment.
    /// 2. Adjusts `Jump`/`JumpIfTrue`/`JumpIfFalse` by an offset equal to the current length of `self.input`.
    /// 3. Adds to `self.input`: `Func(name)`, [body], `EndFunc`.
    /// 4. Updates `functions[name]` to point to the start of the inserted body.
    pub fn hotswap_function(&mut self, new_code: &[Instruction]) {
        let (function_name, body) = VM::extract_func_block(new_code);
        let offset = self.input.len();
        let binding = self.functions.clone();
        let function_start = binding.get(&function_name).expect("no function found");
        let mut function_end = *function_start;

        while function_end > self.input.len() {
            match self.input.get(function_end) {
                Some(Instruction::EndFunc) => {
                    break;
                }
                _ => {
                    function_end += 1;
                }
            }
        }

        let body_fixed = VM::adjust_jumps(body, offset);

        self.input.push(Instruction::Func(function_name.clone()));
        let start_address = self.input.len();

        for instruction in body_fixed {
            self.input.push(instruction);
        }

        self.input.push(Instruction::EndFunc);
        self.functions.insert(function_name.clone(), start_address);
        self.execution_stats
            .optimized_functions
            .remove(&function_name);
        self.execution_stats
            .loop_last_optimization
            .retain(|&address, _| address < function_end && address > *function_start);
    }

    fn extract_func_block(code: &[Instruction]) -> (EcoString, Vec<Instruction>) {
        let mut func_name = EcoString::new();
        let mut start = None;
        let mut end = None;

        for (i, instruction) in code.iter().enumerate() {
            match instruction {
                Instruction::Func(name) => {
                    func_name = name.clone();
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

        let start = start.expect("no Func(...) in new_code");
        let end = end.expect("no EndFunc after Func(...)");

        let body = code[start + 1..end].to_vec();

        (func_name, body)
    }

    fn adjust_jumps(body: Vec<Instruction>, offset: usize) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(body.len());

        for instruction in body {
            let instruction = match instruction {
                Instruction::Jump(index) => Instruction::Jump(index + offset),
                Instruction::JumpIfTrue(index) => Instruction::JumpIfTrue(index + offset),
                Instruction::JumpIfFalse(index) => Instruction::JumpIfFalse(index + offset),
                other => other,
            };

            result.push(instruction);
        }

        result
    }
}
