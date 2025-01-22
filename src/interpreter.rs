use crate::{cli, parse};

#[derive(Debug)]
pub enum Command {
    OpenMenu,
}

// TODO: will take typed ast module as an argument
pub fn run(receiver: std::sync::mpsc::Receiver<Command>, sender: std::sync::mpsc::Sender<()>) {
    // TODO: vm::new()

    loop {
        dbg!("looping");
        if let Ok(command) = receiver.try_recv() {
            dbg!(command);
            match command {
                Command::OpenMenu => {
                    let user_input = cli::input::get_user_input();
                    dbg!(user_input);
                    let _ = sender.send(()).unwrap();
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
