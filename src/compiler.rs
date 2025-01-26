use std::collections::HashMap;

use ecow::EcoString;

use crate::vm::instruction::{Bytecode, Instruction};

pub struct Compiler {
    // Для оптимизации функции:
    // bytecode = тело функции
    // hot_region = None
    //
    // Для оптимизации цикла:
    // bytecode = вся функция
    // hot_region = Some((start, end)) границы цикла
    bytecode: Bytecode,
    hot_region: Option<(usize, usize)>,
    shift: usize,
}

impl Compiler {
    pub fn optimize_function(function_body: Bytecode, shift: usize) -> Bytecode {
        let mut compiler = Self {
            bytecode: function_body,
            hot_region: None,
            shift,
        };
        compiler.optimize()
    }

    pub fn optimize_loop(
        function_code: Bytecode,
        loop_start: usize,
        loop_end: usize,
        shift: usize,
    ) -> Bytecode {
        let mut compiler = Self {
            bytecode: function_code,
            hot_region: Some((loop_start, loop_end)),
            shift,
        };
        compiler.optimize()
    }

    fn optimize(&mut self) -> Bytecode {
        let mut changed = true;
        let initial_len = self.bytecode.len();
        while changed {
            let len_before = self.bytecode.len();
            self.peephole_optimization();
            self.dead_code_elimination();
            self.removing_empty_conditionals();
            self.constant_folding();
            changed = self.bytecode.len() != len_before;
        }

        match self.hot_region {
            Some((start, end)) => {
                let removed = initial_len - self.bytecode.len();
                let new_end = end - removed;
                let new_start = start.min(new_end);
                self.bytecode[new_start..=new_end].to_vec()
            }
            None => self.bytecode.clone(),
        }
    }

    fn constant_folding(&mut self) {
        let mut i = 0;
        while i < self.bytecode.len() {
            let mut constants = Vec::new();
            let mut j = i;
            let mut can_fold = true;
            let mut last_type = None;

            while j < self.bytecode.len() {
                match &self.bytecode[j] {
                    Instruction::JumpIfTrue(_)
                    | Instruction::JumpIfFalse(_)
                    | Instruction::Jump(_)
                    | Instruction::Func(_)
                    | Instruction::Return
                    | Instruction::EndFunc => break,

                    Instruction::PushInt(n) => {
                        constants.push((n.to_string(), "int"));
                        last_type = Some("int");
                    }
                    Instruction::PushFloat(f) => {
                        constants.push((f.to_string(), "float"));
                        last_type = Some("float");
                    }
                    Instruction::PushString(s) => {
                        constants.push((s.clone().to_string(), "string"));
                        last_type = Some("string");
                    }

                    Instruction::Call(_)
                    | Instruction::GetField(_)
                    | Instruction::GetByIndex
                    | Instruction::PushArray(_)
                    | Instruction::Append
                    | Instruction::LoadToStack(_)
                    | Instruction::SetByIndex => {
                        can_fold = false;
                        break;
                    }

                    Instruction::AddInt
                    | Instruction::SubInt
                    | Instruction::MulInt
                    | Instruction::DivInt
                        if last_type == Some("int") && constants.len() >= 2 =>
                    {
                        let b = constants.pop().unwrap().0.parse::<i64>().unwrap();
                        let a = constants.pop().unwrap().0.parse::<i64>().unwrap();

                        let result = match &self.bytecode[j] {
                            Instruction::AddInt => a + b,
                            Instruction::SubInt => a - b,
                            Instruction::MulInt => a * b,
                            Instruction::DivInt => {
                                if b == 0 {
                                    can_fold = false;
                                    break;
                                }
                                a / b
                            }
                            _ => {
                                can_fold = false;
                                break;
                            }
                        };

                        constants.push((result.to_string(), "int"));
                    }

                    Instruction::AddFloat
                    | Instruction::SubFloat
                    | Instruction::MulFloat
                    | Instruction::DivFloat
                        if last_type == Some("float") && constants.len() >= 2 =>
                    {
                        let b = constants.pop().unwrap().0.parse::<f64>().unwrap();
                        let a = constants.pop().unwrap().0.parse::<f64>().unwrap();

                        let result = match &self.bytecode[j] {
                            Instruction::AddFloat => a + b,
                            Instruction::SubFloat => a - b,
                            Instruction::MulFloat => a * b,
                            Instruction::DivFloat => {
                                if b == 0.0 {
                                    can_fold = false;
                                    break;
                                }
                                a / b
                            }
                            _ => {
                                can_fold = false;
                                break;
                            }
                        };

                        constants.push((result.to_string(), "float"));
                    }

                    Instruction::Concat if last_type == Some("string") && constants.len() >= 2 => {
                        let b = constants.pop().unwrap().0;
                        let a = constants.pop().unwrap().0;

                        let result = format!("{}{}", a, b);
                        constants.push((result, "string"));
                    }

                    _ => {
                        break;
                    }
                }
                j += 1;
            }

            if can_fold && !constants.is_empty() && j > i {
                self.bytecode.drain(i..j);

                let (folded_value, value_type) = constants.pop().unwrap();
                if value_type == "int" {
                    self.bytecode
                        .insert(i, Instruction::PushInt(folded_value.parse().unwrap()));
                } else if value_type == "float" {
                    self.bytecode
                        .insert(i, Instruction::PushFloat(folded_value.parse().unwrap()));
                } else if value_type == "string" {
                    self.bytecode
                        .insert(i, Instruction::PushString(folded_value.into()));
                }

                i += 1;
            } else {
                i += 1;
            }
        }
    }

    fn peephole_optimization(&mut self) {
        let mut i = 0;
        while i < self.bytecode.len() {
            if i + 1 < self.bytecode.len() {
                match (&self.bytecode[i], &self.bytecode[i + 1]) {
                    (
                        Instruction::LoadToStack(first_variable),
                        Instruction::StoreInMap(second_variable),
                    ) if first_variable == second_variable => {
                        self.bytecode.remove(i);
                        self.bytecode.remove(i);
                        continue;
                    }
                    (
                        Instruction::PushInt(_)
                        | Instruction::PushFloat(_)
                        | Instruction::PushString(_)
                        | Instruction::PushChar(_),
                        Instruction::Pop,
                    ) => {
                        self.bytecode.remove(i);
                        self.bytecode.remove(i);
                        continue;
                    }
                    _ => {}
                }
            }
            i += 1;
        }
    }

    fn removing_empty_conditionals(&mut self) {
        let mut i = 0;
        while i < self.bytecode.len() {
            match &self.bytecode[i] {
                Instruction::JumpIfFalse(target) => {
                    let mut end = i;
                    let mut terminate = false;
                    if *target == self.shift + i + 1 {
                        terminate = true
                    }
                    if i + 1 < self.bytecode.len() {
                        if let Instruction::Jump(second_target) = &self.bytecode[i + 1] {
                            if *second_target == self.shift + i + 2 && *target == self.shift + i + 2
                            {
                                terminate = true;
                                end = i + 1;
                            }
                        }
                    }
                    if terminate {
                        let mut start = i;
                        let mut stack_balance = -1;
                        while start > 0 {
                            match &self.bytecode[start - 1] {
                                Instruction::PushInt(_)
                                | Instruction::PushFloat(_)
                                | Instruction::PushString(_)
                                | Instruction::PushChar(_)
                                | Instruction::PushArray(_)
                                | Instruction::LoadToStack(_)
                                | Instruction::NewStruct(_) => {
                                    stack_balance += 1;
                                    if stack_balance != 0 {
                                        start -= 1;
                                    } else {
                                        break;
                                    }
                                }
                                Instruction::AddInt
                                | Instruction::SubInt
                                | Instruction::MulInt
                                | Instruction::DivInt
                                | Instruction::Mod
                                | Instruction::AddFloat
                                | Instruction::SubFloat
                                | Instruction::MulFloat
                                | Instruction::DivFloat
                                | Instruction::Equal
                                | Instruction::NotEqual
                                | Instruction::And
                                | Instruction::Or
                                | Instruction::LessInt
                                | Instruction::LessEqualInt
                                | Instruction::GreaterInt
                                | Instruction::GreaterEqualInt
                                | Instruction::LessFloat
                                | Instruction::LessEqualFloat
                                | Instruction::GreaterFloat
                                | Instruction::GreaterEqualFloat
                                | Instruction::Concat
                                | Instruction::GetByIndex => {
                                    stack_balance -= 1;
                                    start -= 1;
                                }
                                Instruction::GetField(_) => {
                                    start -= 1;
                                }
                                _ => break,
                            }
                        }
                        if stack_balance == 0 {
                            for j in 0..self.bytecode.len() {
                                if let Instruction::Jump(target)
                                | Instruction::JumpIfTrue(target)
                                | Instruction::JumpIfFalse(target) = &mut self.bytecode[j]
                                {
                                    if *target > start {
                                        *target = target.saturating_sub(end - start + 2);
                                    }
                                }
                            }

                            self.bytecode.drain(start - 1..=end);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn dead_code_elimination(&mut self) {
        let mut i = 0;
        let mut func_args = Vec::new();
        let mut used_variables = HashMap::<EcoString, bool>::new();
        while i < self.bytecode.len() {
            if let Instruction::StoreInMap(var_name) = &self.bytecode[i] {
                func_args.push(var_name.clone());
            } else {
                break;
            }
            i += 1;
        }
        i = 0;
        while i < self.bytecode.len() {
            if let Instruction::StoreInMap(var_name) = &self.bytecode[i] {
                if func_args.contains(&var_name) {
                    i += 1;
                    continue;
                }
                let is_used = self.is_variable_actually_used(&var_name, i + 1);
                if !used_variables.contains_key(var_name) {
                    used_variables.insert(var_name.clone(), is_used);
                }
                if !used_variables.get(var_name).unwrap_or(&true) {
                    let mut assignments = vec![i];
                    let mut current = i + 1;
                    while current < self.bytecode.len() {
                        if let Instruction::StoreInMap(name) = &self.bytecode[current] {
                            if name == var_name {
                                assignments.push(current);
                            }
                        }
                        current += 1;
                    }
                    for &pos in assignments.iter().rev() {
                        let mut start = pos;
                        let mut stack_balance = -1;
                        while start > 0 {
                            match &self.bytecode[start - 1] {
                                Instruction::PushInt(_)
                                | Instruction::PushFloat(_)
                                | Instruction::PushString(_)
                                | Instruction::PushChar(_)
                                | Instruction::PushArray(_)
                                | Instruction::LoadToStack(_)
                                | Instruction::NewStruct(_) => {
                                    stack_balance += 1;
                                    if stack_balance != 0 {
                                        start -= 1;
                                    } else {
                                        break;
                                    }
                                }
                                Instruction::AddInt
                                | Instruction::SubInt
                                | Instruction::MulInt
                                | Instruction::DivInt
                                | Instruction::Mod
                                | Instruction::AddFloat
                                | Instruction::SubFloat
                                | Instruction::MulFloat
                                | Instruction::DivFloat
                                | Instruction::Equal
                                | Instruction::NotEqual
                                | Instruction::And
                                | Instruction::Or
                                | Instruction::LessInt
                                | Instruction::LessEqualInt
                                | Instruction::GreaterInt
                                | Instruction::GreaterEqualInt
                                | Instruction::LessFloat
                                | Instruction::LessEqualFloat
                                | Instruction::GreaterFloat
                                | Instruction::GreaterEqualFloat
                                | Instruction::Concat
                                | Instruction::GetByIndex
                                | Instruction::SetField(_)
                                | Instruction::Append => {
                                    stack_balance -= 1;
                                    start -= 1;
                                }
                                Instruction::GetField(_) => {
                                    start -= 1;
                                }
                                _ => break,
                            }
                        }
                        if stack_balance == 0 {
                            let removed_len = pos - (start - 1) + 1; // +1 так как включаем start-1 в удаление

                            for j in 0..self.bytecode.len() {
                                if let Instruction::Jump(target)
                                | Instruction::JumpIfTrue(target)
                                | Instruction::JumpIfFalse(target) = &mut self.bytecode[j]
                                {
                                    if *target > pos {
                                        *target = target.saturating_sub(removed_len);
                                    }
                                }
                            }

                            self.bytecode.drain(start - 1..=pos);
                        }
                    }
                    i = assignments[0];
                }
            }
            i += 1;
        }
    }

    fn is_variable_actually_used(&self, var_name: &str, start_pos: usize) -> bool {
        let mut i = start_pos;
        while i < self.bytecode.len() {
            match &self.bytecode[i] {
                Instruction::LoadToStack(name) if name == var_name => {
                    if i > 0 {
                        let mut j = i + 1;
                        let mut isassignment = false;
                        while j < self.bytecode.len() {
                            match &self.bytecode[j] {
                                Instruction::PushInt(_)
                                | Instruction::PushFloat(_)
                                | Instruction::PushString(_)
                                | Instruction::PushChar(_)
                                | Instruction::LoadToStack(_)
                                | Instruction::AddInt
                                | Instruction::SubInt
                                | Instruction::MulInt
                                | Instruction::DivInt
                                | Instruction::GreaterInt
                                | Instruction::GreaterEqualInt => {
                                    j += 1;
                                    continue;
                                }
                                Instruction::JumpIfFalse(_) => {
                                    isassignment = false;
                                    break;
                                }
                                Instruction::StoreInMap(store_name) if store_name == var_name => {
                                    isassignment = true;
                                    break;
                                }
                                _ => break,
                            }
                        }
                        if !isassignment {
                            return true;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }
}
