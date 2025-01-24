use crate::{
    ast::{
        analyzer::TypeAnalyzer,
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
    Finished,
}

/// Create bytecode and run typed AST module in VM
///
/// # Panics
///
/// Will panic in case of failed backwards communication via mpsc
pub fn run(
    analyzer: &mut TypeAnalyzer,
    module: &module::Typed,
    command_receiver: &std::sync::mpsc::Receiver<Command>,
    backwards_sender: &std::sync::mpsc::Sender<BackwardsCommunication>,
) {
    let bytecode = make_bytecode(module);
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
                            let module = analyzer.handle_hotswap(&user_input);
                            let module = match module {
                                Ok(module) => module,
                                Err(err) => {
                                    let description = err.get_description();
                                    println!("{description}");
                                    continue;
                                }
                            };
                            let hotswap_bytecode = make_bytecode(&module);

                            vm.hotswap_function(&hotswap_bytecode);

                            if awaiting_hotswap {
                                awaiting_hotswap = false;
                            }
                        }
                        MenuAction::ReturnToExecution => {}
                    }

                    let () = backwards_sender
                        .send(BackwardsCommunication::Hotswapped)
                        .unwrap();
                }
            }
        }

        if !awaiting_hotswap {
            let backoff_message = vm.run();

            if let Some(backoff_message) = backoff_message {
                match backoff_message {
                    vm::RunCommunication::RequireHotswap(backoff_message) => {
                        println!(
                            "recieved bacoff message: `{backoff_message}`. consider hotswapping"
                        );
                        awaiting_hotswap = true;

                        let () = backwards_sender
                            .send(BackwardsCommunication::RequireHotswap)
                            .unwrap();
                    }
                    vm::RunCommunication::Finished => {
                        let () = backwards_sender
                            .send(BackwardsCommunication::Finished)
                            .unwrap();
                        std::process::exit(0); // TODO: FIXME
                    }
                }
            }
        }
    }
}

#[must_use]
pub fn build_ast(input: &str) -> (TypeAnalyzer, Module<DefinitionTyped>) {
    let mut analyzer = TypeAnalyzer::new();
    let typed_module = analyzer.analyze_input(input);

    match typed_module {
        Ok(module) => (analyzer, module),
        Err(err) => {
            let description = err.get_description();
            println!("{description}");
            std::process::exit(1); // TODO: FIXME
        }
    }
}

#[must_use]
pub fn make_bytecode(module: &Module<DefinitionTyped>) -> Vec<vm::instruction::Instruction> {
    let interpreter = interpreter::Interpreter::new();

    interpreter.interpret_module(module)
}
