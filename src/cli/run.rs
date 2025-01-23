use std::sync::mpsc::{channel, Receiver, Sender};

use camino::Utf8PathBuf;

use crate::driver::{self, BackwardsCommunication, Command};

pub fn handle(filename: Option<Utf8PathBuf>) {
    let filename = match filename {
        Some(filename) => filename,
        None => "main.awa".into(),
    };

    let _input = std::fs::read_to_string(filename);
    let _input = match _input {
        Ok(input) => input,
        Err(_err) => todo!(),
    };

    let module = driver::build_ast(&_input);

    let (input_sender, input_reciever): (Sender<Command>, Receiver<Command>) = channel();
    // TODO: perhaps make it perform backwards communication - force hotswap on panics
    // NOTE: could be done via other user input taking logic, to notify user what to do before
    // opening editor
    let (backwards_sender, backwards_reciever): (
        Sender<BackwardsCommunication>,
        Receiver<BackwardsCommunication>,
    ) = channel();

    let _ = std::thread::spawn(move || {
        driver::run(module, &input_reciever, &backwards_sender);
    });

    let term = console::Term::stdout();

    let mut require_hotswap = false;
    loop {
        if let Ok(command) = backwards_reciever.try_recv() {
            match command {
                BackwardsCommunication::Hotswapped => unreachable!(),
                BackwardsCommunication::RequireHotswap => {
                    require_hotswap = true;
                }
            }
        }

        if !require_hotswap {
            let _ = term.read_char().unwrap();
        }

        let () = input_sender.send(Command::OpenMenu).unwrap();
        let confirmation = backwards_reciever.recv().unwrap();
        match confirmation {
            BackwardsCommunication::Hotswapped => {
                if require_hotswap {
                    require_hotswap = false;
                }
            }
            BackwardsCommunication::RequireHotswap => unreachable!(),
        }
    }
}
