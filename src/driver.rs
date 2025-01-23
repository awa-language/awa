use crate::{
    ast::{
        analyzer::analyze_input,
        definition::DefinitionTyped,
        module::{self, Module},
    },
    cli::{self, input::MenuAction},
    interpreter, vm,
};

#[derive(Debug)]
pub enum Command {
    OpenMenu,
}

pub enum BackwardsCommunication {
    Hotswapped,
    RequireHotswap,
}

// TODO: will take typed ast module as an argument
pub fn run(
    module: module::Typed,
    command_receiver: &std::sync::mpsc::Receiver<Command>,
    backwards_sender: &std::sync::mpsc::Sender<BackwardsCommunication>,
) {
    let bytecode = make_bytecode(&module);
    let mut vm = vm::VM::new(bytecode);
    let mut awaiting_hotswap = false;

    loop {
        if let Ok(command) = command_receiver.try_recv() {
            match command {
                Command::OpenMenu => {
                    let decision = cli::input::get_user_menu_decision();

                    match decision {
                        MenuAction::PerformHotswap => {
                            let user_input = cli::input::get_user_input();
                            dbg!(user_input.clone());

                            let module = build_ast(&user_input);
                            let hotswap_bytecode = make_bytecode(&module);

                            vm.hotswap_function(&hotswap_bytecode);

                            if awaiting_hotswap {
                                awaiting_hotswap = false;
                            }
                        }
                        MenuAction::ReturnToExecution => {}
                    }

                    let _ = backwards_sender
                        .send(BackwardsCommunication::Hotswapped)
                        .unwrap();
                }
            }
        }

        if !awaiting_hotswap {
            let backoff_message = vm.run();

            if let Some(backoff_message) = backoff_message {
                println!("recieved bacoff message: `{backoff_message}`. consider hotswapping");
                awaiting_hotswap = true;

                let _ = backwards_sender
                    .send(BackwardsCommunication::RequireHotswap)
                    .unwrap();
            }
        }
    }
}

pub fn build_ast(input: &str) -> Module<DefinitionTyped> {
    let typed_module = analyze_input(input);
    let module = match typed_module {
        Ok(module) => module,
        Err(_) => todo!(),
    };

    module
}

pub fn make_bytecode(module: &Module<DefinitionTyped>) -> Vec<vm::instruction::Instruction> {
    let interpreter = interpreter::Interpreter::new();

    interpreter.interpret_module(module)
}
