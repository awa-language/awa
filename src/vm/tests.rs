use super::instruction::Bytecode;
use super::instruction::Instruction;
use super::vm::Value;
use super::vm::VM;
use std::collections::HashMap;

#[test]
fn test_push_load_store() {
    let bytecode = vec![
        Instruction::PushInt(42),
        Instruction::Store("x".to_string()),
        Instruction::Load("x".to_string()),
        Instruction::Println,
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_arithmetic_int() {
    let bytecode = vec![
        Instruction::PushInt(10),
        Instruction::PushInt(5),
        Instruction::AddInt,  // Stack: 15
        Instruction::Println, // Expected output: 15
        Instruction::PushInt(3),
        Instruction::SubInt,  // Stack: 12
        Instruction::Println, // Expected output: 12
        Instruction::PushInt(4),
        Instruction::MulInt,  // Stack: 48
        Instruction::Println, // Expected output: 48
        Instruction::PushInt(6),
        Instruction::DivInt,  // Stack: 8
        Instruction::Println, // Expected output: 8
        Instruction::PushInt(3),
        Instruction::Mod,     // Stack: 2
        Instruction::Println, // Expected output: 2
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_arithmetic_float() {
    let bytecode = vec![
        Instruction::PushFloat(10.5),
        Instruction::PushFloat(2.5),
        Instruction::AddFloat, // Stack: 13.0
        Instruction::Println,  // Expected output: 13.0
        Instruction::PushFloat(5.0),
        Instruction::SubFloat, // Stack: 8.0
        Instruction::Println,  // Expected output: 8.0
        Instruction::PushFloat(3.0),
        Instruction::MulFloat, // Stack: 24.0
        Instruction::Println,  // Expected output: 24.0
        Instruction::PushFloat(4.0),
        Instruction::DivFloat, // Stack: 6.0
        Instruction::Println,  // Expected output: 6.0
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_comparisons() {
    let bytecode = vec![
        // a = 10, b = 20
        Instruction::PushInt(10),
        Instruction::Store("a".to_string()),
        Instruction::PushInt(20),
        Instruction::Store("b".to_string()),
        // a == b -> false (0)
        Instruction::Load("a".to_string()),
        Instruction::Load("b".to_string()),
        Instruction::Equal,   // Stack: 0
        Instruction::Println, // Expected output: 0
        // a != b -> true (1)
        Instruction::Load("a".to_string()),
        Instruction::Load("b".to_string()),
        Instruction::NotEqual, // Stack: 1
        Instruction::Println,  // Expected output: 1
        // a < b -> true (1)
        Instruction::Load("a".to_string()),
        Instruction::Load("b".to_string()),
        Instruction::LessInt, // Stack: 1
        Instruction::Println, // Expected output: 1
        // a <= 10 -> true (1)
        Instruction::Load("a".to_string()),
        Instruction::PushInt(10),
        Instruction::LessEqualInt, // Stack: 1
        Instruction::Println,      // Expected output: 1
        // b > 15 -> true (1)
        Instruction::Load("b".to_string()),
        Instruction::PushInt(15),
        Instruction::GreaterInt, // Stack: 1
        Instruction::Println,    // Expected output: 1
        // b >= 25 -> false (0)
        Instruction::Load("b".to_string()),
        Instruction::PushInt(25),
        Instruction::GreaterEqualInt, // Stack: 0
        Instruction::Println,         // Expected output: 0
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_jumps() {
    let bytecode = vec![
        // Push 1 (true)
        Instruction::PushInt(1),
        Instruction::JumpIfTrue(4), // Jump to instruction 4
        // This code is skipped
        Instruction::PushInt(999),
        Instruction::Println,
        // Label 4: Push 2 and print
        Instruction::PushInt(2),
        Instruction::Println,
        // Jump to Halt
        Instruction::Jump(9),
        // This code is skipped
        Instruction::PushInt(888),
        Instruction::Println,
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_if_else() {
    let bytecode = vec![
        // a = 10
        Instruction::PushInt(10),
        Instruction::Store("a".to_string()),
        // b = 20
        Instruction::PushInt(20),
        Instruction::Store("b".to_string()),
        // c = a + b
        Instruction::Load("a".to_string()),
        Instruction::Load("b".to_string()),
        Instruction::AddInt,
        Instruction::Store("c".to_string()),
        // if (c > 25) { println("c is greater than 25"); } else { println("c is not greater than 25"); }
        Instruction::Load("c".to_string()),
        Instruction::PushInt(25),
        Instruction::GreaterInt,      // Stack: 1 (true)
        Instruction::JumpIfFalse(15), // If false, jump to instruction 15
        // True block
        Instruction::PushStr("c is greater than 25".to_string()),
        Instruction::Println,
        Instruction::Jump(17), // Jump to instruction 17
        // False block
        Instruction::PushStr("c is not greater than 25".to_string()),
        Instruction::Println,
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_functions() {
    let bytecode = vec![
        // Function add starts at address 0
        Instruction::Store("a".to_string()),
        Instruction::Store("b".to_string()),
        Instruction::Load("a".to_string()),
        Instruction::Load("b".to_string()),
        Instruction::AddInt,
        Instruction::Return,
        // Function main starts at address 4
        Instruction::PushInt(5),
        Instruction::PushInt(7),
        Instruction::Call("add".to_string()), // Stack: 12
        Instruction::Store("c".to_string()),
        Instruction::Load("c".to_string()),
        Instruction::Println, // Expected output: 12
        Instruction::Halt,
    ];

    let mut functions = HashMap::new();
    functions.insert("add".to_string(), 0);

    let mut vm = VM::new(bytecode, functions, 6);
    vm.run();
}

#[test]
fn test_structs() {
    let bytecode = vec![
        // Create struct Person
        Instruction::NewStruct("Person".to_string()),
        Instruction::PushStr("Alice".to_string()),
        Instruction::SetField("name".to_string()),
        Instruction::PushInt(30),
        Instruction::SetField("age".to_string()),
        Instruction::Store("person".to_string()),
        // Get field age and print
        Instruction::Load("person".to_string()),
        Instruction::GetField("age".to_string()),
        Instruction::Println, // Expected output: 30
        // Get field name and print
        Instruction::Load("person".to_string()),
        Instruction::GetField("name".to_string()),
        Instruction::Println, // Expected output: "Alice"
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}

#[test]
fn test_concat() {
    let bytecode = vec![
        Instruction::PushStr("Hello, ".to_string()),
        Instruction::PushStr("World!".to_string()),
        Instruction::Concat,  // Stack: "Hello, World!"
        Instruction::Println, // Expected output: "Hello, World!"
        Instruction::Halt,
    ];

    let functions = HashMap::new();
    let mut vm = VM::new(bytecode, functions, 0);
    vm.run();
}
