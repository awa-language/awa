use std::collections::HashMap;

use super::{instruction::Instruction, instruction::Value, VM};

#[test]
fn test_push_load_store() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushInt(42),
        Instruction::StoreInMap("x".into()),
        Instruction::LoadToStack("x".into()),
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
        Instruction::AddInt,
        Instruction::Println,
        Instruction::PushInt(3),
        Instruction::SubInt,
        Instruction::Println,
        Instruction::PushInt(4),
        Instruction::MulInt,
        Instruction::Println,
        Instruction::PushInt(6),
        Instruction::DivInt,
        Instruction::Println,
        Instruction::PushInt(3),
        Instruction::Mod,
        Instruction::Println,
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
        Instruction::PushFloat(10.0),
        Instruction::PushFloat(2.5),
        Instruction::AddFloat,
        Instruction::Println,
        Instruction::PushFloat(5.0),
        Instruction::SubFloat,
        Instruction::Println,
        Instruction::PushFloat(3.0),
        Instruction::MulFloat,
        Instruction::Println,
        Instruction::PushFloat(4.0),
        Instruction::DivFloat,
        Instruction::Println,
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
        Instruction::PushInt(10),
        Instruction::StoreInMap("a".into()),
        Instruction::PushInt(20),
        Instruction::StoreInMap("b".into()),
        Instruction::LoadToStack("a".into()),
        Instruction::LoadToStack("b".into()),
        Instruction::Equal,
        Instruction::Println,
        Instruction::LoadToStack("a".into()),
        Instruction::LoadToStack("b".into()),
        Instruction::NotEqual,
        Instruction::Println,
        Instruction::LoadToStack("a".into()),
        Instruction::LoadToStack("b".into()),
        Instruction::LessInt,
        Instruction::Println,
        Instruction::LoadToStack("a".into()),
        Instruction::PushInt(10),
        Instruction::LessEqualInt,
        Instruction::Println,
        Instruction::LoadToStack("b".into()),
        Instruction::PushInt(15),
        Instruction::GreaterInt,
        Instruction::Println,
        Instruction::LoadToStack("b".into()),
        Instruction::PushInt(25),
        Instruction::GreaterEqualInt,
        Instruction::Println,
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
        Instruction::PushInt(1),
        Instruction::JumpIfTrue(5),
        Instruction::PushInt(999),
        Instruction::Println,
        Instruction::PushInt(2),
        Instruction::Println,
        Instruction::Jump(10),
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
        Instruction::PushInt(10),
        Instruction::StoreInMap("a".into()),
        Instruction::PushInt(20),
        Instruction::StoreInMap("b".into()),
        Instruction::LoadToStack("a".into()),
        Instruction::LoadToStack("b".into()),
        Instruction::AddInt,
        Instruction::StoreInMap("c".into()),
        Instruction::LoadToStack("c".into()),
        Instruction::PushInt(250),
        Instruction::GreaterInt,
        Instruction::JumpIfTrue(16),
        Instruction::PushString("c is greater than 25".into()),
        Instruction::Println,
        Instruction::Jump(18),
        Instruction::PushString("c is not greater than 25".into()),
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
        Instruction::Func("add".into()),
        Instruction::StoreInMap("a".into()),
        Instruction::StoreInMap("b".into()),
        Instruction::LoadToStack("a".into()),
        Instruction::LoadToStack("b".into()),
        Instruction::AddInt,
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::PushInt(5),
        Instruction::PushInt(7),
        Instruction::Call("add".into()),
        Instruction::StoreInMap("c".into()),
        Instruction::LoadToStack("c".into()),
        Instruction::Println,
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
        Instruction::StoreInMap("a".into()),
        Instruction::PushString("nikitka".into()),
        Instruction::LoadToStack("a".into()),
        Instruction::SetField("name".into()),
        Instruction::StoreInMap("a".into()),
        Instruction::PushInt(22),
        Instruction::LoadToStack("a".into()),
        Instruction::SetField("age".into()),
        Instruction::StoreInMap("a".into()),
        Instruction::LoadToStack("a".into()),
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
        Instruction::PushString("Hello, ".into()),
        Instruction::PushString("World!".into()),
        Instruction::Concat,
        Instruction::Println,
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
        Instruction::StoreInMap("qwe".into()),
        Instruction::PushInt(22),
        Instruction::LoadToStack("qwe".into()),
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
        Instruction::Func("main".into()),
        Instruction::NewStruct("Wrapper".into()),
        Instruction::StoreInMap("w".into()),
        Instruction::PushFloat(20.0),
        Instruction::NewStruct("Custom".into()),
        Instruction::SetField("age".into()),
        Instruction::StoreInMap("c".into()),
        Instruction::LoadToStack("c".into()),
        Instruction::LoadToStack("w".into()),
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
        Instruction::Func("factorial".into()),
        Instruction::StoreInMap("value".into()),
        Instruction::LoadToStack("value".into()),
        Instruction::PushInt(1),
        Instruction::GreaterInt,
        Instruction::JumpIfTrue(8),
        Instruction::PushInt(1),
        Instruction::Return,
        Instruction::LoadToStack("value".into()),
        Instruction::LoadToStack("value".into()),
        Instruction::PushInt(1),
        Instruction::SubInt,
        Instruction::Call("factorial".into()),
        Instruction::MulInt,
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

#[test]
fn test_gc_local_alloc_print() {
    let code = vec![
        Instruction::Func("foo".into()),
        Instruction::PushSlice(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        Instruction::StoreInMap("local_arr".into()),
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::Call("foo".into()),
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(code);
    vm.run();

    println!("Heap before manual GC:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }

    vm.gc
        .collect_garbage(&vm.stack, &vm.environments_stack, &vm.global_variables);

    println!("Heap after manual GC:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }
}

#[test]
fn test_gc_auto_trigger() {
    let code = vec![
        Instruction::Func("creator".into()),
        Instruction::PushSlice(vec![Value::Int(42)]),
        Instruction::StoreInMap("temp_arr".into()),
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::Call("creator".into()),
        Instruction::Call("creator".into()),
        Instruction::Call("creator".into()),
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(code);

    vm.gc.threshold = 2;

    vm.run();

    println!("Heap after auto-run with threshold=2:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }
}

/*
#[test]
fn test_gc_recursion() {
    let mut big_vector = Vec::with_capacity(10_000);
    for i in 0..10_000 {
        big_vector.push(Value::Int(i as i64));
    }

    let mut code = vec![
        Instruction::Func("rec".into()),
        Instruction::LoadToStack("n".into()),
        Instruction::PushSlice(big_vector.clone()),
        Instruction::StoreInMap("big_arr".into()),
        Instruction::LoadToStack("n".into()),
        Instruction::PushInt(1),
        Instruction::SubInt,
        Instruction::StoreInMap("n".into()),
        Instruction::Call("rec".into()),
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::PushInt(20),
        Instruction::StoreInMap("n".into()),
        Instruction::Call("rec".into()),
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(code);

    vm.gc.threshold = 5;

    vm.run();

    println!("=== After run() completed (deep recursion) ===");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }

    vm.gc
        .collect_garbage(&vm.stack, &vm.environments_stack, &vm.global_variables);

    println!("=== After manual GC ===");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }
}
*/
