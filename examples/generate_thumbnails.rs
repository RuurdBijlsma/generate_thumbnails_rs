use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;
use walkdir::WalkDir;
use ruurd_photos_thumbnail_generation::{generate_thumbnails, OutputOptions, VideoOutputFormat};

#[tokio::main]
async fn main() -> Result<()> {
    let source_folder = Path::new("../assets");
    let thumbs_dir = Path::new("../thumbs");
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
