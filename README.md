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
*   FFmpeg and FFprobe installed and available in your system's `PATH`.

## Usage

The main way to use the crate is by creating an `OutputOptions` configuration and passing it to the `generate_thumbnails` function. This function will process a source file and place the generated thumbnails into a dedicated subfolder within the specified output directory.

Here is an example of how to process a directory of files:

```rust
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;
use ruurd_photos_thumbnail_generation::{generate_thumbnails, OutputOptions, VideoOutputFormat};

#[tokio::main]
async fn main() -> Result<()> {
    let source_folder = Path::new("path/to/your/media");
    let thumbs_dir = Path::new("path/to/your/thumbnails");
    fs::create_dir_all(&thumbs_dir).await?;

    // Configure all the thumbnail types you want to generate.
    let config = OutputOptions {
        // The format for still image thumbnails (e.g., "avif", "jpg").
        thumb_format: "avif".to_string(),
        
        // For videos: generate stills of these heights from `thumb_time`.
        // For images: generate thumbnails of these heights.
        heights: vec![240, 480, 1080],
        
        // The time in seconds to take the multi-height stills from a video.
        thumb_time: 0.5,
        
        // For videos: generate stills at these percentages of the video duration.
        percentages: vec![0, 33, 66, 99],
        
        // The height for the percentage-based stills.
        height: 720,
        
        // For videos: generate re-encoded video clips with these settings.
        output_videos: vec![
            VideoOutputFormat {
                height: 480,
                quality: 35, // CRF value for VP9
            },
            VideoOutputFormat {
                height: 144,
                quality: 40,
            },
        ],
    };

    for entry in WalkDir::new(source_folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        println!("Processing file: {:?}", entry.path());
        // generate_thumbnails will inspect the file and create the configured outputs.
        let result = generate_thumbnails(entry.path(), thumbs_dir, &config).await;
        
        if let Err(e) = result {
            eprintln!("Failed to process file {:?}: {}", entry.path(), e);
        }
    }

    Ok(())
}
```