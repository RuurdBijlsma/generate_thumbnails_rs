use std::ffi::OsStr;
use std::process::Stdio;
use anyhow::Context;
use tokio::process::Command;

pub async fn run_ffmpeg<S: AsRef<OsStr>>(args: &[S]) -> anyhow::Result<()> {
    let output = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("failed to run ffmpeg")?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {}", stderr.trim());
    }
}