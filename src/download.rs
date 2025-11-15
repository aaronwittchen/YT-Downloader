use std::process::Command;
use std::sync::{Arc, Mutex};
use crate::app::DownloadProgress;

pub fn run_download_thread(
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
            configure_audio_download(&mut command, format, url, &progress);
        }
        0 => { // Video
            configure_video_download(&mut command, format, url, &progress);
        }
        2 => { // Subtitles
            configure_subtitle_download(&mut command, format, url, &progress);
        }
        _ => {}
    }

    let output = command.output();
    update_progress_with_result(output, download_type, progress);
}

fn configure_audio_download(
    command: &mut Command,
    format: usize,
    url: &str,
    progress: &Arc<Mutex<DownloadProgress>>,
) {
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

fn configure_video_download(
    command: &mut Command,
    format: usize,
    url: &str,
    progress: &Arc<Mutex<DownloadProgress>>,
) {
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

fn configure_subtitle_download(
    command: &mut Command,
    format: usize,
    url: &str,
    progress: &Arc<Mutex<DownloadProgress>>,
) {
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

fn update_progress_with_result(
    output: Result<std::process::Output, std::io::Error>,
    download_type: usize,
    progress: Arc<Mutex<DownloadProgress>>,
) {
    let mut prog = progress.lock().unwrap();
    prog.active = false;
    
    match output {
        Ok(output) => {
            if output.status.success() {
                if download_type == 2 {
                    // For subtitle downloads, always show success if the command succeeded
                    let success_msg = "Subtitles downloaded successfully!";
                    prog.message = format!("{} Press 'r' to restart or 'q' to quit", success_msg);
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