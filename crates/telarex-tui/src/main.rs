//! Crate-level entry point for the TelaRex TUI application.
//!
//! Initializes logging, parses CLI args, sets up raw terminal mode and the
//! alternate screen, then runs the main event loop via [`App`].
#![allow(dead_code)]
mod app;
mod tui_compat;
mod components;
mod screens;
mod theme;
mod network;
mod events;
pub mod utils;

use app::App;
use clap::Parser;
use std::fs::OpenOptions;

use std::io;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tui_compat::{AppContext, Component, DrawContext};

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            use crossterm::event::PopKeyboardEnhancementFlags;
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[derive(Parser, Debug)]
#[command(name = "tr")]
struct Args {
    file: Option<String>,
    #[arg(short, long)]
    session: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Redirect logs to file to avoid breaking the TUI
    let mut log_path = std::env::current_exe()?;
    log_path.set_extension("log");
    
    // OPEN IN APPEND MODE to preserve history across runs
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    
    // FORCE LOG LEVEL for diagnostics
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    log::info!("--- TelaRex Session Started ---");

    let args = Args::parse();
    let mut app = App::new(args.file, args.session);
    let draw_ctx = DrawContext;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // Enable Keyboard Enhancement for Windows — must come AFTER raw mode and alternate screen
    #[cfg(windows)]
    {
        use crossterm::event::{PushKeyboardEnhancementFlags, KeyboardEnhancementFlags};
        let _ = crossterm::execute!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES |
                KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES |
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        );
    }

    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let _guard = TerminalGuard;
    let mut ctx = AppContext::new();

    while !ctx.should_quit() {
        terminal.draw(|frame| {
            let area = frame.area();
            app.draw(frame, area, &draw_ctx);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(5))? {
            let event = crossterm::event::read()?;
            app.handle_event(&event, &mut ctx);
        }
    }

    Ok(())
}
