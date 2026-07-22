/// Real-time ASR streaming via whisper-cli.
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct StreamEvent {
    pub event_type: String,
    pub text: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub confidence: Option<f64>,
}

#[tauri::command]
pub async fn stream_transcribe(
    app: AppHandle,
    wav_bytes: Vec<u8>,
    model_id: String,
    language: String,
) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("swt2_stream_{}.wav", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, &wav_bytes).map_err(|e| e.to_string())?;
    let tmp_str = tmp.to_string_lossy().to_string();

    match crate::commands::transcribe::call_whisper_cli(&tmp_str, &model_id, &language).await {
        Ok((text, _segments)) => {
            emit_stream(&app, "final", &text, None, None, None);
            emit_stream(&app, "done", "", None, None, None);
        }
        Err(e) => {
            emit_stream(&app, "error", &e, None, None, None);
        }
    }

    let _ = std::fs::remove_file(&tmp);
    Ok(())
}

fn emit_stream(app: &AppHandle, event_type: &str, text: &str, start: Option<f64>, end: Option<f64>, confidence: Option<f64>) {
    let _ = app.emit("stream-event", StreamEvent {
        event_type: event_type.to_string(),
        text: text.to_string(),
        start,
        end,
        confidence,
    });
}
