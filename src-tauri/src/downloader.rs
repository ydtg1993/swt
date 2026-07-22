/// HuggingFace GGUF model downloader.
use crate::model_registry::{llm_dir, ModelInfo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub state: String,
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

pub async fn download_model(
    app: AppHandle,
    info: ModelInfo,
    cancel_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    let model_dir = llm_dir().join(&info.local_dir_name);
    std::fs::create_dir_all(&model_dir).map_err(|e| e.to_string())?;

    let gguf_url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        info.gguf_repo_id, info.gguf_filename
    );
    download_file(&app, &info.id, &gguf_url, &model_dir.join(&info.gguf_filename), &cancel_flag).await?;

    if info.download_mode == "gguf_multi" && !info.config_repo_id.is_empty() {
        for config_file in &info.config_filenames {
            let config_url = format!(
                "https://huggingface.co/{}/resolve/main/{}",
                info.config_repo_id, config_file
            );
            let _ = download_file(&app, &info.id, &config_url, &model_dir.join(config_file), &cancel_flag).await;
        }
    }

    emit_progress(&app, &info.id, 0, 0, 100.0, "complete");
    Ok(())
}

async fn download_file(
    app: &AppHandle,
    model_id: &str,
    url: &str,
    dest: &PathBuf,
    cancel: &Arc<AtomicBool>,
) -> Result<(), String> {
    let client = http_client();

    // HEAD request
    let head_resp = client.head(url).send().await.map_err(|e| e.to_string())?;
    let total_size = head_resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    // Re-check file size right before deciding
    let current_size = if dest.exists() {
        std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };
    if current_size >= total_size && total_size > 0 {
        emit_progress(app, model_id, total_size, total_size, 100.0, "complete");
        return Ok(());
    }

    let mut request = client.get(url);
    if current_size > 0 {
        request = request.header("Range", format!("bytes={}-", current_size));
    }

    let resp = request.send().await.map_err(|e| e.to_string())?;

    // Check status
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Download failed ({}): {}", status, body));
    }

    // If server returned 200 instead of 206, restart from scratch
    let append = status.as_u16() == 206 && current_size > 0;
    let starting_bytes = if append { current_size } else { 0 };

    // Chunked read with cancel check
    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(!append)
        .open(dest)
        .map_err(|e| e.to_string())?;

    let mut downloaded = starting_bytes;
    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let bytes = chunk.map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
        downloaded += bytes.len() as u64;
        let pct = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        emit_progress(app, model_id, downloaded, total_size, pct, "downloading");
    }

    Ok(())
}

fn emit_progress(app: &AppHandle, model_id: &str, downloaded: u64, total: u64, pct: f64, state: &str) {
    let _ = app.emit("download-progress", DownloadProgress {
        model_id: model_id.to_string(),
        downloaded_bytes: downloaded,
        total_bytes: total,
        percentage: pct,
        state: state.to_string(),
    });
}
