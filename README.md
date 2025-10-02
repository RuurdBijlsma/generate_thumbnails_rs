# Thumbnail Generator

Simple Rust scripts for generating thumbnails from images and videos using FFmpeg.

## Features

- **Image thumbnails:** Generate multiple sizes from a single source image in one FFmpeg call.
- **Video thumbnails:** Generate multiple thumbnails:
  - At multiple resolutions from a single timestamp.
  - At multiple timestamps with a single resolution.
- Async execution using Tokio.

## Usage

### Image Thumbnails

```rust
generate_image_thumbnails(
    Path::new("assets/PICT0008.JPG"),
    Path::new("image_thumbnails"),
    &[240, 480, 1080],
    "avif",
).await?;
````

### Video Thumbnails

```rust
generate_video_thumbnail_series(
    Path::new("assets/jellyfish.mp4"),
    Path::new("video_thumbnails_series"),
    "avif",
    &[240, 480, 1080], // Multi-size heights
    0.5,               // Multi-size timestamp
    &[10.0, 30.0, 50.0], // Multi-time timestamps
    720,               // Multi-time height
).await?;
```

## Requirements

* Rust with Tokio
* FFmpeg installed and available in `PATH`

## Notes

* Paths are converted using `to_string_lossy()` for safety.
* FFmpeg errors are captured and returned using `anyhow`.
* Image thumbnails use a single call with `split` and `scale`.
* Video thumbnails support complex combinations of sizes and timestamps in a single FFmpeg call.
