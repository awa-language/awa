use std::collections::HashMap;

use super::{instruction::Instruction, instruction::Value, VM};
#[test]
fn test_push_load_store() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushInt(42),
        Instruction::Println,
        Instruction::StoreInMap("x".into()),
        Instruction::LoadToStack("x".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in 1..1000 {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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
        Instruction::PushInt(25),
        Instruction::GreaterInt,
        Instruction::JumpIfTrue(16),
        Instruction::PushString("c is not greater than 25".into()),
        Instruction::Println,
        Instruction::Jump(18),
        Instruction::PushString("c is greater than 25".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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
    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
}

#[test]
fn test_slice_1d() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushArray(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
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

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
}

#[test]
fn test_complex() {
    let bytecode = vec![
        Instruction::Struct("Custom".into()),
        Instruction::Field("name".into(), Value::String("".into())),
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
        Instruction::PushString("Vasya".into()),
        Instruction::PushFloat(20.0),
        Instruction::NewStruct("Custom".into()),
        Instruction::SetField("age".into()),
        Instruction::SetField("name".into()),
        Instruction::StoreInMap("c".into()),
        Instruction::PushFloat(15.0),
        Instruction::LoadToStack("c".into()),
        Instruction::LoadToStack("w".into()),
        Instruction::SetField("custom".into()),
        Instruction::SetField("height".into()),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
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
    let mut vm = VM::new(bytecode.clone());
    for _i in 1..1000 {
        vm.run();
    }
}

/*
#[test]
fn test_oom() {
    let bytecode = vec![
        Instruction::Func("counter".into()),
        Instruction::StoreInMap("value".into()),
        Instruction::LoadToStack("value".into()),
        Instruction::PushInt(1),
        Instruction::AddInt,
        Instruction::Println,
        Instruction::StoreInMap("value".into()),
        Instruction::LoadToStack("value".into()),
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
    let mut vm = VM::new(bytecode.clone());
    for _i in 1..10000000 {
        vm.run();
    }

    vm.run();
}
*/

#[test]
fn test_gc_local_alloc_print() {
    let bytecode = vec![
        Instruction::Func("foo".into()),
        Instruction::PushArray(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        Instruction::StoreInMap("local_arr".into()),
        Instruction::Return,
        Instruction::EndFunc,
        Instruction::Func("main".into()),
        Instruction::Call("foo".into()),
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }

    println!("Heap before manual GC:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{i}] {obj:?}");
    }

    vm.gc.collect_garbage(&vm.stack, &vm.environments_stack);

    println!("Heap after manual GC:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{i}] {obj:?}");
    }
}

#[test]
fn test_gc_auto_trigger() {
    let bytecode = vec![
        Instruction::Func("creator".into()),
        Instruction::PushArray(vec![Value::Int(42)]),
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

    let mut vm = VM::new(bytecode.clone());
    vm.gc.threshold = 2;

    for _i in bytecode {
        vm.run();
    }

    println!("Heap after auto-run with threshold=2:");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{i}] {obj:?}");
    }
}

/*
#[test]
fn test_gc_recursion() {
    let mut big_vector = Vec::with_capacity(10_000);
    for i in 0..5 {
        big_vector.push(Value::Int(i as i64));
    }

    let bytecode = vec![
        Instruction::Func("rec".into()),
        Instruction::LoadToStack("n".into()),
        Instruction::PushArray(big_vector.clone()),
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

    let mut vm = VM::new(bytecode.clone());

    vm.gc.threshold = 5;

    for _i in bytecode {
        vm.run();
    }

    println!("=== After run() completed (deep recursion) ===");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }

    vm.gc.collect_garbage(&vm.stack, &vm.environments_stack);

    println!("=== After manual GC ===");
    for (i, obj) in vm.gc.heap.iter().enumerate() {
        println!("  [{}] {:?}", i, obj);
    }
}
*/

#[test]
fn test_hotswap() {
    let code = vec![
        // func rec
        Instruction::Func("rec".into()),
        Instruction::PushString("recur...".into()),
        Instruction::Println,
        Instruction::Call("rec".into()),
        Instruction::Return,
        Instruction::EndFunc,
        // func main
        Instruction::Func("main".into()),
        Instruction::Call("rec".into()),
        Instruction::Halt,
        Instruction::EndFunc,
    ];

    let mut vm = VM::new(code);

    for _i in 1..=100 {
        vm.run();
    }

    let new_code = vec![
        Instruction::Func("rec".into()),
        Instruction::PushString("hot_swap_e_boy".into()),
        Instruction::Println,
        Instruction::Return,
        Instruction::EndFunc,
    ];

    vm.hotswap_function(&new_code);

    for _i in 1..=100 {
        vm.run();
    }
}

#[test]
fn test_slice_2d() {
    let bytecode = vec![
        Instruction::Func("main".into()),
        Instruction::PushArray(vec![Value::Slice(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
        ])]),
        Instruction::Println,
        Instruction::Append(Value::Slice(vec![Value::Int(4)])),
        Instruction::Println,
        Instruction::StoreInMap("ab".into()),
        Instruction::PushArray(vec![Value::Int(5)]),
        Instruction::LoadToStack("ab".into()),
        Instruction::SetByIndex(1),
        Instruction::Println,
        Instruction::GetByIndex(0),
        Instruction::Println,
        Instruction::Halt,
        Instruction::EndFunc,
    ];
    let mut vm = VM::new(bytecode.clone());
    for _i in bytecode {
        vm.run();
    }
}
