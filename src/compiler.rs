use crate::ast::statement::TypedStatement;
use crate::ast::{
    definition::DefinitionTyped, expression::TypedExpression, operator::BinaryOperator,
    reassignment::TypedReassignment, reassignment::TypedReassignmentTarget,
};
use crate::type_::Type;
use crate::vm::instruction::{Bytecode, Instruction, Value};
use ecow::EcoString;
use std::collections::HashMap;
/// Основной компилятор
pub struct Compiler {
    /// Здесь будет итоговый байткод
    pub bytecode: Bytecode,

    /// Для хранения информации о функциях: имя -> индекс начала
    pub functions: HashMap<EcoString, usize>,

    /// Для хранения информации о структурах: имя -> список полей
    pub structs: HashMap<EcoString, Vec<EcoString>>,
}

impl Compiler {
    /// Создаём новый компилятор
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
        }
    }

    /// Основной метод: скомпилировать список определений (функции/структуры и т.д.)
    pub fn compile_program(&mut self, definitions: &[DefinitionTyped]) {
        // Сначала регистрируем, чтобы знать адреса (functions) и структуру (structs)
        for def in definitions {
            self.register_definitions(def);
        }

        // Генерируем код для каждого определения
        for def in definitions {
            self.compile_definition(def);
        }
    }

    /// Сбор метаданных: имена функций, структур и т.д.
    fn register_definitions(&mut self, def: &DefinitionTyped) {
        match def {
            DefinitionTyped::Struct { name, fields, .. } => {
                let field_names = fields
                    .as_ref()
                    .map(|f| f.iter().map(|sf| sf.name.clone()).collect())
                    .unwrap_or_default();
                self.structs.insert(name.clone(), field_names);
            }
            DefinitionTyped::Function { name, .. } => {
                // Пускай адрес начала функции будет просто текущий размер байткода
                let func_address = self.bytecode.len();
                self.functions.insert(name.clone(), func_address);
            }
        }
    }

    /// Генерация непосредственных инструкций для DefinitionTyped
    fn compile_definition(&mut self, def: &DefinitionTyped) {
        match def {
            DefinitionTyped::Struct { name, fields, .. } => {
                // Если хотите реально генерировать инструкции для описания структуры
                self.bytecode.push(Instruction::Struct(name.clone()));
                if let Some(f) = fields {
                    for field in f {
                        // В момент объявления структуры можно делать что-то вроде:
                        self.bytecode
                            .push(Instruction::Field(field.name.clone(), Value::Nil));
                    }
                }
                self.bytecode.push(Instruction::EndStruct);
            }
            DefinitionTyped::Function {
                name,
                arguments,
                body,
                ..
            } => {
                // Начало функции
                self.bytecode.push(Instruction::Func(name.clone()));

                // Параметры
                if let Some(args_vec1) = arguments {
                    for arg in args_vec1.iter() {
                        // Сразу StoreInMap для каждого аргумента
                        self.bytecode
                            .push(Instruction::StoreInMap(arg.name.clone()));
                    }
                }

                // Тело функции
                if let Some(body_vec1) = body {
                    for stmt in body_vec1.iter() {
                        self.compile_statement(stmt);
                    }
                }

                // Завершение функции
                self.bytecode.push(Instruction::EndFunc);
            }
        }
    }

    /// Компиляция оператора (Statement)
    fn compile_statement(&mut self, stmt: &TypedStatement) {
        match stmt {
            TypedStatement::Expression(expr) => {
                self.compile_expression(expr);
                // при необходимости можно убрать результат из стека (нет Pop в Instruction)
            }
            TypedStatement::Assignment(assign) => {
                // a = expr
                self.compile_expression(&assign.value);
                self.bytecode
                    .push(Instruction::StoreInMap(assign.variable_name.clone()));
            }
            TypedStatement::Reassignment(reassign) => {
                self.compile_reassignment(reassign);
            }
            TypedStatement::Loop { body, .. } => {
                let start_ip = self.bytecode.len(); // начало цикла

                // Тело
                if let Some(vec1) = body {
                    for s in vec1 {
                        self.compile_statement(s);
                    }
                }

                // Безусловный прыжок на начало
                self.bytecode.push(Instruction::Jump(start_ip));

                // Примечание: если используете break, нужно размечать конец цикла
            }
            TypedStatement::If {
                condition,
                if_body,
                else_body,
                ..
            } => {
                // 1. Вычисляем условие
                self.compile_expression(condition);

                // 2. JumpIfFalse(...) с пока 0
                let jump_if_false_index = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0));

                // 3. if_body
                if let Some(ifb) = if_body {
                    for s in ifb {
                        self.compile_statement(s);
                    }
                }

                // 4. Переход из конца if на конец else
                let jump_end_if = self.bytecode.len();
                self.bytecode.push(Instruction::Jump(0));

                // 5. Заменяем адрес в JumpIfFalse на начало else
                let else_start = self.bytecode.len();
                if let Instruction::JumpIfFalse(ref mut addr) = self.bytecode[jump_if_false_index] {
                    *addr = else_start;
                }

                // 6. else_body
                if let Some(elseb) = else_body {
                    for s in elseb {
                        self.compile_statement(s);
                    }
                }

                // 7. конец else
                let end_of_else = self.bytecode.len();
                if let Instruction::Jump(ref mut addr) = self.bytecode[jump_end_if] {
                    *addr = end_of_else;
                }
            }
            TypedStatement::Break { .. } => {
                // Нужно знать, куда прыгать — конец цикла
                // Для этого обычно нужен стек меток или что-то в этом духе.
                // Упрощённо (заглушка):
                self.bytecode.push(Instruction::Halt);
            }
            TypedStatement::Return { value, .. } => {
                if let Some(expr) = value {
                    self.compile_expression(expr);
                }
                // Возвращаем
                self.bytecode.push(Instruction::Return);
            }
            TypedStatement::Todo { .. } => {
                // Заглушка, можете вставить Instruction::Halt или Panic
                self.bytecode.push(Instruction::Halt);
            }
            TypedStatement::Panic { .. } => {
                // Аналогично
                self.bytecode.push(Instruction::Halt);
            }
            TypedStatement::Exit { .. } => {
                // Завершаем программу
                self.bytecode.push(Instruction::Halt);
            }
        }
    }

    /// Компиляция переприсваивания (Reassignment)
    fn compile_reassignment(&mut self, reassign: &TypedReassignment) {
        match &reassign.target {
            TypedReassignmentTarget::Variable { name, .. } => {
                // variable = new_value
                self.compile_expression(&reassign.new_value);
                self.bytecode.push(Instruction::StoreInMap(name.clone()));
            }
            TypedReassignmentTarget::FieldAccess {
                struct_name,
                field_name,
                ..
            } => {
                // struct_name.field_name = new_value
                self.bytecode
                    .push(Instruction::LoadToStack(struct_name.clone()));
                self.compile_expression(&reassign.new_value);
                self.bytecode
                    .push(Instruction::SetField(field_name.clone()));
            }
            TypedReassignmentTarget::ArrayAccess {
                array_name,
                index_expression,
                ..
            } => {
                // array_name[index] = newValue
                self.bytecode
                    .push(Instruction::LoadToStack(array_name.clone()));
                self.compile_expression(index_expression);
                self.compile_expression(&reassign.new_value);
                // У вас Instruction::SetByIndex(i64) — работает только для
                // константного индекса. Для динамического индекса нужна своя инструкция,
                // например, SetIndex из стека. Для примера оставим так:
                self.bytecode.push(Instruction::SetByIndex(0));
            }
        }
    }

    /// Компиляция выражений
    fn compile_expression(&mut self, expr: &TypedExpression) {
        match expr {
            TypedExpression::IntLiteral { value, .. } => {
                self.bytecode.push(Instruction::PushInt(*value));
            }
            TypedExpression::FloatLiteral { value, .. } => {
                self.bytecode.push(Instruction::PushFloat(*value));
            }
            TypedExpression::StringLiteral { value, .. } => {
                self.bytecode.push(Instruction::PushString(value.clone()));
            }
            TypedExpression::CharLiteral { value, .. } => {
                self.bytecode.push(Instruction::PushChar(*value));
            }
            TypedExpression::VariableValue { name, .. } => {
                self.bytecode.push(Instruction::LoadToStack(name.clone()));
            }
            TypedExpression::FunctionCall {
                function_name,
                arguments,
                ..
            } => {
                // Скомпилировать аргументы
                if let Some(args) = arguments {
                    for arg in args {
                        self.compile_expression(&arg.value);
                    }
                }
                // Call
                self.bytecode.push(Instruction::Call(function_name.clone()));
            }
            TypedExpression::StructFieldAccess {
                struct_name,
                field_name,
                ..
            } => {
                self.bytecode
                    .push(Instruction::LoadToStack(struct_name.clone()));
                self.bytecode
                    .push(Instruction::GetField(field_name.clone()));
            }
            TypedExpression::ArrayElementAccess {
                array_name,
                index_expression,
                ..
            } => {
                self.bytecode
                    .push(Instruction::LoadToStack(array_name.clone()));
                self.compile_expression(index_expression);
                // Аналогичная проблема с GetByIndex(i64): если индекс не константа,
                // нужна другая инструкция (например, GetByIndexStack).
                self.bytecode.push(Instruction::GetByIndex(0));
            }
            TypedExpression::ArrayInitialization { elements, .. } => {
                // Пример: создаём массив из констант
                if let Some(el_vec1) = elements {
                    // Попробуем собрать в вектор Value, если всё константы
                    let mut const_values = Vec::new();
                    let mut all_const = true;
                    for elem in el_vec1 {
                        if let Some(v) = self.try_extract_const_value(elem) {
                            const_values.push(v);
                        } else {
                            all_const = false;
                            break;
                        }
                    }
                    if all_const {
                        self.bytecode.push(Instruction::PushArray(const_values));
                    } else {
                        // Иначе, придётся генерировать PushArray([]), затем поэлементно Append
                        self.bytecode.push(Instruction::PushArray(Vec::new()));
                        for elem in el_vec1 {
                            self.compile_expression(elem);
                            // Append предполагает, что значение Value уже на этапе компиляции.
                            // В реальности может быть нужно что-то типа: Append из стека.
                            // Допустим, у нас есть Instruction::Append(Value), тогда
                            // нужны доработки. Здесь оставим заглушку:
                            self.bytecode.push(Instruction::Append(Value::Nil));
                        }
                    }
                } else {
                    // Пустой массив
                    self.bytecode.push(Instruction::PushArray(Vec::new()));
                }
            }
            TypedExpression::StructInitialization { type_, fields, .. } => {
                // Предположим, что наш пользовательский тип структуры описывается как Type::Custom { name }
                let struct_name = match type_ {
                    Type::Custom { name } => name.clone(),
                    _ => {
                        // Обработка ошибки или заглушка
                        self.bytecode.push(Instruction::PushInt(0));
                        return;
                    }
                };

                // Генерируем инструкцию для создания новой структуры
                self.bytecode.push(Instruction::NewStruct(struct_name));

                // Инициализируем поля (если есть)
                if let Some(field_vec1) = fields {
                    for field_val in field_vec1 {
                        // Скомпилируем выражение для значения поля
                        self.compile_expression(&field_val.value);
                        // Генерируем установку поля
                        self.bytecode
                            .push(Instruction::SetField(field_val.name.clone()));
                    }
                }
            }
            TypedExpression::BinaryOperation {
                operator,
                left,
                right,
                ..
            } => {
                // 1. Скомпилируем left
                self.compile_expression(left);
                // 2. Скомпилируем right
                self.compile_expression(right);
                // 3. Сгенерируем инструкцию
                self.compile_binop(operator);
            }
        }
    }

    /// Подбираем инструкции под каждый BinaryOperator
    fn compile_binop(&mut self, op: &BinaryOperator) {
        match op {
            BinaryOperator::And => {
                // Если нужна короткозамкнутость (short-circuit),
                // то придётся делать логику через JumpIfFalse.
                // Иначе, если есть прямая инструкция:
                // self.bytecode.push(Instruction::And);
            }
            BinaryOperator::Or => {
                // Аналогично And
                // self.bytecode.push(Instruction::Or);
            }
            BinaryOperator::Equal => {
                self.bytecode.push(Instruction::Equal);
            }
            BinaryOperator::NotEqual => {
                self.bytecode.push(Instruction::NotEqual);
            }
            BinaryOperator::LessInt => {
                self.bytecode.push(Instruction::LessInt);
            }
            BinaryOperator::LessEqualInt => {
                self.bytecode.push(Instruction::LessEqualInt);
            }
            BinaryOperator::LessFloat => {
                self.bytecode.push(Instruction::LessFloat);
            }
            BinaryOperator::LessEqualFloat => {
                self.bytecode.push(Instruction::LessEqualFloat);
            }
            BinaryOperator::GreaterEqualInt => {
                self.bytecode.push(Instruction::GreaterEqualInt);
            }
            BinaryOperator::GreaterInt => {
                self.bytecode.push(Instruction::GreaterInt);
            }
            BinaryOperator::GreaterEqualFloat => {
                self.bytecode.push(Instruction::GreaterEqualFloat);
            }
            BinaryOperator::GreaterFloat => {
                self.bytecode.push(Instruction::GreaterFloat);
            }
            BinaryOperator::AdditionInt => {
                self.bytecode.push(Instruction::AddInt);
            }
            BinaryOperator::AdditionFloat => {
                self.bytecode.push(Instruction::AddFloat);
            }
            BinaryOperator::SubtractionInt => {
                self.bytecode.push(Instruction::SubInt);
            }
            BinaryOperator::SubtractionFloat => {
                self.bytecode.push(Instruction::SubFloat);
            }
            BinaryOperator::MultipicationInt => {
                self.bytecode.push(Instruction::MulInt);
            }
            BinaryOperator::MultipicationFloat => {
                self.bytecode.push(Instruction::MulFloat);
            }
            BinaryOperator::DivisionInt => {
                self.bytecode.push(Instruction::DivInt);
            }
            BinaryOperator::DivisionFloat => {
                self.bytecode.push(Instruction::DivFloat);
            }
            BinaryOperator::Modulo => {
                self.bytecode.push(Instruction::Mod);
            }
            BinaryOperator::Concatenation => {
                self.bytecode.push(Instruction::Concat);
            }
        }
    }

    /// Пример попытки вытащить константное значение из выражения
    /// (для ArrayInitialization).
    fn try_extract_const_value(&self, expr: &TypedExpression) -> Option<Value> {
        match expr {
            TypedExpression::IntLiteral { value, .. } => Some(Value::Int(*value)),
            TypedExpression::FloatLiteral { value, .. } => Some(Value::Float(*value)),
            TypedExpression::StringLiteral { value, .. } => Some(Value::String(value.clone())),
            TypedExpression::CharLiteral { value, .. } => Some(Value::Char(*value)),
            // и т.д. для других «констант»
            _ => None,
        }
    }
}
