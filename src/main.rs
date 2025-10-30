use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem, ListState},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::error::Error;
use std::io;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(PartialEq)]
enum AppStep {
    SelectType,
    EnterUrl,
    SelectFormat,
    Confirm,
    Downloading,
    Complete,
}

struct AppState {
    step: AppStep,
    download_type: Option<usize>,
    url: String,
    format: Option<usize>,
    status: String,
    list_state: ListState,
    input_mode: bool,
    download_progress: Arc<Mutex<DownloadProgress>>,
}

struct DownloadProgress {
    active: bool,
    message: String,
    spinner_index: usize,
}

impl AppState {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        AppState {
            step: AppStep::SelectType,
            download_type: None,
            url: String::new(),
            format: None,
            status: "Select download type using arrow keys and Enter".to_string(),
            list_state,
            input_mode: false,
            download_progress: Arc::new(Mutex::new(DownloadProgress {
                active: false,
                message: String::new(),
                spinner_index: 0,
            })),
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

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

    let mut app = AppState::new();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Check if download completed
        if app.step == AppStep::Downloading {
            if let Ok(progress) = app.download_progress.lock() {
                if !progress.active && !progress.message.is_empty() {
                    // Download finished, check result
                    app.step = AppStep::Complete;
                    app.status = progress.message.clone();
                }
            }
        }

        // Update spinner when downloading
        if app.step == AppStep::Downloading {
            if let Ok(mut progress) = app.download_progress.lock() {
                if progress.active {
                    progress.spinner_index = (progress.spinner_index + 1) % 8;
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != crossterm::event::KeyEventKind::Press {
                    continue;
                }
                
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char('q') if !app.input_mode => break,
                    KeyCode::Char(c) if app.input_mode && app.step == AppStep::EnterUrl => {
                        app.url.push(c);
                    }
                    KeyCode::Backspace if app.input_mode && app.step == AppStep::EnterUrl => {
                        app.url.pop();
                    }
                    KeyCode::Enter => {
                        handle_enter(&mut app, &ytdlp_path, &output_dir)?;
                    }
                    KeyCode::Up if !app.input_mode => {
                        if let Some(selected) = app.list_state.selected() {
                            let options_len = get_current_options_len(&app);
                            if options_len > 0 {
                                let new_selected = if selected > 0 { selected - 1 } else { options_len - 1 };
                                app.list_state.select(Some(new_selected));
                            }
                        }
                    }
                    KeyCode::Down if !app.input_mode => {
                        if let Some(selected) = app.list_state.selected() {
                            let options_len = get_current_options_len(&app);
                            if options_len > 0 {
                                let new_selected = if selected < options_len - 1 { selected + 1 } else { 0 };
                                app.list_state.select(Some(new_selected));
                            }
                        }
                    }
                    KeyCode::Char('r') if app.step == AppStep::Complete => {
                        app.reset();
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn get_current_options_len(app: &AppState) -> usize {
    match app.step {
        AppStep::SelectType => 3,
        AppStep::SelectFormat => {
            match app.download_type {
                Some(0) => 3, // Video formats
                Some(1) => 5, // Audio formats
                Some(2) => 2, // Subtitle options
                _ => 0,
            }
        }
        AppStep::Confirm => 2,
        _ => 0,
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
                    app.step = AppStep::Downloading;
                    app.status = "Downloading... Please wait".to_string();
                    
                    // Start progress indicator
                    {
                        let mut progress = app.download_progress.lock().unwrap();
                        progress.active = true;
                        progress.message = "Initializing download...".to_string();
                    }
                    
                    // Launch download in background
                    let ytdlp_path = ytdlp_path.clone();
                    let output_dir = output_dir.clone();
                    let download_type = app.download_type.unwrap();
                    let format = app.format.unwrap();
                    let url = app.url.clone();
                    let progress_clone = app.download_progress.clone();
                    
                    thread::spawn(move || {
                        run_download_thread(download_type, format, &url, &ytdlp_path, &output_dir, progress_clone)
                    });
                } else {
                    app.reset();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn run_download_thread(
    download_type: usize,
    format: usize,
    url: &str,
    ytdlp_path: &std::path::PathBuf,
    output_dir: &std::path::PathBuf,
    progress: Arc<Mutex<DownloadProgress>>,
) {
    let mut command = Command::new(ytdlp_path);
    command.current_dir(output_dir);
    
    use std::process::Stdio;
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    match download_type {
        1 => { // Audio
            let audio_formats = ["flac", "mp3", "wav", "aac", "m4a"];
            command.args(&[
                "-f", "bestaudio/best",
                "-ciw",
                "-o", "%(title)s.%(ext)s",
                "--extract-audio",
                "--audio-format", audio_formats[format],
                url,
            ]);
            
            let mut prog = progress.lock().unwrap();
            prog.message = format!("Downloading audio in {} format...", audio_formats[format]);
        }
        0 => { // Video
            let video_formats = ["mp4", "mkv", "webm"];
            let format_str = match video_formats[format] {
                "mp4" => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                "mkv" => "bestvideo[ext=webm]+bestaudio/best[ext=mkv]/best",
                "webm" => "bestvideo[ext=webm]+bestaudio/best[ext=webm]/best",
                _ => "bestvideo+bestaudio/best",
            };

            command.args(&[
                "-f", format_str,
                "-ciw",
                "-o", "%(title)s.%(ext)s",
                url,
            ]);
            
            let mut prog = progress.lock().unwrap();
            prog.message = format!("Downloading video in {} format...", video_formats[format]);
        }
        2 => { // Subtitles
            command.args(&[
                "--skip-download",
                "--write-subs",
                "--write-auto-subs",
                "--sub-format", "srt",
                "-o", "%(title)s.%(ext)s",
                url,
            ]);

            if format == 0 {
                command.args(&["--sub-langs", "en"]);
            } else {
                command.args(&["--sub-langs", "all"]);
            }
            
            let mut prog = progress.lock().unwrap();
            prog.message = "Downloading subtitles...".to_string();
        }
        _ => {}
    }

    let output = command.output();
    
    // Update progress with result
    let mut prog = progress.lock().unwrap();
    prog.active = false;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if download_type == 2 {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("no subtitles") || stderr.contains("No subtitles") {
                        prog.message = "No subtitles available! Press 'r' to restart or 'q' to quit".to_string();
                    } else {
                        prog.message = "Download complete! Press 'r' to restart or 'q' to quit".to_string();
                    }
                } else {
                    prog.message = "Download complete! Press 'r' to restart or 'q' to quit".to_string();
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("no subtitles") || stderr.contains("No subtitles") {
                    prog.message = "No subtitles available! Press 'r' to restart or 'q' to quit".to_string();
                } else {
                    prog.message = "Download failed! Press 'r' to restart or 'q' to quit".to_string();
                }
            }
        }
        Err(_) => {
            prog.message = "Download failed! Press 'r' to restart or 'q' to quit".to_string();
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title = Paragraph::new("YouTube Downloader")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let type_str = match app.download_type {
        Some(0) => "Video",
        Some(1) => "Audio",
        Some(2) => "Subtitles",
        Some(_) => "Unknown",
        None => "Not selected",
    };

    let format_str = if let Some(dtype) = app.download_type {
        if let Some(fmt) = app.format {
            match dtype {
                0 => ["MP4", "MKV", "WebM"][fmt],
                1 => ["FLAC", "MP3", "WAV", "AAC", "M4A"][fmt],
                2 => ["English", "All"][fmt],
                _ => "Unknown",
            }
        } else {
            "Not selected"
        }
    } else {
        "Not selected"
    };

    let url_display = if app.url.is_empty() {
        "Not entered"
    } else if app.url.len() > 50 {
        &app.url[..50]
    } else {
        &app.url
    };

    let info_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Type:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(type_str, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  Format: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format_str, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("  URL:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(url_display, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status, Style::default().fg(Color::Cyan)),
        ]),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Information"));
    f.render_widget(info, chunks[1]);

    match app.step {
        AppStep::SelectType => {
            let items: Vec<ListItem> = vec!["Video", "Audio", "Subtitles"]
                .iter()
                .map(|s| ListItem::new(*s))
                .collect();
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Download Type"))
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            
            f.render_stateful_widget(list, chunks[2], &mut app.list_state.clone());
        }
        AppStep::EnterUrl => {
            let input_text = if app.url.is_empty() {
                vec![Line::from("Type your YouTube URL and press Enter...")]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled(&app.url, Style::default().fg(Color::Yellow))),
                ]
            };
            
            let paragraph = Paragraph::new(input_text)
                .block(Block::default().borders(Borders::ALL).title("Enter URL"));
            f.render_widget(paragraph, chunks[2]);
        }
        AppStep::SelectFormat => {
            let formats = match app.download_type {
                Some(0) => vec!["MP4", "MKV", "WebM"],
                Some(1) => vec!["FLAC", "MP3", "WAV", "AAC", "M4A"],
                Some(2) => vec!["English only", "All languages"],
                _ => vec![],
            };

            let items: Vec<ListItem> = formats.iter().map(|s| ListItem::new(*s)).collect();
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Select Format"))
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            
            f.render_stateful_widget(list, chunks[2], &mut app.list_state.clone());
        }
        AppStep::Confirm => {
            let items = vec![
                ListItem::new("Start Download"),
                ListItem::new("Cancel"),
            ];
            
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Confirm"))
                .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            
            f.render_stateful_widget(list, chunks[2], &mut app.list_state.clone());
        }
        AppStep::Downloading => {
            let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
            let (spinner, message) = {
                let progress = app.download_progress.lock().unwrap();
                let spinner = if progress.active {
                    spinner_frames[progress.spinner_index % spinner_frames.len()]
                } else {
                    "⠋"
                };
                (spinner.to_string(), progress.message.clone())
            };
            
            let text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("{} ", spinner),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        message,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from("This may take a while depending on file size..."),
                Line::from(""),
                Line::from(Span::styled(
                    "Please wait...",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Downloading"));
            f.render_widget(paragraph, chunks[2]);
        }
        AppStep::Complete => {
            let success = app.status.contains("complete");
            let color = if success { Color::Green } else { Color::Red };
            let symbol = if success { "[SUCCESS]" } else { "[FAILED]" };
            
            let text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    symbol,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(app.status.as_str()),
            ];
            
            let paragraph = Paragraph::new(text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Complete"));
            f.render_widget(paragraph, chunks[2]);
        }
    };

    let help_text = match app.step {
        AppStep::Complete => "Press 'r' to restart  |  Press 'q' to quit",
        AppStep::EnterUrl => "Type URL and press Enter",
        AppStep::Downloading => "Please wait...",
        _ => "Use Arrow Keys to navigate  |  Press Enter to select  |  Press 'q' to quit",
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[3]);
}