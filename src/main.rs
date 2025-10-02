use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

/// Runs an FFmpeg command asynchronously.
async fn run_ffmpeg<S: AsRef<OsStr>>(args: &[S]) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("failed to run ffmpeg command")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr.trim());
    }
}

/// Generates multiple thumbnails from a single source image in one FFmpeg call.
async fn generate_image_thumbnails(
    input_path: &Path,
    output_folder: &Path,
    heights: &[u64],
    output_extension: &str,
) -> Result<()> {
    if heights.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(output_folder)
        .await
        .context("failed to create output directory")?;

    let input_str = input_path
        .to_str()
        .context("input path is not valid UTF-8")?;

    let mut args = vec!["-y".to_string(), "-i".to_string(), input_str.to_string()];

    let split_labels: String = (0..heights.len()).map(|i| format!("[v{i}]")).collect();
    let mut filter = format!("[0:v]split={}{};", heights.len(), split_labels);

    for (i, &h) in heights.iter().enumerate() {
        filter.push_str(&format!("[v{i}]scale=-1:{h}[out{i}];"));
    }
    filter.pop(); // remove trailing ;

    args.push("-filter_complex".into());
    args.push(filter);

    for (i, &height) in heights.iter().enumerate() {
        let out = output_folder.join(format!("{height}.{output_extension}"));
        let out_str = out.to_str().context("output path is not valid UTF-8")?;
        args.extend(["-map".into(), format!("[out{i}]"), out_str.into()]);
    }

    println!("Running image thumbnail generation...");
    run_ffmpeg(&args).await
}

/// Generates a complex series of video thumbnails in a single FFmpeg call.
async fn generate_video_thumbnail_series(
    input_path: &Path,
    output_folder: &Path,
    output_extension: &str,
    // For the multi-size set
    multi_size_heights: &[u64],
    multi_size_time_sec: f64,
    // For the multi-time set
    multi_time_stamps_sec: &[f64],
    multi_time_height: u64,
) -> Result<()> {
    if multi_size_heights.is_empty() && multi_time_stamps_sec.is_empty() {
        return Ok(()); // Nothing to do
    }

    fs::create_dir_all(output_folder)
        .await
        .context("failed to create video thumbnail output directory")?;

    let input_str = input_path
        .to_str()
        .context("input path is not valid UTF-8")?;

    let mut args = vec!["-y".to_string()];

    // --- Input seeking for faster frame grabbing ---
    // This is a bit of a trick. We specify the input multiple times, once for each
    // timestamp we need to seek to. This is generally faster than decoding the
    // whole video to find the frames.
    for &ts in multi_time_stamps_sec {
        args.extend(["-ss".to_string(), ts.to_string(), "-i".to_string(), input_str.to_string()]);
    }
    if !multi_size_heights.is_empty() {
        args.extend(["-ss".to_string(), multi_size_time_sec.to_string(), "-i".to_string(), input_str.to_string()]);
    }

    let mut filter_complex = String::new();
    let mut map_args: Vec<String> = Vec::new();

    let mut input_idx = 0;

    // Part 1: Multi-timestamp, single resolution thumbnails
    for (i, &ts) in multi_time_stamps_sec.iter().enumerate() {
        let stream_label = format!("[ts{i}]");
        let output_label = format!("[out_ts{i}]");

        filter_complex.push_str(&format!("[{input_idx}:v]scale=-1:{multi_time_height}{stream_label};"));
        filter_complex.push_str(&format!("{stream_label}fps=1{output_label};"));

        let out_path = output_folder.join(format!("{ts}s.{output_extension}"));
        let out_str = out_path.to_str().context("output path is not valid UTF-8")?;
        map_args.extend(["-map".into(), output_label, "-frames:v".into(), "1".into(), out_str.into()]);
        input_idx += 1;
    }

    // Part 2: Multi-size, single timestamp thumbnails
    if !multi_size_heights.is_empty() {
        let split_labels: String = (0..multi_size_heights.len()).map(|i| format!("[ms_v{i}]")).collect();
        filter_complex.push_str(&format!("[{input_idx}:v]split={}{};", multi_size_heights.len(), split_labels));

        for (i, &h) in multi_size_heights.iter().enumerate() {
            let stream_label = format!("[ms_v{i}]");
            let output_label = format!("[out_ms{i}]");

            filter_complex.push_str(&format!("{stream_label}scale=-1:{h}{output_label};"));

            let out_path = output_folder.join(format!("{h}p.{output_extension}"));
            let out_str = out_path.to_str().context("output path is not valid UTF-8")?;
            map_args.extend(["-map".into(), output_label, "-frames:v".into(), "1".into(), out_str.into()]);
        }
    }


    if filter_complex.ends_with(';') {
        filter_complex.pop();
    }


    args.push("-filter_complex".into());
    args.push(filter_complex);
    args.extend(map_args);


    println!("Running complex video thumbnail generation...");
    run_ffmpeg(&args).await
}


#[tokio::main]
async fn main() -> Result<()> {
    let image_in_path = Path::new("assets/PICT0008.JPG");
    let image_out_path = Path::new("image_thumbnails");
    generate_image_thumbnails(image_in_path, image_out_path, &[240, 480, 1080], "avif").await?;

    // --- Video Example ---
    let video_in_path = Path::new("assets/jellyfish.mp4");
    let video_out_folder = Path::new("video_thumbnails_series");

    println!("\n--- Generating Complex Video Thumbnail Series ---");
    generate_video_thumbnail_series(
        video_in_path,
        video_out_folder,
        "avif",
        // Part 1: Get 1080p and 480p thumbnails from the 10-second mark
        &[240,480, 1080],
        0.5,
        // Part 2: Get 720p thumbnails from the 5, 15, and 25-second marks
        &[10.0, 30.0, 50.0],
        720,
    )
        .await?;
    println!(
        "Video thumbnail series generated in '{}'",
        video_out_folder.display()
    );

    Ok(())
}