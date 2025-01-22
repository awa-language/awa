use camino::Utf8PathBuf;

use crate::interpreter;

use super::input::get_user_input;

pub fn handle(filename: Option<Utf8PathBuf>) {
    let filename = match filename {
        Some(filename) => filename,
        None => "main.awa".into(),
    };

    let input = std::fs::read_to_string(filename);
    let input = match input {
        Ok(input) => input,
        Err(_err) => todo!(),
    };

    let user_input = get_user_input();
    dbg!(user_input);

    interpreter::build_ast(&input);
    // TODO: add beautiful error wrapping everywhere
}
