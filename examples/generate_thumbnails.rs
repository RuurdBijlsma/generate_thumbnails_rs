use color_eyre::Result;
use futures::stream::{self, StreamExt};
use ruurd_photos_thumbnail_generation::{
    AvifOptions, ThumbOptions, VideoOutputFormat, VideoThumbOptions, generate_thumbnails,
};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio_retry::Retry;
use tokio_retry::strategy::FixedInterval;
use walkdir::WalkDir;

const CONCURRENT_FILES: usize = 4;

#[tokio::main]
async fn main() -> Result<()> {
    let source_folder = Path::new("assets");
    let thumbnails_dir = Path::new("thumbs");
    fs::create_dir_all(&thumbnails_dir).await?;

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
        skip_if_exists: true,
        heights: vec![10, 144, 240, 360, 480, 720, 1080],
        thumbnail_extension: "avif".to_string(),
        avif_options: AvifOptions {
            quality: 80.,
            alpha_quality: 80.,
            speed: 4,
        },
        video_options: VideoThumbOptions {
            extension: "webm".to_string(),
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
        },
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

            tokio::spawn(async move {
                println!("Processing file: {:?}", &path);
                let retry_strategy = FixedInterval::from_millis(500).take(3);
                let result = Retry::spawn(retry_strategy, || async {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    generate_thumbnails(&path, &thumbnails_dir.join(filename), &config).await
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
