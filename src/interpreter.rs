use crate::{cli, parse};

#[derive(Debug)]
pub enum Command {
    OpenMenu,
}

// TODO: will take typed ast module as an argument
pub fn run(
    command_receiver: std::sync::mpsc::Receiver<Command>,
    confirmation_sender: std::sync::mpsc::Sender<()>,
) {
    // TODO: vm::new()

    loop {
        if let Ok(command) = command_receiver.try_recv() {
            match command {
                Command::OpenMenu => {
                    let decision = cli::input::get_user_menu_decision();
                    match decision {
                        cli::input::MenuAction::PerformHotswap => {
                            let user_input = cli::input::get_user_input();
                            dbg!(user_input);

                            // TODO: vm::perform_hotswap
                        }
                        cli::input::MenuAction::ReturnToExecution => {}
                    }

                    let _ = confirmation_sender.send(()).unwrap();
                }
            }
        }
        // TODO: vm::run()
    }
}

pub fn build_ast(input: &str) {
    let _module = parse::parse_module(input);

    // TODO: TAST+translation
    todo!();
}
