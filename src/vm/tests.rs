use std::collections::HashMap;

use super::{instruction::Instruction, instruction::Value, VM};

#[test]
fn test_push_load_store() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushInt(42),
        Instruction::Store("x".into()),
        Instruction::Load("x".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_arithmetic_int() {
    let bytecode = vec![
        Instruction::Func("main".into()),
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
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_arithmetic_float() {
    let bytecode = vec![
        Instruction::Func("main".into()),
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
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_comparisons() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        // a = 10, b = 20
        Instruction::PushInt(10),
        Instruction::Store("a".into()),
        Instruction::PushInt(20),
        Instruction::Store("b".into()),
        // a == b -> false (0)
        Instruction::Load("a".into()),
        Instruction::Load("b".into()),
        Instruction::Equal,   // Stack: 0
        Instruction::Println, // Expected output: 0
        // a != b -> true (1)
        Instruction::Load("a".into()),
        Instruction::Load("b".into()),
        Instruction::NotEqual, // Stack: 1
        Instruction::Println,  // Expected output: 1
        // a < b -> true (1)
        Instruction::Load("a".into()),
        Instruction::Load("b".into()),
        Instruction::LessInt, // Stack: 1
        Instruction::Println, // Expected output: 1
        // a <= 10 -> true (1)
        Instruction::Load("a".into()),
        Instruction::PushInt(10),
        Instruction::LessEqualInt, // Stack: 1
        Instruction::Println,      // Expected output: 1
        // b > 15 -> true (1)
        Instruction::Load("b".into()),
        Instruction::PushInt(15),
        Instruction::GreaterInt, // Stack: 1
        Instruction::Println,    // Expected output: 1
        // b >= 25 -> false (0)
        Instruction::Load("b".into()),
        Instruction::PushInt(25),
        Instruction::GreaterEqualInt, // Stack: 0
        Instruction::Println,         // Expected output: 0
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_jumps() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        // Push 1 (true)
        Instruction::PushInt(1),
        Instruction::JumpIfTrue(5), // Jump to instruction 5
        // This code is skipped
        Instruction::PushInt(999),
        Instruction::Println,
        // Label 5: Push 2 and print
        Instruction::PushInt(2),
        Instruction::Println,
        // Jump to Halt
        Instruction::Jump(10),
        // This code is skipped
        Instruction::PushInt(888),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_if_else() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        // a = 10
        Instruction::PushInt(10),
        Instruction::Store("a".into()),
        // b = 20
        Instruction::PushInt(20),
        Instruction::Store("b".into()),
        // c = a + b
        Instruction::Load("a".into()),
        Instruction::Load("b".into()),
        Instruction::AddInt,
        Instruction::Store("c".into()),
        // if (c > 25) { println("c is greater than 25"); } else { println("c is not greater than 25"); }
        Instruction::Load("c".into()),
        Instruction::PushInt(25),
        Instruction::GreaterInt,      // Stack: 1 (true)
        Instruction::JumpIfFalse(16), // If false, jump to instruction 16
        // True block
        Instruction::PushStr("c is greater than 25".into()),
        Instruction::Println,
        Instruction::Jump(17), // Jump to instruction 18
        // False block
        Instruction::PushStr("c is not greater than 25".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_functions() {
    let bytecode = vec![
        // Function add starts at address 0
        Instruction::Func("add".into()),
        Instruction::Store("a".into()),
        Instruction::Store("b".into()),
        Instruction::Load("a".into()),
        Instruction::Load("b".into()),
        Instruction::AddInt,
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::PushInt(5),
        Instruction::PushInt(7),
        Instruction::Call("add".into()), // Stack: 12
        Instruction::Store("c".into()),
        Instruction::Load("c".into()),
        Instruction::Println, // Expected output: 12
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_structs() {
    let bytecode = vec![
        Instruction::Struct("Person".into()),
        Instruction::Field("name".into(), Value::String("".into())),
        Instruction::Field("age".into(), Value::Int(0)),
        Instruction::EndStruct,
        Instruction::Func("main".into()),
        Instruction::NewStruct("Person".into()),
        Instruction::Store("a".into()),
        Instruction::PushStr("nikitka".into()),
        Instruction::Load("a".into()),
        Instruction::SetField("name".into()),
        Instruction::Store("a".into()),
        Instruction::PushInt(22),
        Instruction::Load("a".into()),
        Instruction::SetField("age".into()),
        Instruction::Store("a".into()),
        Instruction::Load("a".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_concat() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushStr("Hello, ".into()),
        Instruction::PushStr("World!".into()),
        Instruction::Concat,  // Stack: "Hello, World!"
        Instruction::Println, // Expected output: "Hello, World!"
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_slice() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushSlice(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        Instruction::Println,
        Instruction::Append(Value::Int(4)),
        Instruction::Println,
        Instruction::Store("qwe".into()),
        Instruction::PushInt(22),
        Instruction::Load("qwe".into()),
        Instruction::SetByIndex(1),
        Instruction::Println,
        Instruction::GetByIndex(1),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode);

    vm.run();
}
#[test]
fn test_complex() {
    let bytecode = vec![
        Instruction::Struct("Custom".into()),
        Instruction::Field("name".into(), Value::Char('.')),
        Instruction::Field("age".into(), Value::Float(0.0)),
        Instruction::EndStruct,
        //
        Instruction::Struct("Wrapper".into()),
        Instruction::Field(
            "custom".into(),
            Value::Struct {
                name: "Custom".into(),
                fields: HashMap::from([
                    ("name".into(), Value::Char('.')),
                    ("age".into(), Value::Float(0.0)),
                ]),
            },
        ),
        Instruction::Field("height".into(), Value::Float(0.0)),
        Instruction::EndStruct,
        //
        Instruction::Func("main".into()),
        Instruction::NewStruct("Wrapper".into()),
        Instruction::Store("w".into()),
        //
        Instruction::PushFloat(20.0),
        Instruction::NewStruct("Custom".into()),
        Instruction::SetField("age".into()),
        Instruction::Store("c".into()),
        Instruction::Load("c".into()),
        Instruction::Load("w".into()),
        Instruction::SetField("custom".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];
    let mut vm = VM::new(bytecode);

    vm.run();
}

#[test]
fn test_recursion() {
    let bytecode = vec![
        Instruction::Func("factorial".into()), //20
        Instruction::Store("value".into()),    //
        Instruction::Load("value".into()),     //20
        Instruction::PushInt(1),               //1, 20
        Instruction::GreaterInt,               //1
        Instruction::JumpIfTrue(8),            //
        Instruction::PushInt(1),               // 1
        Instruction::Return,
        Instruction::Load("value".into()),     //20
        Instruction::Load("value".into()),     //20, 20
        Instruction::PushInt(1),               //1, 20, 20
        Instruction::SubInt,                   // 19, 20
        Instruction::Call("factorial".into()), //20
        Instruction::MulInt,                   //20 * x
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::PushInt(20),
        Instruction::Call("factorial".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];
    let mut vm = VM::new(bytecode);

    vm.run();
}

/*
#[test]
fn test_oom() {
    let bytecode = vec![
        Instruction::Func("counter".into()),
        Instruction::Store("value".into()),
        Instruction::Load("value".into()),
        Instruction::PushInt(1),
        Instruction::AddInt,
        Instruction::Println,
        Instruction::Store("value".into()),
        Instruction::Load("value".into()),
        Instruction::Call("counter".into()),
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::PushInt(0),
        Instruction::Call("counter".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];
    let mut vm = VM::new(bytecode);

    vm.run();
}
*/
