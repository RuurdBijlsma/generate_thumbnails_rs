mod ffmpeg;
mod ffprobe;
mod thumbnails;

use crate::thumbnails::{generate_image_thumbnails, generate_video_thumbnails};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio_retry::Retry;
use tokio_retry::strategy::FixedInterval;
use walkdir::WalkDir;

// todo:
// process to temp folder, copy when success
// webm outputs
// clean up code

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VideoOutputFormat {
    height: u64,
    quality: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputOptions {
    thumb_format: String,
    heights: Vec<u64>,
    thumb_time: f64,
    percentages: Vec<u64>,
    height: u64,
    output_videos: Vec<VideoOutputFormat>,
}

async fn generate_thumbnails(file: &Path, thumbs_dir: &Path, config: &OutputOptions) -> Result<()> {
    let Some(extension) = file.extension().and_then(|s| s.to_str()) else {
        return Ok(());
    };
    let Some(filename) = file.file_name().and_then(|s| s.to_str()) else {
        return Ok(());
    };

    let photo_extensions = ["jpg", "jpeg", "png", "gif", "tiff", "tga"];
    let video_extensions = [
        "mp4", "webm", "av1", "3gp", "mov", "mkv", "flv", "m4v", "m4p",
    ];

    let extension = extension.to_lowercase();
    let output_folder = thumbs_dir.join(filename);
    fs::create_dir_all(&output_folder).await?;

    if photo_extensions.contains(&extension.as_str()) {
        generate_image_thumbnails(file, &output_folder, &config.heights, "avif").await?
    } else if video_extensions.contains(&extension.as_str()) {
        generate_video_thumbnails(file, &output_folder, config).await?
    } else {
        println!("Skipping file: {:?}", file);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let source_folder = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;

    let config = OutputOptions {
        thumb_format: "avif".to_string(),
        heights: vec![240, 480, 1080],
        thumb_time: 0.5,
        percentages: vec![0, 33, 66, 99],
        height: 720,
        output_videos: vec![
            VideoOutputFormat {
                height: 480,
                quality: 35,
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
        let retry_strategy = FixedInterval::from_millis(500).take(3);
        let result = Retry::spawn(retry_strategy, || async {
            generate_thumbnails(entry.path(), thumbs_dir, &config).await
        })
        .await;
        if let Err(e) = result {
            eprintln!(
                "Failed to process file {:?} after multiple attempts: {}",
                entry.path(),
                e
            );
        }
    }

    Ok(())
}
