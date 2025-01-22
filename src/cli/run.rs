use std::sync::mpsc::{channel, Receiver, Sender};

use camino::Utf8PathBuf;

use crate::interpreter::{self, Command};

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

    // let () = interpreter::build_ast(&_input);

    let (input_sender, input_reciever): (Sender<Command>, Receiver<Command>) = channel();
    let (confirmation_sender, confirmation_reciever): (Sender<()>, Receiver<()>) = channel();

    let _ = std::thread::spawn(move || {
        interpreter::run(input_reciever, confirmation_sender);
    });

    let term = console::Term::stdout();
    loop {
        let _ = term.read_char().unwrap();
        let _ = input_sender.send(Command::OpenMenu).unwrap();
        let _ = confirmation_reciever.recv();
    }
}
