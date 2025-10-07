# Thumbnail Generator

A personal Rust crate for generating a variety of thumbnails from image and video files using FFmpeg.

## Features

- **Unified Interface**: A single `generate_thumbnails` function handles both images and videos based on their file extension.
- **Image Thumbnail Generation**: Creates multiple thumbnails of different heights from a source image in a single FFmpeg call.
- **Complex Video Thumbnailing**: A single FFmpeg command can generate a combination of:
  - **Multi-Size Stills**: Still images (e.g., AVIF) of various heights from a single, specific timestamp.
  - **Multi-Time Stills**: Still images of a fixed height taken at different percentages of the video's duration.
  - **Video Previews**: Re-encoded, lower-resolution video clips (e.g., WebM) for previews.
- **Async Execution**: Built with Tokio for non-blocking, asynchronous operations.

## Requirements

*   Rust (2021 edition or later)
*   nasm: https://nasm.us/ to build ravif
*   FFmpeg and FFprobe installed and available in your system's `PATH`.

## Usage

The main way to use the crate is by creating an `ThumbOptions` configuration and passing it to the `generate_thumbnails` function. This function will process a source file and place the generated thumbnails into a dedicated subfolder within the specified output directory.

Check [examples/generate_thumbnails.rs](examples/generate_thumbnails.rs) to see how to generate thumbnails.
