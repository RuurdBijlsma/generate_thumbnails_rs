use crate::ffmpeg::run_ffmpeg;
use crate::ffprobe::get_video_duration;
use crate::thumbnails::generic_thumbnails::ThumbOptions;
use crate::thumbnails::utils::{map_still, path_str};
use std::path::Path;
use tokio::fs;

pub async fn generate_video_thumbnails(
    input: &Path,
    output_dir: &Path,
    config: &ThumbOptions,
) -> color_eyre::Result<()> {
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
    let thumb_ext = config.thumb_ext.clone();

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
