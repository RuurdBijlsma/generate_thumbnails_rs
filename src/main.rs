mod ffmpeg;
mod ffprobe;
mod thumbnails;

use crate::thumbnails::{generate_image_thumbnails, generate_video_thumbnails};
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;
use walkdir::WalkDir;

#[tokio::main]
async fn main() -> Result<()> {
    let photo_extensions = ["jpg", "jpeg", "png", "gif", "tiff", "tga"];
    let video_extensions = [
        "mp4", "webm", "av1", "3gp", "mov", "mkv", "flv", "m4v", "m4p",
    ];

    let source_folder = Path::new("assets");
    let image_thumbs_dir = Path::new("thumbs/photo");
    let video_thumbs_dir = Path::new("thumbs/video");
    fs::create_dir_all(&image_thumbs_dir).await?;
    fs::create_dir_all(&video_thumbs_dir).await?;

    let sizes = &[240, 480, 1080];
    let thumb_time = 0.5;
    let video_percentages = &[0., 20., 40., 60., 80., 98.];
    let video_thumb_height = 720;

    for entry in WalkDir::new(source_folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(extension) = entry.path().extension().and_then(|s| s.to_str())
            && let Some(filename) = entry.path().file_name().and_then(|s| s.to_str())
        {
            let extension = extension.to_lowercase();
            println!("Processing file: {:?}", entry.path());

            let retry_strategy = FixedInterval::from_millis(500).take(3);

            let result = if photo_extensions.contains(&extension.as_str()) {
                let output_folder = image_thumbs_dir.join(filename);
                fs::create_dir_all(&output_folder).await?;

                Retry::spawn(retry_strategy, || async {
                    generate_image_thumbnails(entry.path(), &output_folder, sizes, "avif").await
                })
                .await
            } else if video_extensions.contains(&extension.as_str()) {
                let output_folder = video_thumbs_dir.join(filename);
                fs::create_dir_all(&output_folder).await?;

                Retry::spawn(retry_strategy, || async {
                    generate_video_thumbnails(
                        entry.path(),
                        &output_folder,
                        "avif",
                        sizes,
                        thumb_time,
                        video_percentages,
                        video_thumb_height,
                    )
                    .await
                })
                .await
            } else {
                println!("Skipping file: {:?}", entry.path());
                // Not a file type we need to process, so we mark it as success.
                Ok(())
            };

            if let Err(e) = result {
                eprintln!(
                    "Failed to process file {:?} after multiple attempts: {}",
                    entry.path(),
                    e
                );
            }
        }
    }

    Ok(())
}
