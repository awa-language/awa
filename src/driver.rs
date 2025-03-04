use camino::Utf8PathBuf;

use crate::{
    ast::{
        analyzer::TypeAnalyzer,
        definition::DefinitionTyped,
        module::{self, Module},
    },
    cli::{self, input::MenuAction},
    error::Error,
    interpreter,
    parse::error::ConvertingErrorType::ParsingError,
    vm,
};

#[derive(Debug)]
pub enum Command {
    OpenMenu,
}

pub enum BackwardsCommunication {
    Hotswapped,
    ReturnedToExecution,
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
    unoptimized: bool,
    command_receiver: &std::sync::mpsc::Receiver<Command>,
    backwards_sender: &std::sync::mpsc::Sender<BackwardsCommunication>,
) {
    let bytecode = make_bytecode(module);
    let mut vm = vm::VM::new(bytecode, !unoptimized);
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
                                    print_diagnostics(
                                        "hotswap.awa".into(),
                                        user_input.into(),
                                        &err,
                                    );

                                    if awaiting_hotswap {
                                        awaiting_hotswap = false;
                                    }

                                    let () = backwards_sender
                                        .send(BackwardsCommunication::Hotswapped)
                                        .unwrap();

                                    continue;
                                }
                            };

                            let hotswap_bytecode = make_bytecode(&module);

                            vm.hotswap_function(&hotswap_bytecode);

                            if awaiting_hotswap {
                                awaiting_hotswap = false;
                            }

                            let () = backwards_sender
                                .send(BackwardsCommunication::Hotswapped)
                                .unwrap();
                        }
                        MenuAction::ReturnToExecution => {
                            let () = backwards_sender
                                .send(BackwardsCommunication::ReturnedToExecution)
                                .unwrap();
                        }
                        MenuAction::CtrlC => {
                            let () = backwards_sender
                                .send(BackwardsCommunication::Finished)
                                .unwrap();
                            println!("recieved SIGINT");

                            return;
                        }
                    }
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
                        return;
                    }
                }
            }
        }
    }
}

#[must_use]
pub fn build_ast(
    path: Utf8PathBuf,
    input: &str,
) -> Option<(TypeAnalyzer, Module<DefinitionTyped>)> {
    let mut analyzer = TypeAnalyzer::new();
    let typed_module = analyzer.analyze_input(input);

    match typed_module {
        Ok(module) => Some((analyzer, module)),
        Err(err) => {
            print_diagnostics(path, input.into(), &err);
            None
        }
    }
}

#[must_use]
pub fn make_bytecode(module: &Module<DefinitionTyped>) -> Vec<vm::instruction::Instruction> {
    let interpreter = interpreter::Interpreter::new();

    interpreter.interpret_module(module)
}

fn print_diagnostics(
    path: Utf8PathBuf,
    src: ecow::EcoString,
    converting_error: &crate::parse::error::ConvertingError,
) {
    let error = match converting_error.error {
        ParsingError { ref error } => Error::Parsing {
            path,
            src,
            error: error.clone(),
        },
        _ => Error::Ast {
            path,
            src,
            error: converting_error.clone(),
        },
    };

    let buffer_writer = termcolor::BufferWriter::stderr(termcolor::ColorChoice::Auto);
    let mut buffer = buffer_writer.buffer();

    let diagnostics = error.to_diagnostics();

    for diagnostic in diagnostics {
        diagnostic.write(&mut buffer);
    }

    buffer_writer.print(&buffer).unwrap();
}
