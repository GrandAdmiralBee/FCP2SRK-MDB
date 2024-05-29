mod cli;
mod mdb_converter;
mod tui;

use color_eyre::config::HookBuilder;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Modifier, Style, Stylize},
    terminal::Terminal,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

fn main() -> anyhow::Result<()> {
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;

    // create app and run it
    tui::App::new().run(terminal)?;

    restore_terminal()?;

    Ok(())
}
fn init_error_hooks() -> anyhow::Result<()> {
    let (panic, error) = HookBuilder::default().into_hooks();
    let panic = panic.into_panic_hook();
    let error = error.into_eyre_hook();
    color_eyre::eyre::set_hook(Box::new(move |e| {
        let _ = restore_terminal();
        error(e)
    }))?;
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        panic(info);
    }));
    Ok(())
}

fn init_terminal() -> anyhow::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    std::io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> anyhow::Result<()> {
    disable_raw_mode()?;
    std::io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
