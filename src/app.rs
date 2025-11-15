use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
pub enum AppStep {
    SelectType,
    EnterUrl,
    SelectFormat,
    Confirm,
    Downloading,
    Complete,
}

pub struct AppState {
    pub step: AppStep,
    pub download_type: Option<usize>,
    pub url: String,
    pub format: Option<usize>,
    pub status: String,
    pub list_state: ListState,
    pub input_mode: bool,
    pub download_progress: Arc<Mutex<DownloadProgress>>,
}

pub struct DownloadProgress {
    pub active: bool,
    pub message: String,
    pub spinner_index: usize,
}

impl AppState {
    pub fn new() -> Self {
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

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn get_current_options_len(&self) -> usize {
        match self.step {
            AppStep::SelectType => 3,
            AppStep::SelectFormat => {
                match self.download_type {
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
}