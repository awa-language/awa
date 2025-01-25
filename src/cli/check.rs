use camino::Utf8PathBuf;

use crate::driver;

pub fn handle(filename: Option<Utf8PathBuf>) {
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

    let _ = driver::build_ast(filename, &input);
}
