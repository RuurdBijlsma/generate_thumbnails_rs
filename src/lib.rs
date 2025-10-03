//! # Thumbnail Generation Crate
//!
//! A personal library for generating a variety of thumbnails from image and video files
//! using FFmpeg and FFprobe.
//!
//! This crate provides a unified interface, `generate_thumbnails`, which can handle
//! both image and video files based on their extension. The generation process is highly
//! configurable through the `OutputOptions` struct, allowing for the creation of:
//! - Multiple sizes of still images from a single timestamp in a video.
//! - Stills from multiple timestamps (as percentages) in a video.
//! - Lower-resolution video previews (e.g., WebM).
//! - Multiple sizes of thumbnails from a source image.
//!
//! All operations are performed asynchronously using `tokio`.
//!
//! ## Requirements
//!
//! - **FFmpeg**: Must be installed and accessible in the system's `PATH`.
//! - **FFprobe**: Must be installed and accessible in the system's `PATH`.
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//! use ruurd_photos_thumbnail_generation::{generate_thumbnails, OutputOptions, VideoOutputFormat};
//! use anyhow::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let source_file = Path::new("path/to/video.mp4");
//!     let output_dir = Path::new("path/to/thumbnails");
//!
//!     let config = OutputOptions {
//!         thumb_format: "avif".to_string(),
//!         heights: vec!,
//!         thumb_time: 1.5, // seconds
//!         percentages: vec!,
//!         height: 720,
//!         output_videos: vec![
//!             VideoOutputFormat {
//!                 height: 480,
//!                 quality: 35,
//!             },
//!         ],
//!     };
//!
//!     if let Err(e) = generate_thumbnails(source_file, output_dir, &config).await {
//!         eprintln!("Failed to generate thumbnails: {}", e);
//!     }
//!
//!     Ok(())
//! }
//! ```

// Internal module for utility functions, like moving files.
mod utils;
// The core module for generating thumbnails.
mod thumbnails;
// Module for interacting with the `ffprobe` command-line tool.
mod ffprobe;
// Module for interacting with the `ffmpeg` command-line tool.
mod ffmpeg;

// Re-export the primary configuration structs and the main function for easy access.
pub use thumbnails::OutputOptions;
pub use thumbnails::VideoOutputFormat;
pub use thumbnails::generate_thumbnails;