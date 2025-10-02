mod ffmpeg;
mod thumbnails;

use crate::thumbnails::{generate_image_thumbnails, generate_video_thumbnails};
use anyhow::Result;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    generate_image_thumbnails(
        Path::new("assets/PICT0008.JPG"),
        Path::new("thumbs_image"),
        &[240, 480, 1080],
        "avif",
    )
    .await?;

    generate_video_thumbnails(
        Path::new("assets/jellyfish.mp4"),
        Path::new("thumbs_video"),
        "avif",
        &[240, 480, 1080],
        0.5,
        &[10.0, 30.0, 50.0],
        720,
    )
    .await?;

    Ok(())
}
