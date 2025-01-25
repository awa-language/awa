use std::collections::HashMap;

use camino::Utf8PathBuf;
use codespan_reporting::diagnostic::Severity;
use ecow::EcoString;
use termcolor::Buffer;

use crate::ast::location::Location as AstLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub src: EcoString,
    pub path: Utf8PathBuf,
    pub location: AstLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub location: Location,
    pub text: String,
}

impl Diagnostic {
    pub fn write(&self, buffer: &mut Buffer) {
        self.write_location(buffer);
    }

    fn write_location(&self, buffer: &mut Buffer) {
        let mut file_map = HashMap::new();
        let mut files = codespan_reporting::files::SimpleFiles::new();

        let main_location_path = self.location.path.as_str();
        let main_location_src = self.location.src.as_str();

        let main_file_id = files.add(main_location_path, main_location_src);
        let _ = file_map.insert(main_location_path, main_file_id);

        let labels = vec![codespan_reporting::diagnostic::Label {
            style: codespan_reporting::diagnostic::LabelStyle::Primary,
            file_id: main_file_id,
            range: (self.location.location.start as usize)..(self.location.location.end as usize),
            message: self.text.clone(),
        }];

        let diagnostic = codespan_reporting::diagnostic::Diagnostic::new(Severity::Error)
            .with_labels(labels)
            .with_message(&self.text);
        let config = codespan_reporting::term::Config::default();

        codespan_reporting::term::emit(buffer, &config, &files, &diagnostic).unwrap();
    }
}
