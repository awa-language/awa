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

    /*
    fn constant_folding(&mut self) {
        let mut i = 0;
        while i < self.bytecode.len() {
            let mut constants = Vec::new();
            let mut j = i;
            let mut can_fold = true;

            while j < self.bytecode.len() {
                match &self.bytecode[j] {
                    // Остановиться на управляющих конструкциях
                    Instruction::JumpIfTrue(_)
                    | Instruction::JumpIfFalse(_)
                    | Instruction::Jump(_)
                    | Instruction::Func(_)
                    | Instruction::Return
                    | Instruction::EndFunc => break,

                    // Сохранить константы
                    Instruction::PushInt(n) => {
                        constants.push(*n);
                    }

                    // Если встречается инструкция, зависящая от времени выполнения, остановиться
                    Instruction::Call(_)
                    | Instruction::GetField(_)
                    | Instruction::GetByIndex
                    | Instruction::LoadToStack(_) => {
                        can_fold = false;
                        break;
                    }

                    // Арифметика
                    Instruction::AddInt => {
                        if constants.len() >= 2 {
                            let b = constants.pop().unwrap();
                            let a = constants.pop().unwrap();
                            constants.push(a + b);
                        } else {
                            can_fold = false;
                            break;
                        }
                    }
                    Instruction::SubInt => {
                        if constants.len() >= 2 {
                            let b = constants.pop().unwrap();
                            let a = constants.pop().unwrap();
                            constants.push(a - b);
                        } else {
                            can_fold = false;
                            break;
                        }
                    }
                    Instruction::MulInt => {
                        if constants.len() >= 2 {
                            let b = constants.pop().unwrap();
                            let a = constants.pop().unwrap();
                            constants.push(a * b);
                        } else {
                            can_fold = false;
                            break;
                        }
                    }
                    Instruction::DivInt => {
                        if constants.len() >= 2 {
                            let b = constants.pop().unwrap();
                            let a = constants.pop().unwrap();
                            if b != 0 {
                                constants.push(a / b);
                            } else {
                                can_fold = false;
                                break;
                            }
                        } else {
                            can_fold = false;
                            break;
                        }
                    }

                    _ => break,
                }
                j += 1;
            }

            // Если есть что оптимизировать и все инструкции были константными
            if constants.len() > 0 && can_fold && j > i {
                // Вычисленное значение
                let folded_value = constants.pop().unwrap();

                // Удаляем инструкции от i до j
                self.bytecode.drain(i..j);

                // Вставляем одну инструкцию PushInt
                self.bytecode.insert(i, Instruction::PushInt(folded_value));

                // Продолжаем с позиции после вставленной инструкции
                i += 1;
            } else {
                i += 1;
            }
        }
    } */

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
