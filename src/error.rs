use camino::Utf8PathBuf;
use ecow::EcoString;
use std::io::Write;
use termcolor::Buffer;

use crate::{
    diagnostic::{Diagnostic, Location},
    parse::error::{ConvertingError, ParsingError},
};

pub enum Error {
    Parsing {
        src: EcoString,
        path: Utf8PathBuf,
        error: ParsingError,
    },
    Ast {
        src: EcoString,
        path: Utf8PathBuf,
        error: ConvertingError,
    },
}

impl Error {
    /// Converts the error into a human-readable string.
    ///
    /// # Panics
    ///
    /// This function will panic if the buffer contains invalid UTF-8 bytes.
    #[must_use]
    pub fn to_pretty_string(&self) -> String {
        let mut buffer = Buffer::no_color();
        self.prettify(&mut buffer);

        String::from_utf8(buffer.into_inner()).unwrap()
    }

    pub fn prettify(&self, buffer: &mut Buffer) {
        for diagnostic in self.to_diagnostics() {
            diagnostic.write(buffer);
            writeln!(buffer).unwrap();
        }
    }

    #[must_use]
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
            Error::Ast { src, path, error } => {
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
