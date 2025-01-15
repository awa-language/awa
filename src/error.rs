use camino::Utf8PathBuf;
use ecow::EcoString;
use std::io::Write;
use termcolor::Buffer;

use crate::{
    diagnostic::{Diagnostic, Location},
    parse::error::ParsingError,
};

pub enum Error {
    Parsing {
        src: EcoString,
        path: Utf8PathBuf,
        error: ParsingError,
    },
}

impl Error {
    pub fn pretty_string(&self) -> String {
        let mut nocolor = Buffer::no_color();
        self.pretty(&mut nocolor);
        String::from_utf8(nocolor.into_inner()).unwrap()
    }

    pub fn pretty(&self, buffer: &mut Buffer) {
        for diagnostic in self.to_diagnostics() {
            diagnostic.write(buffer);
            writeln!(buffer).unwrap();
        }
    }

    pub fn to_diagnostics(&self) -> Vec<Diagnostic> {
        match self {
            Error::Parsing { src, path, error } => {
                vec![Diagnostic {
                    text: error.get_description(),
                    location: Location {
                        src: src.clone(),
                        path: path.clone(),
                        location: crate::ast::location::Location {
                            start: error.location.start,
                            end: error.location.end,
                        },
                    },
                }]
            }
        }
    }
}
