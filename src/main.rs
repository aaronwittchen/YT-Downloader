mod app;
mod ui;
mod download;
mod handlers;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::error::Error;
use std::io;

use app::{AppState, AppStep};
use ui::render_ui;
use handlers::handle_key_event;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let base_dir = std::env::current_dir()?;
    let setup_dir = base_dir.join("setup");
    let output_dir = base_dir.join("output");
    let ytdlp_path = setup_dir.join("yt-dlp.exe");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
    }

    let result = run_app(&mut terminal, ytdlp_path, output_dir);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ytdlp_path: std::path::PathBuf,
    output_dir: std::path::PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut app = AppState::new();

    loop {
        terminal.draw(|f| render_ui(f, &app))?;
        check_download_status(&mut app);
        update_spinner(&mut app);

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }

                let should_quit = handle_key_event(&mut app, key.code, &ytdlp_path, &output_dir)?;
                if should_quit {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn check_download_status(app: &mut AppState) {
    if app.step == AppStep::Downloading {
        if let Ok(progress) = app.download_progress.lock() {
            if !progress.active && !progress.message.is_empty() {
                app.step = AppStep::Complete;
                app.status = progress.message.clone();
            }
        }
    }
}

fn update_spinner(app: &mut AppState) {
    if app.step == AppStep::Downloading {
        if let Ok(mut progress) = app.download_progress.lock() {
            if progress.active {
                progress.spinner_index = (progress.spinner_index + 1) % 8;
            }
        }
    }
}
