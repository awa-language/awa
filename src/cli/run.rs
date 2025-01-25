use std::sync::mpsc::{channel, Receiver, Sender};

use camino::Utf8PathBuf;

use crate::driver::{self, BackwardsCommunication, Command};

/// Handle run cli command - read from provided filename and run VM with interpreted bytecode
///
/// # Panics
///
/// Will panic if file does not exist, or in case of unexpected internal errors
pub fn handle(filename: Option<Utf8PathBuf>) {
    let filename = match filename {
        Some(filename) => filename,
        None => "main.awa".into(),
    };

    let input = std::fs::read_to_string(filename);
    let input = match input {
        Ok(input) => input,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let Some((mut analyzer, module)) = driver::build_ast(&input) else {
        return;
    };

    let (input_sender, input_reciever): (Sender<Command>, Receiver<Command>) = channel();
    let (backwards_sender, backwards_reciever): (
        Sender<BackwardsCommunication>,
        Receiver<BackwardsCommunication>,
    ) = channel();

    let _ = std::thread::spawn(move || {
        let _ = driver::run(&mut analyzer, &module, &input_reciever, &backwards_sender);
        return;
    });

    let term = console::Term::stdout();
    let mut require_hotswap = false;

    let (keypress_sender, keypress_reciever) = channel();
    let term_clone = term.clone();

    std::thread::spawn(move || loop {
        if let Ok(_) = term_clone.read_char() {
            let _ = keypress_sender.send(Some(()));
        } else {
            let _ = keypress_sender.send(None);
        }
    });

    loop {
        if let Ok(command) = backwards_reciever.try_recv() {
            match command {
                BackwardsCommunication::Hotswapped => {
                    if require_hotswap {
                        require_hotswap = false;
                    }
                }
                BackwardsCommunication::RequireHotswap => {
                    require_hotswap = true;
                }
                BackwardsCommunication::Finished => return,
            }
        }

        if let Ok(keypress) = keypress_reciever.try_recv() {
            match keypress {
                Some(_) => {
                    if !require_hotswap {
                        let () = input_sender.send(Command::OpenMenu).unwrap();
                    }

                    let confirmation = backwards_reciever.recv().unwrap();
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
                    }
                }
                None => {
                    // NOTE: only happens when there is no terminal, i.e. in CI
                    let confirmation = backwards_reciever.recv().unwrap();
                    match confirmation {
                        BackwardsCommunication::Hotswapped => {
                            unreachable!();
                        }
                        BackwardsCommunication::RequireHotswap => unreachable!(),
                        BackwardsCommunication::Finished => return,
                    }
                }
            }
        }
    }
}
