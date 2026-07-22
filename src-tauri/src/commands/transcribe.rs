/// File transcription via whisper-cli (one-shot CLI, no server).
use crate::db::{Database, HistoryRecord};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub text: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    #[serde(default)]
    pub speaker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub file_path: String,
    pub file_name: String,
    pub file_duration: f64,
    pub full_text: String,
    pub segments: Vec<Segment>,
    pub model_name: String,
    pub language: String,
}

#[derive(Clone, Serialize)]
pub struct BatchProgress {
    pub file_index: usize,
    pub total_files: usize,
    pub file_name: String,
    pub percentage: f64,
    pub state: String,
    pub message: String,
}

// ── Tool resolution ────────────────────────────────────────────

/// Unified tool finder — checks PATH first, then bundled locations.
pub fn find_tool(name: &str) -> Option<PathBuf> {
    let exe_name = format!("{}.exe", name);
    // Check PATH first
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let full = dir.join(&exe_name);
            if full.exists() {
                return Some(full);
            }
        }
    }
    // Check bundled locations
    let exe_dir = std::env::current_exe().ok()?;
    let dir = exe_dir.parent()?.to_path_buf();
    let candidates = [
        dir.join("tools").join(&exe_name),
        dir.join(&exe_name),
        dir.parent()?.join("swt").join("tools").join(&exe_name),
        dir.join("binaries").join(&exe_name),
    ];
    for c in &candidates {
        if c.exists() {
            return Some(c.clone());
        }
    }
    None
}

fn find_whisper_cli() -> Option<PathBuf> {
    crate::sidecar::find_whisper_cli()
}

/// Map swt2 model IDs to whisper.cpp model files.
fn resolve_model_path(model_id: &str) -> Result<PathBuf, String> {
    // Try llm/ directory first
    let llm = crate::model_registry::llm_dir();

    // Direct GGUF path
    let model_dir_name = match model_id {
        id if id.contains("0.6B") || id.contains("0.6b") => "whisper-base",
        id if id.contains("1.7B") || id.contains("1.7b") => "whisper-small",
        id if id.to_lowercase().contains("nemotron") => "whisper-small",
        id if id.to_lowercase().contains("citrinet") => "whisper-tiny",
        id if id.to_lowercase().contains("higgs") => "whisper-small",
        _ => "whisper-base",
    };

    let candidate = llm.join(model_dir_name).join("ggml-base.bin");
    if candidate.exists() {
        return Ok(candidate);
    }
    // Try ggml-small.bin if base doesn't exist
    let small = llm.join(model_dir_name).join("ggml-small.bin");
    if small.exists() {
        return Ok(small);
    }
    // Fallback: any ggml-*.bin in the model directory
    if let Ok(entries) = std::fs::read_dir(llm.join(model_dir_name)) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("ggml-") && name.ends_with(".bin") {
                return Ok(e.path());
            }
        }
    }
    Err(format!(
        "Model not found. Download a whisper model to {}/{}",
        llm.display(),
        model_dir_name
    ))
}

// ── Audio extraction (pure Rust, ffmpeg fallback) ─────────────

fn extract_audio(input: &str, output: &str) -> Result<(), String> {
    crate::audio::extract_audio(input, output)
}

fn needs_extraction(path: &str) -> bool {
    crate::audio::needs_extraction(path)
}

fn get_duration(path: &str) -> f64 {
    // Try symphonia first for supported formats
    if let Ok(dur) = crate::audio::get_duration_symphonia(path) {
        return dur;
    }
    // ffprobe fallback
    if let Some(probe) = find_tool("ffprobe") {
        if let Ok(output) = Command::new(&probe)
            .args(["-v", "quiet", "-show_entries", "format=duration", "-of", "csv=p=0", path])
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(d) = s.parse::<f64>() {
                return d;
            }
        }
    }
    0.0
}

// ── Whisper CLI transcription ─────────────────────────────────

/// Parse a "HH:MM:SS,mmm" timestamp to seconds.
fn parse_timestamp(ts: &str) -> f64 {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() == 3 {
        let secs: Vec<&str> = parts[2].split(',').collect();
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let s: f64 = secs.first().unwrap_or(&"0").parse().unwrap_or(0.0);
        let ms: f64 = secs.get(1).unwrap_or(&"0").parse().unwrap_or(0.0);
        return h * 3600.0 + m * 60.0 + s + ms / 1000.0;
    }
    0.0
}

pub async fn call_whisper_cli(
    audio_path: &str,
    model_id: &str,
    language: &str,
) -> Result<(String, Vec<Segment>), String> {
    let whisper_exe = find_whisper_cli().ok_or("whisper-cli not found")?;
    let model_path = resolve_model_path(model_id)?;
    let lang = if language == "auto" { "auto".to_string() } else { language.to_string() };

    // Add DLL dir to PATH so whisper-cli finds its CPU backend DLLs
    let dll_dir = whisper_exe.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();

    let out_base = std::env::temp_dir().join(format!("swt2_whisper_{}", uuid::Uuid::new_v4()));
    let out_base_str = out_base.to_string_lossy().to_string();
    let json_path = format!("{}.json", out_base_str);

    let mut cmd = Command::new(&whisper_exe);
    cmd.args([
        "--model", &model_path.to_string_lossy(),
        "--file", audio_path,
        "--output-json",
        "-of", &out_base_str,
        "--language", &lang,
        "--threads", "4",
    ]);
    // Set PATH for DLL resolution
    if let Ok(current_path) = std::env::var("PATH") {
        cmd.env("PATH", format!("{};{}", dll_dir.display(), current_path));
    }

    let output = cmd.output().map_err(|e| format!("whisper-cli failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("whisper-cli error: {}", stderr));
    }

    // Read JSON output
    let json_str = std::fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read output: {}", e))?;

    // Clean up temp files
    let _ = std::fs::remove_file(&json_path);

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    let transcription = parsed["transcription"].as_array();

    let segments: Vec<Segment> = transcription
        .map(|arr| {
            arr.iter()
                .filter_map(|seg| {
                    let text = seg["text"].as_str()?.to_string();
                    let timestamps = &seg["timestamps"];
                    let start = parse_timestamp(timestamps["from"].as_str()?);
                    let end = parse_timestamp(timestamps["to"].as_str()?);
                    Some(Segment {
                        text,
                        start,
                        end,
                        confidence: 0.0,
                        speaker: String::new(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let full_text = segments.iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    Ok((full_text, segments))
}

// ── Tauri commands ────────────────────────────────────────────

#[tauri::command]
pub async fn transcribe_file(
    app: AppHandle,
    db: State<'_, Database>,
    file_path: String,
    model_id: String,
    language: String,
) -> Result<TranscriptionResult, String> {
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let audio_path = if needs_extraction(&file_path) {
        emit_batch(&app, 0, 1, &file_name, 10.0, "extracting", "extracting audio...");
        let tmp = std::env::temp_dir().join(format!("swt2_{}.wav", uuid::Uuid::new_v4()));
        let tmp_str = tmp.to_string_lossy().to_string();
        extract_audio(&file_path, &tmp_str)?;
        tmp_str
    } else {
        file_path.clone()
    };

    emit_batch(&app, 0, 1, &file_name, 30.0, "transcribing", "transcribing...");
    let (full_text, segments) = call_whisper_cli(&audio_path, &model_id, &language).await?;

    if audio_path != file_path {
        let _ = std::fs::remove_file(&audio_path);
    }

    let duration = get_duration(&file_path);

    let result = TranscriptionResult {
        file_path: file_path.clone(),
        file_name: file_name.clone(),
        file_duration: duration,
        full_text: full_text.clone(),
        segments: segments.clone(),
        model_name: model_id.clone(),
        language: language.clone(),
    };

    emit_batch(&app, 0, 1, &file_name, 100.0, "done", "done");

    let word_count = full_text.chars().count() as i64;
    let segments_json = serde_json::to_string(&segments).unwrap_or_default();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = db.insert(&HistoryRecord {
        id: 0,
        source: "file".into(),
        file_name: file_name.clone(),
        file_duration: duration,
        model_name: model_id.clone(),
        language: language.clone(),
        full_text: full_text.clone(),
        word_count,
        segments_json,
        created_at: now,
    });

    Ok(result)
}

#[tauri::command]
pub async fn transcribe_batch(
    app: AppHandle,
    db: State<'_, Database>,
    file_paths: Vec<String>,
    model_id: String,
    language: String,
) -> Result<Vec<TranscriptionResult>, String> {
    let total = file_paths.len();
    let mut results = Vec::with_capacity(total);

    for (i, path) in file_paths.iter().enumerate() {
        let file_name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        emit_batch(&app, i, total, &file_name, 0.0, "transcribing", "starting...");

        match transcribe_file(
            app.clone(),
            db.clone(),
            path.clone(),
            model_id.clone(),
            language.clone(),
        )
        .await
        {
            Ok(r) => {
                emit_batch(&app, i, total, &file_name, 100.0, "done", "done");
                results.push(r);
            }
            Err(e) => {
                emit_batch(&app, i, total, &file_name, 0.0, "error", &e);
                results.push(TranscriptionResult {
                    file_path: path.clone(),
                    file_name: file_name.clone(),
                    file_duration: 0.0,
                    full_text: format!("[ERROR] {}", e),
                    segments: vec![],
                    model_name: model_id.clone(),
                    language: language.clone(),
                });
            }
        }
    }

    let _ = app.emit("batch-all-done", total);
    Ok(results)
}

fn emit_batch(app: &AppHandle, index: usize, total: usize, name: &str, pct: f64, state: &str, msg: &str) {
    let _ = app.emit("batch-progress", BatchProgress {
        file_index: index,
        total_files: total,
        file_name: name.to_string(),
        percentage: pct,
        state: state.to_string(),
        message: msg.to_string(),
    });
}
