use color_eyre::Result;
use futures::stream::{self, StreamExt};
use ruurd_photos_thumbnail_generation::{ThumbOptions, VideoOutputFormat, generate_thumbnails};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio_retry::Retry;
use tokio_retry::strategy::FixedInterval;
use walkdir::WalkDir;

const CONCURRENT_FILES: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    let source_folder = Path::new("assets");
    let thumbs_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbs_dir).await?;

    let config = ThumbOptions {
        photo_extensions: ["jpg", "jpeg", "png", "gif", "tiff", "tga", "avif"]
            .iter()
            .map(|x| x.to_string())
            .collect(),
        video_extensions: [
            "mp4", "webm", "av1", "3gp", "mov", "mkv", "flv", "m4v", "m4p",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect(),
        thumb_ext: "avif".to_string(),
        transcode_ext: "webm".to_string(),
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
        skip_if_exists: true,
    };

    let files_to_process: Vec<PathBuf> = WalkDir::new(source_folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();

    let processing_tasks = stream::iter(files_to_process)
        .map(|path| {
            let config = config.clone();
            let thumbs_dir = thumbs_dir.to_path_buf();

            tokio::spawn(async move {
                println!("Processing file: {:?}", &path);
                let retry_strategy = FixedInterval::from_millis(500).take(3);
                let result = Retry::spawn(retry_strategy, || async {
                    generate_thumbnails(&path, &thumbs_dir, &config).await
                })
                .await;
                if let Err(e) = result {
                    eprintln!(
                        "Failed to process file {:?} after multiple attempts: {}",
                        &path, e
                    );
                }
            })
        })
        .buffer_unordered(CONCURRENT_FILES);

    processing_tasks.for_each(|_| async {}).await;
    Ok(())
}
