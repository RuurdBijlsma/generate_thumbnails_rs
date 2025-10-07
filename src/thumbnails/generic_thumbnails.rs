use crate::thumbnails::ffmpeg_photo_thumbnail::generate_ffmpeg_photo_thumbnails;
use crate::thumbnails::photo_thumbnails::generate_photo_thumbnails;
use crate::thumbnails::video_thumbnails::generate_video_thumbnails;
use crate::utils::move_dir_contents;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use temp_dir::TempDir;

/// Defines the output format for a generated video preview.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoOutputFormat {
    /// The height of the output video in pixels. The width will be scaled automatically to maintain aspect ratio.
    pub height: u64,
    /// The quality setting for the video encoding. For VP9, this is the CRF (Constant Rate Factor) value.
    pub quality: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvifOptions {
    /// Quality 1..=100. Panics if out of range.
    pub quality: f32,
    /// Quality for the alpha channel only. `1..=100`. Panics if out of range.
    pub alpha_quality: f32,
    /// - 1 = very slow, but max compression.
    /// - 10 = quick, but larger file sizes and lower quality.
    ///
    /// Panics if outside 1..=10.
    pub speed: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoThumbOptions {
    /// The specific time in seconds from the start of the video to generate multi-size stills from.
    pub thumb_time: f64,
    /// A vector of percentages of the video's total duration at which to capture still images.
    pub percentages: Vec<u64>,
    /// The height in pixels for the thumbnails generated based on the `percentages` field.
    pub height: u64,
    /// A list of video formats to generate as previews from the source video.
    pub transcode_outputs: Vec<VideoOutputFormat>,
    /// The file extension for video transcoding (e.g., "webm", "mp4").
    pub extension: String,
}

/// A comprehensive configuration for generating thumbnails for both images and videos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThumbOptions {
    /// Which extensions are categorized as video
    pub video_extensions: Vec<String>,
    /// Which extensions are categorized as photos
    pub photo_extensions: Vec<String>,
    /// A vector of heights for generating multiple thumbnails.
    /// - For videos, these are the heights for stills taken at `thumb_time`.
    /// - For images, these are the heights for the generated thumbnails.
    pub heights: Vec<u64>,
    /// The file extension for photo thumbnails (e.g., "avif", "webp", "jpg").
    pub thumbnail_extension: String,
    pub avif_options: AvifOptions,
    pub video_options: VideoThumbOptions,
    pub skip_if_exists: bool,
}

async fn thumbs_exist(file: &Path, thumb_folder: &Path, config: &ThumbOptions) -> Result<bool> {
    let Some(extension) = file
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| x.to_lowercase())
    else {
        return Ok(false);
    };
    let is_photo = config.photo_extensions.contains(&extension);
    let is_video = config.video_extensions.contains(&extension);

    let photo_thumb_ext = &config.thumbnail_extension;
    let video_thumb_ext = &config.video_options.extension;
    let mut should_exist: Vec<String> = vec![];

    if is_photo || is_video {
        // Both photo and video should have a thumbnail for each entry in .heights.
        for h in &config.heights {
            should_exist.push(format!("{h}p.{photo_thumb_ext}"))
        }
    }
    if is_video {
        for p in &config.video_options.percentages {
            should_exist.push(format!("{p}_percent.{photo_thumb_ext}"))
        }
        for x in &config.video_options.transcode_outputs {
            let height = x.height;
            should_exist.push(format!("{height}p.{video_thumb_ext}"))
        }
    }

    for thumb_filename in should_exist {
        if !fs::exists(thumb_folder.join(thumb_filename.clone()))? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Generates thumbnails for a given media file (image or video) based on the provided configuration.
///
/// This function detects the file type based on its extension and then calls the appropriate
/// thumbnail generation logic.
///
/// - For supported image types, it generates resized thumbnails.
/// - For supported video types, it can generate a complex combination of still images and video previews.
///
/// The generated files are first created in a temporary directory and then moved to a dedicated
/// subfolder within the `thumbs_dir`, named after the original file.
///
/// # Arguments
///
/// * `file` - The path to the source image or video file.
/// * `out_folder` - Where to output the thumbnail files.
/// * `config` - An `ThumbOptions` struct detailing what thumbnails to generate.
///
/// # Errors
///
/// This function will return an error if:
/// - File paths are invalid.
/// - The `ffmpeg` or `ffprobe` commands fail.
/// - There are issues with file I/O, such as creating directories or moving files.
pub async fn generate_thumbnails(
    file: &Path,
    out_folder: &Path,
    config: &ThumbOptions,
) -> Result<()> {
    let Some(extension) = file.extension().and_then(|s| s.to_str()) else {
        return Ok(());
    };

    if config.skip_if_exists && thumbs_exist(file, out_folder, config).await? {
        return Ok(());
    }

    let extension = extension.to_lowercase();
    let temp_dir = TempDir::new()?;
    let temp_out_dir = temp_dir.path();

    if config.photo_extensions.contains(&extension) {
        if config.thumbnail_extension == "avif" {
            generate_photo_thumbnails(file, temp_out_dir, config)?;
        } else {
            generate_ffmpeg_photo_thumbnails(file, temp_out_dir, config).await?;
        }
    } else if config.video_extensions.contains(&extension) {
        generate_video_thumbnails(file, temp_out_dir, config).await?;
    }

    move_dir_contents(temp_out_dir, out_folder).await?;
    temp_dir.cleanup()?;

    Ok(())
}
