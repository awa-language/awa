use awa::cli;
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
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
)]
enum Command {
    /// Check the code.
    ///
    /// Performs lexing, parsing and translating from untyped to typed AST, thus
    /// identifying lexing, parsing and type mismatch errors.
    /// By default, checks `main.awa`
    Check,

    /// Run the specified file in interactive environment.
    ///
    /// By default, runs `main.awa`
    Run,
}

fn main() {
    cli::panic::add_handler();

    match Command::parse() {
        Command::Check => cli::check::handle(),
        Command::Run => cli::run::handle(),
    }
}
