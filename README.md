# Rust FFmpeg Thumbnail Generator

[ai generated readme]

This repo is an archive of my thumbnail generation code attempts which i'll use for ruurd photos 2
[link](github.com/ruurdbijlsma/photos-backend)

## Features

- **Asynchronous Execution**: Uses `tokio` to run `ffmpeg` commands asynchronously, preventing the application from
  blocking.
- **Error Handling**: Leverages the `anyhow` crate for robust and easy-to-read error handling.
- **Efficient Image Thumbnailing**: Generates multiple thumbnails of different heights from a single source image in a
  *single `ffmpeg` call*.
- **Complex Video Thumbnailing**: Generates a complex series of video thumbnails in a *single `ffmpeg` call*, including:
    - Multiple sizes from a single timestamp.
    - Multiple timestamps at a single size.
- **Efficient Seeking**: Uses multiple `-ss` inputs for video processing, which is significantly faster for grabbing
  frames from specific timestamps compared to decoding the entire video.

## Prerequisites

Before you can run this project, you must have the following installed on your system:

1. **Rust Toolchain**: Install Rust by following the official instructions at [rustup.rs](https://rustup.rs/).
2. **FFmpeg**: The `ffmpeg` command-line tool must be installed and accessible in your system's `PATH`. You can download
   it from the [official FFmpeg website](https://ffmpeg.org/download.html) or install it via a package manager like
   `brew`, `apt`, or `chocolatey`.

## How to Use

1. **Clone the Repository**:
   ```bash
   git clone <your-repository-url>
   cd <repository-name>
   ```

2. **Create the Assets Directory**:
   The code expects an `assets` directory with source files. Create it in the project root:
   ```bash
   mkdir assets
   ```

3. **Add Sample Files**:
   Place a sample image and a sample video into the `assets` directory. The current code is configured to use:
    - `assets/PICT0008.JPG`
    - `assets/kwal.mp4`

   You can modify the paths in the `main` function in `src/main.rs` to point to your own files.

4. **Run the Project**:
   Execute the program using Cargo:
   ```bash
   cargo run --release
   ```

   The program will create two new directories in the project root:
    - `image_thumbnails/`: Will contain the generated thumbnails from your source image (e.g., `240.avif`, `480.avif`,
      `1080.avif`).
    - `video_thumbnails_series/`: Will contain the complex set of thumbnails generated from your video file.

## Code Overview

- `main()`: The entry point of the application. It defines the input paths, output directories, and the desired
  thumbnail specifications before calling the generation functions.
- `run_ffmpeg()`: A helper function that takes `ffmpeg` arguments, spawns a child process, and waits for it to complete.
  It returns an error if the `ffmpeg` command fails, capturing the `stderr` output for easy debugging.
- `generate_image_thumbnails()`: Constructs a single `ffmpeg` command using the `-filter_complex` flag with the `split`
  and `scale` filters to create multiple resized versions of a source image in one go.
- `generate_video_thumbnail_series()`: Constructs a more advanced `ffmpeg` command. It uses multiple inputs with `-ss`
  for fast seeking and a complex filtergraph to extract frames from different timestamps and scale them to various
  sizes, all within a single process.

This project serves as a practical example of how to orchestrate powerful command-line tools like `ffmpeg` from within a
modern, safe, and concurrent language like Rust.