use dialoguer::{theme::ColorfulTheme, Select};
use std::process::Command;
use std::io::{self, Write};

fn main() {
    let base_dir = std::env::current_dir().unwrap();
    let setup_dir = base_dir.join("setup");
    let output_dir = base_dir.join("output");
    let ytdlp_path = setup_dir.join("yt-dlp.exe");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir).unwrap();
    }

    let options = vec!["Audio", "Video"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Download type")
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    let download_type = options[selection];

    print!("Enter YouTube URL: ");
    io::stdout().flush().unwrap();
    let mut url = String::new();
    io::stdin().read_line(&mut url).unwrap();
    let url = url.trim();

    let (format_arg, is_audio) = if download_type == "Audio" {
        let audio_formats = vec!["flac", "mp3", "wav", "aac", "m4a"];
        let format_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select audio format")
            .items(&audio_formats)
            .default(0)
            .interact()
            .unwrap();
        (audio_formats[format_selection], true)
    } else {
        let video_formats = vec!["mp4", "mkv", "webm"];
        let format_selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select video format")
            .items(&video_formats)
            .default(0)
            .interact()
            .unwrap();
        (video_formats[format_selection], false)
    };

    let mut command = Command::new(&ytdlp_path);
    command.current_dir(&output_dir);

    if is_audio {
        command.args(&[
            "-f", "bestaudio/best",
            "-ciw",
            "-o", "%(title)s.%(ext)s",
            "-v",
            "--extract-audio",
            "--audio-format", format_arg,
            url,
        ]);
    } else {
        let format_str = match format_arg {
            "mp4" => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "mkv" => "bestvideo[ext=webm]+bestaudio/best[ext=mkv]/best",
            "webm" => "bestvideo[ext=webm]+bestaudio/best[ext=webm]/best",
            _ => "bestvideo+bestaudio/best",
        };
        command.args(&[
            "-f", format_str,
            "-ciw",
            "-o", "%(title)s.%(ext)s",
            "-v",
            url,
        ]);
    }

    let status = command.status().expect("Failed to run yt-dlp");
    if status.success() {
        println!("\nDownload complete!");
    } else {
        println!("\nDownload failed!");
    }
}
