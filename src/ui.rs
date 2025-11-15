use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Frame,
};
use crate::app::{AppState, AppStep};

pub fn render_ui(f: &mut Frame, app: &AppState) {
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

    render_title(f, chunks[0]);
    render_info(f, app, chunks[1]);
    render_main_content(f, app, chunks[2]);
    render_help(f, app, chunks[3]);
}

fn render_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("YouTube Downloader")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn render_info(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let type_str = match app.download_type {
        Some(0) => "Video",
        Some(1) => "Audio",
        Some(2) => "Subtitles",
        Some(_) => "Unknown",
        None => "Not selected",
    };

    let format_str = get_format_string(app);
    let url_display = get_url_display(&app.url);

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
    f.render_widget(info, area);
}

fn get_format_string(app: &AppState) -> &'static str {
    if let Some(dtype) = app.download_type {
        if let Some(fmt) = app.format {
            return match dtype {
                0 => ["MP4", "MKV", "WebM"][fmt],
                1 => ["FLAC", "MP3", "WAV", "AAC", "M4A"][fmt],
                2 => ["English", "All"][fmt],
                _ => "Unknown",
            };
        }
    }
    "Not selected"
}

fn get_url_display(url: &str) -> &str {
    if url.is_empty() {
        "Not entered"
    } else if url.len() > 50 {
        &url[..50]
    } else {
        url
    }
}

fn render_main_content(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    match app.step {
        AppStep::SelectType => render_select_type(f, app, area),
        AppStep::EnterUrl => render_url_input(f, app, area),
        AppStep::SelectFormat => render_select_format(f, app, area),
        AppStep::Confirm => render_confirm(f, app, area),
        AppStep::Downloading => render_downloading(f, app, area),
        AppStep::Complete => render_complete(f, app, area),
    }
}

fn render_select_type(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = vec!["Video", "Audio", "Subtitles"]
        .iter()
        .map(|s| ListItem::new(*s))
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select Download Type"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(list, area, &mut app.list_state.clone());
}

fn render_url_input(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
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
    f.render_widget(paragraph, area);
}

fn render_select_format(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
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
    
    f.render_stateful_widget(list, area, &mut app.list_state.clone());
}

fn render_confirm(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let items = vec![
        ListItem::new("Start Download"),
        ListItem::new("Cancel"),
    ];
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Confirm"))
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    
    f.render_stateful_widget(list, area, &mut app.list_state.clone());
}

fn render_downloading(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
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
    f.render_widget(paragraph, area);
}

fn render_complete(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let success = app.status.to_lowercase().contains("complete") || 
                 app.status.to_lowercase().contains("success");
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
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
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
    f.render_widget(help, area);
}