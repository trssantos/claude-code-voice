use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;

const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

pub fn get_model_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let model_dir = home.join(".claude-code-voice").join("models");
    fs::create_dir_all(&model_dir)?;
    Ok(model_dir)
}

pub fn get_model_path(model_size: &str) -> Result<PathBuf> {
    let model_dir = get_model_dir()?;
    Ok(model_dir.join(format!("ggml-{}.bin", model_size)))
}

pub async fn is_model_downloaded(model_size: &str) -> Result<bool> {
    let model_path = get_model_path(model_size)?;
    Ok(model_path.exists())
}

pub async fn download_model(model_size: &str) -> Result<()> {
    let model_filename = format!("ggml-{}.bin", model_size);
    let url = format!("{}/{}", MODEL_BASE_URL, model_filename);
    let model_path = get_model_path(model_size)?;

    info!("Downloading model from: {}", url);
    info!("Saving to: {}", model_path.display());

    // Create a temporary file
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("{}.tmp", model_filename));

    // Download with progress
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let mut response = client
        .get(&url)
        .send()
        .context("Failed to download model")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download model: HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    let mut file = fs::File::create(&temp_path)?;

    info!("Downloading {} bytes...", total_size);

    use std::io::Read;
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        if total_size > 0 && downloaded % (1024 * 1024) == 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            info!("Progress: {:.1}%", progress);
        }
    }

    file.sync_all()?;
    drop(file);

    // Move temp file to final location
    fs::rename(&temp_path, &model_path)?;

    info!("Model downloaded successfully");
    Ok(())
}
