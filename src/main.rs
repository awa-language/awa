use awa::cli;
use clap::builder::{styling::AnsiColor, Styles};

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
    Check,
    Run,
}

fn main() {
    cli::panic::add_handler();
    println!("Hello, world!");
    panic!("some panic text");
}
