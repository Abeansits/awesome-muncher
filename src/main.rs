mod app;
mod config;
mod db;
mod fetch;
mod models;
mod ui;

use std::io;
use std::time::Duration;

use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::App;
use crate::config::Config;
use crate::db::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup error handling
    color_eyre::install()?;

    // Load config
    let config = Config::load()?;

    // Open database
    let db_path = Config::database_path()?;
    let db = Database::open(&db_path)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(config, db);
    app.init()?;

    // Check if we have any data
    let project_count = app.db.get_project_count()?;
    if project_count == 0 {
        // First run - start sync automatically
        app.start_sync()?;
    }

    // Run the main loop
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
    let tick_rate = Duration::from_millis(100);

    loop {
        // Draw UI
        terminal.draw(|frame| {
            app.screen.render(frame);
        })?;

        // Handle events with timeout for animation ticks
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        // Process background messages
        app.process_messages()?;

        // Animation tick
        app.tick();

        // Check if we should quit
        if app.should_quit {
            return Ok(());
        }
    }
}
