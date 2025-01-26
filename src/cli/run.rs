use camino::Utf8PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use termion::input::TermRead;

use crate::driver::{self, BackwardsCommunication, Command};

/// Handle run cli command - read from provided filename and run VM with interpreted bytecode
///
/// # Panics
///
/// Will panic if file does not exist, or in case of unexpected internal errors
pub fn handle(filename: Option<Utf8PathBuf>, unoptimized: bool) {
    let filename = match filename {
        Some(filename) => filename,
        None => "main.awa".into(),
    };

    let input = std::fs::read_to_string(filename.clone());
    let input = match input {
        Ok(input) => input,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let Some((mut analyzer, module)) = driver::build_ast(filename, &input) else {
        return;
    };

    let (driver_sender, driver_reciever): (Sender<Command>, Receiver<Command>) = channel();
    let (driver_backwards_sender, driver_backwards_reciever): (
        Sender<BackwardsCommunication>,
        Receiver<BackwardsCommunication>,
    ) = channel();

    let _ = std::thread::spawn(move || {
        driver::run(
            &mut analyzer,
            &module,
            unoptimized,
            &driver_reciever,
            &driver_backwards_sender,
        );
    });

    let mut require_hotswap = false;

    let (keypress_sender, keypress_reciever) = channel();
    let (keypress_backwards_sender, keypress_backwards_reciever) = channel();

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut keys = stdin.keys();

        // NOTE: it is blocking, sleeping won't optimize CPU usage
        loop {
            if let Some(Ok(_key)) = keys.next() {
                let _ = keypress_sender.send(Some(()));
            }

            if keypress_backwards_reciever.recv().is_err() {
                return;
            }
        }
    });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));

        if let Ok(command) = driver_backwards_reciever.try_recv() {
            match command {
                BackwardsCommunication::Hotswapped
                | BackwardsCommunication::ReturnedToExecution => {
                    unreachable!()
                }
                BackwardsCommunication::RequireHotswap => {
                    require_hotswap = true;
                }
                BackwardsCommunication::Finished => return,
            }
        }

        if let Ok(Some(())) = keypress_reciever.try_recv() {
            driver_sender.send(Command::OpenMenu).unwrap();

            let confirmation = driver_backwards_reciever.recv().unwrap();

            match confirmation {
                BackwardsCommunication::Hotswapped => {
                    if require_hotswap {
                        require_hotswap = false;
                    }
                }
                BackwardsCommunication::RequireHotswap => {
                    require_hotswap = true;
                }
                BackwardsCommunication::Finished => return,
                BackwardsCommunication::ReturnedToExecution => {}
            }

            keypress_backwards_sender.send(()).unwrap();
        }
    }
}
