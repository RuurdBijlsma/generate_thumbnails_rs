use color_eyre::eyre::eyre;
use fast_image_resize::{PixelType, Resizer};
use image::{ImageBuffer, Rgba};
use imgref::Img;
use ravif::Encoder;
use rgb::RGBA;
use std::fs;
use std::num::NonZeroU32;
use std::path::Path;
use fast_image_resize::images::Image;

pub fn generate_photo_thumbnails(
    input_path: &Path,
    output_dir: &Path,
    heights: &[u64],
) -> color_eyre::Result<()> {
    fs::create_dir_all(output_dir)?;

    // 1. Open image and convert to RGBA8. This remains the correct first step.
    let src_img = image::open(input_path)?;
    let src_rgba8 = src_img.to_rgba8();
    let (orig_w, orig_h) = src_rgba8.dimensions();

    // 2. Create a `fast_image_resize::Image` from the raw pixel data.
    // This is the source image that the resizer will work with.
    // `src_rgba8.into_raw()` consumes the buffer and passes ownership.
    let src_image = Image::from_vec_u8(
        NonZeroU32::new(orig_w).ok_or_else(|| eyre!("source image width is zero"))?.into(),
        NonZeroU32::new(orig_h).ok_or_else(|| eyre!("source image height is zero"))?.into(),
        src_rgba8.into_raw(),
        PixelType::U8x4,
    )?;

    // Create the resizer once, outside the loop
    let mut resizer = Resizer::new();

    for &target_h in heights {
        let target_w = ((orig_w as u64 * target_h) / orig_h as u64) as u32;

        // Skip generating empty images
        if target_w == 0 || target_h == 0 {
            continue;
        }

        let w = NonZeroU32::new(target_w).unwrap();
        let h = NonZeroU32::new(target_h as u32).unwrap();

        // 3. Create an empty destination image with the target dimensions.
        let mut dst_img = Image::new(w.into(), h.into(), PixelType::U8x4);

        // 4. Resize directly.
        // `&src_image` fulfills the `&impl IntoImageView` requirement.
        // `&mut dst_img` fulfills the `&mut impl IntoImageViewMut` requirement.
        // We don't need to manually create views.
        resizer.resize(&src_image, &mut dst_img, None)?;

        // This will now succeed because `dst_img` contains the correctly resized RGBA data.
        let resized = ImageBuffer::<Rgba<u8>, _>::from_raw(
            target_w,
            target_h as u32,
            dst_img.into_vec(), // Use into_vec() to take ownership
        )
            .ok_or_else(|| eyre!("Failed to construct resized image from buffer"))?;

        let encoder = Encoder::new()
            .with_quality(80.0)
            .with_speed(4)
            .with_alpha_quality(80.0);

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

        let filename = format!("{target_h}p.avif");
        let output_path = output_dir.join(filename);
        fs::write(output_path, avif_data.avif_file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;
    #[test]
    fn test_generate_thumbnails() -> color_eyre::Result<()> {
        let input = Path::new("assets/tree.jpg");
        let out_dir = Path::new("new_thumbs");
        // Ensure the output directory is clean for a fresh test run
        if out_dir.exists() {
            fs::remove_dir_all(out_dir)?;
        }
        fs::create_dir_all(out_dir)?;
        let now = Instant::now();
        generate_photo_thumbnails(input, out_dir, &[240, 480, 720, 1080])?;
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        Ok(())
    }
}