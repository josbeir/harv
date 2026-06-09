use color_eyre::eyre;
use ratatui::crossterm::event::EventStream;
use std::io;
use std::panic;

use ratatui::crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::ExecutableCommand;
use ratatui::prelude::*;

pub fn init() -> eyre::Result<()> {
    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        let _ = terminal::disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        panic_hook(panic);
    }));

    Ok(())
}

pub fn restore() -> eyre::Result<()> {
    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

pub fn terminal() -> eyre::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    Terminal::new(backend).map_err(Into::into)
}

pub fn event_stream() -> EventStream {
    EventStream::new()
}
