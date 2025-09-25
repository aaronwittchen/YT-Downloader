# YT Downloader CLI (Rust)

## Requirements

Make sure Rust is installed:

```bash
rustc --version
cargo --version
```

If not, install via [rustup](https://rustup.rs/).

Run with cargo:

```bash
cargo run
```

---

## Setup

1. Create a folder named `setup` in your project root.
2. Download the following files:

   - `ffmpeg.exe`, `ffplay.exe`, `ffprobe.exe` → [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html)
   - `yt-dlp.exe` → [https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#installation](https://github.com/yt-dlp/yt-dlp?tab=readme-ov-file#installation)

3. Move all files into the `setup` folder.

---

## Build

From your project root, run:

```bash
cargo build --release
```

This produces an optimized build.

The output `.exe` will be located at:

```
target/release/yt_downloader.exe
```

---

## Folder Structure for Distribution

Make sure the following is in your project root:

```
yt_downloader.exe
setup/      <-- contains ffmpeg.exe, ffplay.exe, ffprobe.exe, yt-dlp.exe
output/     <-- downloads will go here
```

That’s all you need to run the downloader.

---

## Usage

- Run `yt_downloader.exe`
- Select download type (Audio or Video)
- Enter a YouTube link:

```
https://www.youtube.com/watch?v=84J_XmGkX48&list=PLE6dlt5SQB8r5oagkd_cwA6FlhGLGlxef
```

> Note: If downloading a playlist, make sure the link contains `&list=`

- Choose the format (e.g., mp4, mp3, flac)
- Downloads will be saved in the `output` folder.
