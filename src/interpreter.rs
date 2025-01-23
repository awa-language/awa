use crate::{cli, parse, vm};

#[derive(Debug)]
pub enum Command {
    OpenMenu,
}

// TODO: will take typed ast module as an argument
pub fn run(
    command_receiver: &std::sync::mpsc::Receiver<Command>,
    confirmation_sender: &std::sync::mpsc::Sender<()>,
) {
    let bytecode = Vec::new();

    let mut vm = vm::VM::new(bytecode);

    loop {
        if let Ok(command) = command_receiver.try_recv() {
            match command {
                Command::OpenMenu => {
                    let decision = cli::input::get_user_menu_decision();
                    match decision {
                        cli::input::MenuAction::PerformHotswap => {
                            let user_input = cli::input::get_user_input();
                            dbg!(user_input.clone());

                            let hotswap_bytecode = make_bytecode(&user_input);

                            vm.hotswap_function(&hotswap_bytecode);
                        }
                        cli::input::MenuAction::ReturnToExecution => {}
                    }

                    let () = confirmation_sender.send(()).unwrap();
                }
            }
        }

        vm.run();
    }
}

pub fn build_ast(input: &str) {
    let _module = parse::parse_module(input);

    // TODO: TAST+translation
    todo!();
}

pub fn make_bytecode(input: &str) -> Vec<vm::instruction::Instruction> {
    todo!();
}
