//! Terminal UI implementation using ratatui
//!
//! This module will be fully implemented in Phase 7.

mod app;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use app::{App, AppState};
use ui::draw;

/// Run the TUI application
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.state {
                AppState::Menu => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('s') | KeyCode::Char('S') => app.state = AppState::Stamp,
                    KeyCode::Char('v') | KeyCode::Char('V') => app.state = AppState::Verify,
                    KeyCode::Char('w') | KeyCode::Char('W') => app.state = AppState::Wallet,
                    _ => {}
                },
                AppState::Stamp | AppState::Verify | AppState::Wallet => match key.code {
                    KeyCode::Esc => {
                        app.state = AppState::Menu;
                        app.input_buffer.clear();
                        app.result_message.clear();
                    }
                    _ => app.handle_input(key.code).await?,
                },
            }
        }

        // Background updates (e.g., wallet sync status)
        app.tick().await?;
    }
}
