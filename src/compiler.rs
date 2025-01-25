use crate::vm::instruction::{Bytecode, Instruction};

pub struct Compiler {
    bytecode: Bytecode,
}

impl Compiler {
    #[must_use]
    pub fn new(bytecode: Bytecode) -> Self {
        Self { bytecode }
    }

    pub fn optimize(&mut self) -> Bytecode {
        let mut changed = true;
        while changed {
            let len_before = self.bytecode.len();
            //self.constant_folding();
            self.peephole_optimization();
            changed = self.bytecode.len() != len_before;
        }
        self.bytecode.clone()
    }


    fn constant_folding(&mut self) {
        let mut i = 0;
        while i < self.bytecode.len() {
            let mut constants = Vec::new();
            let mut j = i;
            let mut can_fold = true;

            while j < self.bytecode.len() {
                match &self.bytecode[j] {

                    Instruction::JumpIfTrue(_)
                    | Instruction::JumpIfFalse(_)
                    | Instruction::Jump(_)
                    | Instruction::Func(_)
                    | Instruction::Return
                    | Instruction::EndFunc => break,

                    Instruction::PushInt(n) => {
                        constants.push(*n);
                    }

                    Instruction::Call(_)
                    | Instruction::GetField(_)
                    | Instruction::GetByIndex
                    | Instruction::LoadToStack(_) => {
                        can_fold = false;
                        break;
                    }

                    Instruction::AddInt | Instruction::SubInt | Instruction::MulInt | Instruction::DivInt => {
                        if constants.len() < 2 {
                            can_fold = false;
                            break;
                        }

                        let b = constants.pop().unwrap();
                        let a = constants.pop().unwrap();

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
                            _ => unreachable!(),
                        };

                        constants.push(result);
                    }

                    _ => break,
                }

                j += 1;
            }

            if can_fold && !constants.is_empty() && j > i {
                let folded_value = constants.pop().unwrap();

                self.bytecode.drain(i..j);
                self.bytecode.insert(i, Instruction::PushInt(folded_value));

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
}
