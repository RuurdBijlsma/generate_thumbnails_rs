use crate::ffmpeg::run_ffmpeg;
use crate::ffprobe::get_video_duration;
use anyhow::Context;
use std::path::Path;
use tokio::fs;

pub async fn generate_image_thumbnails(
    input: &Path,
    output_dir: &Path,
    heights: &[u64],
    ext: &str,
) -> anyhow::Result<()> {
    if heights.is_empty() {
        return Ok(());
    }

    fs::create_dir_all(output_dir).await?;

    let input_str = input.to_str().context("invalid input path")?;
    let split_labels: String = (0..heights.len()).map(|i| format!("[v{i}]")).collect();
    let mut filter = format!("[0:v]split={}{};", heights.len(), split_labels);
    for (i, &h) in heights.iter().enumerate() {
        filter.push_str(&format!("[v{i}]scale=-1:{h}[out{i}];"));
    }
    filter.pop();

    let mut args = vec!["-y".to_string(), "-i".to_string(), input_str.to_string()];
    args.push("-filter_complex".into());
    args.push(filter);

    for (i, &h) in heights.iter().enumerate() {
        let out = output_dir.join(format!("{h}.{ext}"));
        args.extend([
            "-map".into(),
            format!("[out{i}]"),
            out.to_string_lossy().to_string(),
        ]);
    }

    run_ffmpeg(&args).await
}

pub async fn generate_video_thumbnails(
    input: &Path,
    output_dir: &Path,
    ext: &str,
    multi_size_heights: &[u64],
    multi_size_time: f64,
    multi_time_percentages: &[f64],
    multi_time_height: u64,
) -> anyhow::Result<()> {
    if multi_size_heights.is_empty() && multi_time_percentages.is_empty() {
        return Ok(());
    }
    let duration = get_video_duration(input).await?;

    fs::create_dir_all(output_dir).await?;
    let input_str = input.to_string_lossy();

    let mut args = vec!["-y".to_string()];
    let mut filter_complex = String::new();
    let mut map_args = Vec::new();
    let mut input_idx = 0;

    // Multi-time thumbnails
    for (i, &percentage) in multi_time_percentages.iter().enumerate() {
        let timestamp = percentage / 100. * duration;
        args.extend([
            "-ss".to_string(),
            timestamp.to_string(),
            "-i".to_string(),
            input_str.clone().to_string(),
        ]);
        let out_label = format!("[out_ts{i}]");
        filter_complex.push_str(&format!(
            "[{input_idx}:v]scale=-1:{multi_time_height}{out_label};"
        ));
        let out_path = output_dir.join(format!("{percentage:.0}_percent.{ext}"));
        map_args.extend([
            "-map".into(),
            out_label,
            "-frames:v".into(),
            "1".into(),
            out_path.to_string_lossy().to_string(),
        ]);
        input_idx += 1;
    }

    // Multi-size thumbnails
    if !multi_size_heights.is_empty() {
        args.extend([
            "-ss".to_string(),
            multi_size_time.to_string(),
            "-i".to_string(),
            input_str.to_string(),
        ]);
        let split_labels: String = (0..multi_size_heights.len())
            .map(|i| format!("[ms{i}]"))
            .collect();
        filter_complex.push_str(&format!(
            "[{input_idx}:v]split={}{};",
            multi_size_heights.len(),
            split_labels
        ));
        for (i, &h) in multi_size_heights.iter().enumerate() {
            let out_label = format!("[out_ms{i}]");
            filter_complex.push_str(&format!("[ms{i}]scale=-1:{h}{out_label};"));
            let out_path = output_dir.join(format!("{h}p.{ext}"));
            map_args.extend([
                "-map".into(),
                out_label,
                "-frames:v".into(),
                "1".into(),
                out_path.to_string_lossy().to_string(),
            ]);
        }
    }

    if filter_complex.ends_with(';') {
        filter_complex.pop();
    }

    args.push("-filter_complex".into());
    args.push(filter_complex);
    args.extend(map_args);

    run_ffmpeg(&args).await
}
