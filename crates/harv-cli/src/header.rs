use std::io::{stdout, Write};

const HEADER_RAW: &str = "\
▗▖ ▗▖ ▗▄▖ ▗▄▄▖ ▗▖  ▗▖\n\
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌\n\
▐▛▀▜▌▐▛▀▜▌▐▛▀▚▖▐▌  ▐▌\n\
▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌ ▝▚▞▘ \n";

/// Print the header instantly to stdout.
pub fn show() {
    let shades: &[(u8, u8, u8)] = &[
        (250, 210, 140),
        (250, 170, 90),
        (250, 130, 40),
        (250, 93, 0),
    ];

    for (i, line) in HEADER_RAW.lines().enumerate() {
        let (r, g, b) = shades[i];
        println!("\x1b[38;2;{};{};{}m{}", r, g, b, line);
    }
    let version = env!("CARGO_PKG_VERSION");
    let text = format!("HARV CLI v{}", version);
    let pad = (21usize.saturating_sub(text.len())) / 2;
    println!(
        "{}\x1b[38;2;250;93;0mHARV CLI\x1b[38;2;160;160;160m v{}\x1b[0m\n",
        " ".repeat(pad),
        version
    );
    let _ = stdout().flush();
}
