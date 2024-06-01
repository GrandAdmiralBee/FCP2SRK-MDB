mod cli;
pub mod mdb_converter;
pub mod tui;

use color_eyre::config::HookBuilder;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use log::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    terminal::Terminal,
};
use tui_logger::*;

fn main() -> anyhow::Result<()> {
    let cli = crate::cli::cli().unwrap();
    let mdb_files = cli.mdb_files.clone();
    let cpp_files = cli.cpp_files.clone();

    for file in mdb_files {
        if !std::path::Path::new(&file).exists() {
            println!("Path {} does not exist", file);
            return Ok(());
        }
    }
    for file in cpp_files {
        if !std::path::Path::new(&file).exists() {
            println!("Path {} does not exist", file);
            return Ok(());
        }
    }
    // setup terminal
    init_error_hooks()?;
    let terminal = init_terminal()?;
    init_logger(LevelFilter::Trace).unwrap();
    set_default_level(LevelFilter::Trace);

    // create app and run it
    tui::App::new().run(terminal, cli)?;

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
