/// Text-to-Speech via Windows SAPI (built-in, no server needed).
use tauri::State;
use crate::db::{Database, HistoryRecord};

#[tauri::command]
pub async fn synthesize(
    db: State<'_, Database>,
    text: String,
    model_id: String,
    _voice: String,
    _speed: f64,
) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("Text is empty".into());
    }

    // Use Windows SAPI via PowerShell
    let escaped = text.replace('"', "''");
    let ps_script = format!(
        r#"Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.SetOutputToWaveFile('{0}\swt2_tts.wav'); $s.Speak('{1}')"#,
        std::env::temp_dir().display().to_string().replace('\\', "\\\\"),
        escaped
    );
    let ps_path = std::env::temp_dir().join(format!("swt2_tts_{}.ps1", uuid::Uuid::new_v4()));
    std::fs::write(&ps_path, &ps_script).map_err(|e| e.to_string())?;

    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&ps_path)
        .output()
        .map_err(|e| format!("TTS failed: {}", e))?;

    let _ = std::fs::remove_file(&ps_path);

    if !output.status.success() {
        return Err("TTS synthesis failed".into());
    }

    let wav_path = std::env::temp_dir().join("swt2_tts.wav");
    let dest = std::env::temp_dir().join(format!("swt2_tts_{}.wav", uuid::Uuid::new_v4()));
    std::fs::rename(&wav_path, &dest).map_err(|e| e.to_string())?;
    let path = dest.to_string_lossy().to_string();

    // Save to history
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let _ = db.insert(&HistoryRecord {
        id: 0,
        source: "tts".into(),
        file_name: format!("TTS_{}", now),
        file_duration: 0.0,
        model_name: model_id,
        language: "auto".into(),
        full_text: text.clone(),
        word_count: text.chars().count() as i64,
        segments_json: "[]".into(),
        created_at: now,
    });

    Ok(path)
}

#[tauri::command]
pub async fn get_voices(_model_id: String) -> Result<Vec<String>, String> {
    Ok(vec!["default".into()])
}
