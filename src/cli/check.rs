use camino::Utf8PathBuf;

use crate::driver;

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

    driver::build_ast(&input);
    // TODO: add beautiful error wrapping everywhere
}
