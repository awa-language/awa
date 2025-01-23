use std::collections::HashMap;

use ecow::EcoString;

use crate::{
    ast::{
        definition::DefinitionTyped, expression::TypedExpression, module::Module,
        operator::BinaryOperator, reassignment::TypedReassignmentTarget, statement::TypedStatement,
    },
    type_::Type,
    vm::instruction::{Bytecode, Instruction, Value},
};

pub struct Interpreter {
    bytecode: Bytecode,
    current_func: Option<EcoString>,
    loop_end_stack: Vec<usize>,
    loop_start_stack: Vec<usize>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            current_func: None,
            loop_end_stack: Vec::new(),
            loop_start_stack: Vec::new(),
        }
    }

    #[must_use]
    pub fn interpret_module(mut self, module: &Module<DefinitionTyped>) -> Bytecode {
        if let Some(definitions) = &module.definitions {
            // First pass - declare all structs
            for definition in definitions {
                if let DefinitionTyped::Struct { name, fields, .. } = definition {
                    self.bytecode.push(Instruction::Struct(name.clone()));
                    if let Some(fields) = fields {
                        for field in fields {
                            self.bytecode.push(Instruction::Field(
                                field.name.clone(),
                                Self::default_value_for_type(&field.type_),
                            ));
                        }
                    }
                    self.bytecode.push(Instruction::EndStruct);
                }
            }

            // Second pass - process functions
            for definition in definitions {
                if let DefinitionTyped::Function {
                    name,
                    arguments,
                    body,
                    return_type,
                    ..
                } = definition
                {
                    self.current_func = Some(name.clone());
                    self.bytecode.push(Instruction::Func(name.clone()));

                    // Store arguments in reverse order
                    if let Some(arguments) = arguments {
                        for argument in arguments.iter().rev() {
                            self.bytecode
                                .push(Instruction::StoreInMap(argument.name.clone()));
                        }
                    }

                    if let Some(statements) = body {
                        for statement in statements {
                            self.interpret_statement(statement);
                        }
                    }

                    // Add implicit return if needed
                    if matches!(return_type, Type::Void) {
                        self.bytecode.push(Instruction::Return);
                    }

                    self.bytecode.push(Instruction::EndFunc);
                }
            }
        }

        self.bytecode
    }

    fn interpret_statement(&mut self, statement: &TypedStatement) {
        match statement {
            TypedStatement::Expression(expression) => {
                self.interpret_expression(expression);
            }
            TypedStatement::Assignment(assign) => {
                self.interpret_expression(&assign.value);
                self.bytecode
                    .push(Instruction::StoreInMap(assign.variable_name.clone()));
            }
            TypedStatement::Reassignment(reassign) => match &reassign.target {
                TypedReassignmentTarget::Variable { name, .. } => {
                    self.interpret_expression(&reassign.new_value);
                    self.bytecode.push(Instruction::StoreInMap(name.clone()));
                }
                TypedReassignmentTarget::FieldAccess {
                    struct_name,
                    field_name,
                    ..
                } => {
                    self.interpret_expression(&reassign.new_value);
                    self.bytecode
                        .push(Instruction::LoadToStack(struct_name.clone()));
                    self.bytecode
                        .push(Instruction::SetField(field_name.clone()));
                    self.bytecode
                        .push(Instruction::StoreInMap(struct_name.clone()));
                }
                TypedReassignmentTarget::ArrayAccess {
                    array_name,
                    index_expression,
                    ..
                } => {
                    self.bytecode
                        .push(Instruction::LoadToStack(array_name.clone()));
                    self.interpret_expression(index_expression);
                    self.interpret_expression(&reassign.new_value);
                    self.bytecode.push(Instruction::SetByIndex);
                    self.bytecode
                        .push(Instruction::StoreInMap(array_name.clone()));
                }
            },
            TypedStatement::Loop { body, .. } => {
                let loop_start = self.bytecode.len();
                self.loop_start_stack.push(loop_start);

                if let Some(statements) = body {
                    for stmt in statements {
                        self.interpret_statement(stmt);
                    }
                }

                self.bytecode.push(Instruction::Jump(loop_start));
                let loop_end = self.bytecode.len();
                self.loop_end_stack.push(loop_end);
            }
            TypedStatement::Break { .. } => {
                if let Some(end) = self.loop_end_stack.last() {
                    self.bytecode.push(Instruction::Jump(*end));
                }
            }
            TypedStatement::Return { value, .. } => {
                if let Some(expression) = value {
                    self.interpret_expression(expression);
                }
                self.bytecode.push(Instruction::Return);
            }
            TypedStatement::If {
                condition,
                if_body,
                else_body,
                ..
            } => {
                self.interpret_expression(condition);

                let jump_if_false = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder

                if let Some(statements) = if_body {
                    for statement in statements {
                        self.interpret_statement(statement);
                    }
                }

                let jump_to_end = self.bytecode.len();
                self.bytecode.push(Instruction::Jump(0)); // Placeholder

                let else_start = self.bytecode.len();
                if let Some(statements) = else_body {
                    for stmt in statements {
                        self.interpret_statement(stmt);
                    }
                }

                let end = self.bytecode.len();

                // Fix up jumps
                if let Instruction::JumpIfFalse(address) = &mut self.bytecode[jump_if_false] {
                    *address = else_start;
                }
                if let Instruction::Jump(address) = &mut self.bytecode[jump_to_end] {
                    *address = end;
                }
            }
            TypedStatement::Todo { .. }
            | TypedStatement::Panic { .. }
            | TypedStatement::Exit { .. } => {
                self.bytecode.push(Instruction::Halt);
            }
        }
    }

    fn interpret_expression(&mut self, expr: &TypedExpression) {
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
                if let Some(args) = arguments {
                    for arg in args {
                        self.interpret_expression(&arg.value);
                    }
                }
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
                self.interpret_expression(index_expression);
                self.bytecode.push(Instruction::GetByIndex);
            }
            TypedExpression::ArrayInitialization { elements, .. } => {
                self.bytecode.push(Instruction::PushArray(Vec::new()));
                if let Some(elems) = elements {
                    for elem in elems {
                        self.interpret_expression(elem);
                        self.bytecode.push(Instruction::Append);
                    }
                }
            }
            TypedExpression::StructInitialization { fields, type_, .. } => {
                if let Type::Custom { name } = type_ {
                    self.bytecode.push(Instruction::NewStruct(name.clone()));
                    if let Some(fields) = fields {
                        for field in fields {
                            self.interpret_expression(&field.value);
                            self.bytecode
                                .push(Instruction::SetField(field.name.clone()));
                        }
                    }
                }
            }
            TypedExpression::BinaryOperation {
                operator,
                left,
                right,
                ..
            } => {
                self.interpret_expression(left);
                self.interpret_expression(right);
                match operator {
                    BinaryOperator::AdditionInt => self.bytecode.push(Instruction::AddInt),
                    BinaryOperator::SubtractionInt => self.bytecode.push(Instruction::SubInt),
                    BinaryOperator::MultipicationInt => self.bytecode.push(Instruction::MulInt),
                    BinaryOperator::DivisionInt => self.bytecode.push(Instruction::DivInt),
                    BinaryOperator::Modulo => self.bytecode.push(Instruction::Mod),
                    BinaryOperator::AdditionFloat => self.bytecode.push(Instruction::AddFloat),
                    BinaryOperator::SubtractionFloat => self.bytecode.push(Instruction::SubFloat),
                    BinaryOperator::MultipicationFloat => self.bytecode.push(Instruction::MulFloat),
                    BinaryOperator::DivisionFloat => self.bytecode.push(Instruction::DivFloat),
                    BinaryOperator::Equal => self.bytecode.push(Instruction::Equal),
                    BinaryOperator::NotEqual => self.bytecode.push(Instruction::NotEqual),
                    BinaryOperator::LessInt => self.bytecode.push(Instruction::LessInt),
                    BinaryOperator::LessEqualInt => self.bytecode.push(Instruction::LessEqualInt),
                    BinaryOperator::GreaterInt => self.bytecode.push(Instruction::GreaterInt),
                    BinaryOperator::GreaterEqualInt => {
                        self.bytecode.push(Instruction::GreaterEqualInt);
                    }
                    BinaryOperator::LessFloat => self.bytecode.push(Instruction::LessFloat),
                    BinaryOperator::LessEqualFloat => {
                        self.bytecode.push(Instruction::LessEqualFloat);
                    }
                    BinaryOperator::GreaterFloat => self.bytecode.push(Instruction::GreaterFloat),
                    BinaryOperator::GreaterEqualFloat => {
                        self.bytecode.push(Instruction::GreaterEqualFloat);
                    }
                    BinaryOperator::Concatenation => self.bytecode.push(Instruction::Concat),
                    BinaryOperator::And => self.bytecode.push(Instruction::And),
                    BinaryOperator::Or => self.bytecode.push(Instruction::Or),
                }
            }
        }
    }

    fn default_value_for_type(type_: &Type) -> Value {
        match type_ {
            Type::Int => Value::Int(0),
            Type::Float => Value::Float(0.0),
            Type::String => Value::String("".into()),
            Type::Char => Value::Char('\0'),
            Type::Array { .. } => Value::Slice(Vec::new()),
            Type::Boolean => Value::Int(0),
            Type::Void => Value::Nil,
            Type::Custom { name } => Value::Struct {
                name: name.clone(),
                fields: HashMap::new(),
            },
        }
    }
}
