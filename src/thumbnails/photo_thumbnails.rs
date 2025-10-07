use crate::ThumbOptions;
use color_eyre::eyre::eyre;
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use image::{ImageBuffer, Rgba};
use imgref::Img;
use ravif::Encoder;
use rayon::prelude::*;
use rgb::RGBA;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;

pub fn generate_photo_thumbnails(
    input_path: &Path,
    output_dir: &Path,
    config: &ThumbOptions,
) -> color_eyre::Result<()> {
    fs::create_dir_all(output_dir)?;
    if config.heights.is_empty() {
        return Ok(());
    }

    let src_img = image::open(input_path)?.to_rgba8();
    let (orig_w, orig_h) = src_img.dimensions();

    let src_image = Image::from_vec_u8(
        NonZeroU32::new(orig_w)
            .ok_or_else(|| eyre!("source image width is zero"))?
            .into(),
        NonZeroU32::new(orig_h)
            .ok_or_else(|| eyre!("source image height is zero"))?
            .into(),
        src_img.into_raw(),
        PixelType::U8x4,
    )?;

    config
        .heights
        .par_iter()
        .try_for_each(|&target_h| -> color_eyre::Result<()> {
            let mut resizer = Resizer::new();
            let encoder = Encoder::new()
                .with_quality(config.avif_options.quality)
                .with_speed(config.avif_options.speed)
                .with_alpha_quality(config.avif_options.alpha_quality);

            let target_w = ((orig_w as u64 * target_h) / orig_h as u64) as u32;
            if target_w == 0 || target_h == 0 {
                return Ok(());
            }

            let mut dst_img = Image::new(
                NonZeroU32::new(target_w).unwrap().into(),
                NonZeroU32::new(target_h as u32).unwrap().into(),
                PixelType::U8x4,
            );

            resizer.resize(&src_image, &mut dst_img, None)?;

            let resized =
                ImageBuffer::<Rgba<u8>, _>::from_raw(target_w, target_h as u32, dst_img.into_vec())
                    .ok_or_else(|| eyre!("Failed to construct resized image"))?;

            let rgba_vec: Vec<RGBA<u8>> = resized
                .pixels()
                .map(|p| RGBA {
                    r: p[0],
                    g: p[1],
                    b: p[2],
                    a: p[3],
                })
                .collect();
            let img_ref = Img::new(&rgba_vec[..], target_w as usize, target_h as usize);
            let avif_data = encoder.encode_rgba(img_ref)?;

            fs::write(
                output_dir.join(format!("{target_h}p.avif")),
                avif_data.avif_file,
            )?;

            Ok(())
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VideoOutputFormat;
    use crate::thumbnails::generic_thumbnails::{AvifOptions, VideoThumbOptions};
    use std::fs;
    use std::time::Instant;

    #[test]
    fn test_generate_thumbnails() -> color_eyre::Result<()> {
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
                // Bad quality and speed settings for test to speed it up.
                quality: 20.,
                alpha_quality: 20.,
                speed: 10,
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

        let input = Path::new("assets/tree.jpg");
        let filename = input
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap();
        let out_dir = Path::new("thumbs").join(&filename);
        if out_dir.exists() {
            fs::remove_dir_all(&out_dir)?;
        }

        let now = Instant::now();
        generate_photo_thumbnails(input, &out_dir, &config)?;
        println!("Elapsed: {:.2?}", now.elapsed());
        Ok(())
    }
}
