use crossterm::event::KeyCode;
use std::error::Error;
use std::thread;
use crate::app::{AppState, AppStep};
use crate::download::run_download_thread;

pub fn handle_key_event(
    app: &mut AppState,
    key_code: KeyCode,
    ytdlp_path: &std::path::PathBuf,
    output_dir: &std::path::PathBuf,
) -> Result<bool, Box<dyn Error>> {
    match key_code {
        KeyCode::Esc => return Ok(true), // Signal to quit
        KeyCode::Char('q') if !app.input_mode => return Ok(true),
        KeyCode::Char(c) if app.input_mode && app.step == AppStep::EnterUrl => {
            app.url.push(c);
        }
        KeyCode::Backspace if app.input_mode && app.step == AppStep::EnterUrl => {
            app.url.pop();
        }
        KeyCode::Enter => {
            handle_enter(app, ytdlp_path, output_dir)?;
        }
        KeyCode::Up if !app.input_mode => {
            move_selection_up(app);
        }
        KeyCode::Down if !app.input_mode => {
            move_selection_down(app);
        }
        KeyCode::Char('r') if app.step == AppStep::Complete => {
            app.reset();
        }
        _ => {}
    }
    Ok(false) // Don't quit
}

fn move_selection_up(app: &mut AppState) {
    if let Some(selected) = app.list_state.selected() {
        let options_len = app.get_current_options_len();
        if options_len > 0 {
            let new_selected = if selected > 0 { selected - 1 } else { options_len - 1 };
            app.list_state.select(Some(new_selected));
        }
    }
}

fn move_selection_down(app: &mut AppState) {
    if let Some(selected) = app.list_state.selected() {
        let options_len = app.get_current_options_len();
        if options_len > 0 {
            let new_selected = if selected < options_len - 1 { selected + 1 } else { 0 };
            app.list_state.select(Some(new_selected));
        }
    }
}

fn handle_enter(
    app: &mut AppState,
    ytdlp_path: &std::path::PathBuf,
    output_dir: &std::path::PathBuf,
) -> Result<(), Box<dyn Error>> {
    match app.step {
        AppStep::SelectType => {
            app.download_type = app.list_state.selected();
            let type_name = match app.download_type {
                Some(0) => "Video",
                Some(1) => "Audio",
                Some(2) => "Subtitles",
                _ => "Unknown",
            };
            app.status = format!("{} selected. Enter YouTube URL", type_name);
            app.step = AppStep::EnterUrl;
            app.input_mode = true;
        }
        AppStep::EnterUrl => {
            if !app.url.is_empty() {
                app.input_mode = false;
                app.status = "Select output format using arrow keys".to_string();
                app.step = AppStep::SelectFormat;
                app.list_state.select(Some(0));
            }
        }
        AppStep::SelectFormat => {
            app.format = app.list_state.selected();
            app.status = "Press Enter to start download, or 'q' to cancel".to_string();
            app.step = AppStep::Confirm;
            app.list_state.select(Some(0));
        }
        AppStep::Confirm => {
            if let Some(selected) = app.list_state.selected() {
                if selected == 0 {
                    start_download(app, ytdlp_path, output_dir);
                } else {
                    app.reset();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn start_download(
    app: &mut AppState,
    ytdlp_path: &std::path::PathBuf,
    output_dir: &std::path::PathBuf,
) {
    app.step = AppStep::Downloading;
    app.status = "Downloading... Please wait".to_string();
    
    {
        let mut progress = app.download_progress.lock().unwrap();
        progress.active = true;
        progress.message = "Initializing download...".to_string();
    }
    
    let ytdlp_path = ytdlp_path.clone();
    let output_dir = output_dir.clone();
    let download_type = app.download_type.unwrap();
    let format = app.format.unwrap();
    let url = app.url.clone();
    let progress_clone = app.download_progress.clone();
    
    thread::spawn(move || {
        run_download_thread(download_type, format, &url, &ytdlp_path, &output_dir, progress_clone)
    });
}