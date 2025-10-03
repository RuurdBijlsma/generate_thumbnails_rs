use crate::ffmpeg::run_ffmpeg;
use crate::ffprobe::get_video_duration;
use crate::utils::move_dir_contents;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use temp_dir::TempDir;
use tokio::fs;

/// Defines the output format for a generated video preview.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoOutputFormat {
    /// The height of the output video in pixels. The width will be scaled automatically to maintain aspect ratio.
    pub height: u64,
    /// The quality setting for the video encoding. For VP9, this is the CRF (Constant Rate Factor) value.
    pub quality: u64,
}

/// A comprehensive configuration for generating thumbnails for both images and videos.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputOptions {
    /// The file extension for still image thumbnails (e.g., "avif", "jpg").
    pub thumb_format: String,
    /// A vector of heights for generating multiple thumbnails.
    /// - For videos, these are the heights for stills taken at `thumb_time`.
    /// - For images, these are the heights for the generated thumbnails.
    pub heights: Vec<u64>,
    /// The specific time in seconds from the start of the video to generate multi-size stills from.
    pub thumb_time: f64,
    /// A vector of percentages of the video's total duration at which to capture still images.
    pub percentages: Vec<u64>,
    /// The height in pixels for the thumbnails generated based on the `percentages` field.
    pub height: u64,
    /// A list of video formats to generate as previews from the source video.
    pub output_videos: Vec<VideoOutputFormat>,
}

fn path_str(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn map_still(label: &str, out: &Path) -> Vec<String> {
    vec![
        "-map".into(),
        label.into(),
        "-frames:v".into(),
        "1".into(),
        path_str(out),
    ]
}

pub async fn generate_image_thumbnails(
    input: &Path,
    output_dir: &Path,
    heights: &[u64],
    ext: &str,
) -> Result<()> {
    if heights.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(output_dir).await?;
    let input_str = input.to_str().context("invalid input path")?;

    let split_labels: Vec<String> = (0..heights.len()).map(|i| format!("[v{i}]")).collect();
    let mut filter_parts = vec![format!(
        "[0:v]split={}{}",
        heights.len(),
        split_labels.join(""),
    )];

    let mut args = vec!["-y".into(), "-i".into(), input_str.into()];
    let mut map_args = Vec::new();

    for (i, &h) in heights.iter().enumerate() {
        let out_label = format!("[out{i}]");
        filter_parts.push(format!("[v{i}]scale=-1:{h}{out_label}"));
        let out = output_dir.join(format!("{h}.{ext}"));
        map_args.extend(map_still(&out_label, &out));
    }

    args.push("-filter_complex".into());
    args.push(filter_parts.join(";"));
    args.extend(map_args);

    run_ffmpeg(&args).await
}

pub async fn generate_video_thumbnails(
    input: &Path,
    output_dir: &Path,
    config: &OutputOptions,
) -> Result<()> {
    if config.heights.is_empty() && config.percentages.is_empty() && config.output_videos.is_empty()
    {
        return Ok(());
    }

    fs::create_dir_all(output_dir).await?;
    let input_str = path_str(input);
    let duration = get_video_duration(input).await?;

    let mut args = vec!["-y".into()];
    let mut filters = Vec::new();
    let mut maps = Vec::new();
    let mut input_idx = 0;
    let time_height = config.height;
    let thumb_ext = config.thumb_format.clone();

    // 1. time-based stills
    for (i, &pct) in config.percentages.iter().enumerate() {
        let ts = (pct as f64) / 100. * duration;
        args.extend(["-ss".into(), ts.to_string(), "-i".into(), input_str.clone()]);
        let out_label = format!("[out_ts{i}]");
        filters.push(format!("[{input_idx}:v]scale=-1:{time_height}{out_label}"));
        let out = output_dir.join(format!("{pct:.0}_percent.{thumb_ext}"));
        maps.extend(map_still(&out_label, &out));
        input_idx += 1;
    }

    // 2. multi-size stills at fixed time
    if !config.heights.is_empty() {
        args.extend([
            "-ss".into(),
            config.thumb_time.to_string(),
            "-i".into(),
            input_str.clone(),
        ]);
        let split_labels: Vec<String> = (0..config.heights.len())
            .map(|i| format!("[ms{i}]"))
            .collect();
        filters.push(format!(
            "[{input_idx}:v]split={}{}",
            config.heights.len(),
            split_labels.join("")
        ));
        for (i, &h) in config.heights.iter().enumerate() {
            let out_label = format!("[out_ms{i}]");
            filters.push(format!("[ms{i}]scale=-1:{h}{out_label}"));
            let out = output_dir.join(format!("{h}p.{thumb_ext}"));
            maps.extend(map_still(&out_label, &out));
        }
        input_idx += 1;
    }

    // 3. multi-res webm
    if !config.output_videos.is_empty() {
        args.extend(["-i".into(), input_str.clone()]);
        let vlabels: Vec<String> = (0..config.output_videos.len())
            .map(|i| format!("[v{i}]"))
            .collect();
        let alabels: Vec<String> = (0..config.output_videos.len())
            .map(|i| format!("[a{i}]"))
            .collect();
        filters.push(format!(
            "[{input_idx}:v:0]split={}{}",
            config.output_videos.len(),
            vlabels.join("")
        ));
        filters.push(format!(
            "[{input_idx}:a:0?]asplit={}{}",
            config.output_videos.len(),
            alabels.join("")
        ));

        for (i, hq_config) in config.output_videos.iter().enumerate() {
            let vout = format!("[out_v{i}]");
            let h = hq_config.height;
            filters.push(format!("[v{i}]scale=-2:{h}{vout}"));
            let out = output_dir.join(format!("{h}p.webm"));
            maps.extend([
                "-map".into(),
                vout,
                "-map".into(),
                alabels[i].clone(),
                "-c:v".into(),
                "libvpx-vp9".into(),
                "-crf".into(),
                hq_config.quality.to_string(),
                "-b:v".into(),
                "0".into(),
                "-c:a".into(),
                "libopus".into(),
                "-b:a".into(),
                "64k".into(),
                path_str(&out),
            ]);
        }
    }

    if !filters.is_empty() {
        args.push("-filter_complex".into());
        args.push(filters.join(";"));
        args.extend(maps);
    }

    run_ffmpeg(&args).await
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
/// * `thumbs_dir` - The base directory where the output subfolder will be created.
/// * `config` - An `OutputOptions` struct detailing what thumbnails to generate.
///
/// # Errors
///
/// This function will return an error if:
/// - File paths are invalid.
/// - The `ffmpeg` or `ffprobe` commands fail.
/// - There are issues with file I/O, such as creating directories or moving files.
pub async fn generate_thumbnails(
    file: &Path,
    thumbs_dir: &Path,
    config: &OutputOptions,
) -> Result<()> {
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
    let temp_dir = TempDir::new()?;
    let temp_out_dir = temp_dir.path();

    if photo_extensions.contains(&extension.as_str()) {
        generate_image_thumbnails(file, temp_out_dir, &config.heights, "avif").await?
    } else if video_extensions.contains(&extension.as_str()) {
        generate_video_thumbnails(file, temp_out_dir, config).await?
    } else {
        println!("Skipping file: {:?}", file);
    }

    let output_folder = thumbs_dir.join(filename);
    move_dir_contents(temp_out_dir, &output_folder).await?;
    temp_dir.cleanup()?;

    Ok(())
}
