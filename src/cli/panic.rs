#![allow(clippy::unwrap_used)]
use std::io::Write;
use std::panic::PanicInfo;
use termcolor::{Color, ColorSpec, WriteColor};

pub fn add_handler() {
    std::panic::set_hook(Box::new(move |info: &PanicInfo<'_>| {
        print_wrapped_panic(info)
    }));
}

fn print_wrapped_panic(info: &PanicInfo<'_>) {
    let message = match (
        info.payload().downcast_ref::<&str>(),
        info.payload().downcast_ref::<String>(),
    ) {
        (Some(message), _) => (*message).to_string(),
        (_, Some(message)) => message.to_string(),
        (None, None) => "unknown error".into(),
    };

    let location = match info.location() {
        None => "".into(),
        Some(location) => format!("{}:{}\n\t", location.file(), location.line()),
    };

    let buffer_writer = termcolor::BufferWriter::stderr(termcolor::ColorChoice::Auto);
    let mut buffer = buffer_writer.buffer();

    buffer
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)))
        .unwrap();
    write!(buffer, "error").unwrap();

    buffer.set_color(ColorSpec::new().set_bold(true)).unwrap();
    write!(buffer, ": awa interpreter crashed!\n\n").unwrap();

    buffer.set_color(&ColorSpec::new()).unwrap();
    writeln!(buffer, "Panic: {location}{message}").unwrap();
    buffer_writer.print(&buffer).unwrap();
}
