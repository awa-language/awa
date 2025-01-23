use awa::cli;
use camino::Utf8PathBuf;
use clap::{
    builder::{styling::AnsiColor, Styles},
    Parser,
};

#[derive(clap::Parser, Debug)]
#[command(
    version,
    next_display_order = None,
    help_template = "\
{before-help}{name} {version}

{usage-heading} {usage}

{all-args}{after-help}",
    styles = Styles::styled()
        .header(AnsiColor::Cyan.on_default())
        .usage(AnsiColor::Cyan.on_default())
        .literal(AnsiColor::BrightBlue.on_default())
)]
enum Command {
    /// Check the code.
    ///
    /// Performs lexing, parsing and translation from untyped to typed AST, thus
    /// identifying lexing, parsing and type mismatch errors.
    /// By default, checks `main.awa`
    Check { filename: Option<Utf8PathBuf> },

    /// Run the specified file in interactive environment.
    ///
    /// By default, runs `main.awa`
    Run { filename: Option<Utf8PathBuf> },
}

fn main() {
    cli::panic::add_handler();

    match Command::parse() {
        Command::Check { filename } => cli::check::handle(filename),
        Command::Run { filename } => cli::run::handle(filename),
    }
}
