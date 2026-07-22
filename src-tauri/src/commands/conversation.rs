/// Voice conversation pipeline: ASR (whisper-cli) → LLM (HTTP API) → TTS (Windows SAPI).
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use crate::commands::settings::ConfigState;

#[derive(Clone, Serialize)]
pub struct ConversationEvent {
    pub event_type: String,
    pub text: String,
    pub audio_path: Option<String>,
}

#[tauri::command]
pub async fn conversation_turn(
    app: AppHandle,
    config: State<'_, ConfigState>,
    wav_bytes: Vec<u8>,
    language: String,
    conversation_history: Vec<serde_json::Value>,
) -> Result<(), String> {
    let (asr_model, llm_api_url, llm_api_key, llm_model, llm_system_prompt, _tts_model, _tts_voice, _tts_speed) = {
        let cfg = config.inner.lock().map_err(|e| e.to_string())?;
        (cfg.asr.default_model.clone(), cfg.llm.api_url.clone(), cfg.llm.api_key.clone(),
         cfg.llm.model.clone(), cfg.conversation.llm_system_prompt.clone(),
         cfg.tts.default_model.clone(), cfg.tts.voice.clone(), cfg.tts.speed)
    };

    // Step 1: ASR via whisper-cli
    emit_conv(&app, "status", "recognizing...", None);

    let tmp = std::env::temp_dir().join(format!("swt2_conv_{}.wav", uuid::Uuid::new_v4()));
    std::fs::write(&tmp, &wav_bytes).map_err(|e| e.to_string())?;
    let tmp_str = tmp.to_string_lossy().to_string();

    let (user_text, _) = crate::commands::transcribe::call_whisper_cli(&tmp_str, &asr_model, &language).await?;
    let _ = std::fs::remove_file(&tmp);

    if user_text.trim().is_empty() {
        emit_conv(&app, "error", "no speech detected", None);
        return Ok(());
    }
    emit_conv(&app, "user_text", &user_text, None);

    // Step 2: LLM
    emit_conv(&app, "status", "thinking...", None);

    let mut messages = vec![
        serde_json::json!({"role": "system", "content": llm_system_prompt}),
    ];
    messages.extend(conversation_history);
    messages.push(serde_json::json!({"role": "user", "content": user_text}));

    let llm_body = serde_json::json!({
        "model": llm_model,
        "messages": messages,
        "stream": false,
    });

    let client = reqwest::Client::new();
    let mut llm_req = client
        .post(format!("{}/chat/completions", llm_api_url.trim_end_matches('/')))
        .json(&llm_body);
    if !llm_api_key.is_empty() {
        llm_req = llm_req.header("Authorization", format!("Bearer {}", llm_api_key));
    }

    let llm_resp = llm_req.send().await.map_err(|e| format!("LLM failed: {}", e))?;
    let llm_json: serde_json::Value = llm_resp.json().await.map_err(|e| e.to_string())?;
    let assistant_text = llm_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Sorry, I cannot respond.")
        .to_string();

    emit_conv(&app, "assistant_text", &assistant_text, None);

    // Step 3: TTS via Windows SAPI (inline, no DB access needed for audio output)
    emit_conv(&app, "status", "synthesizing...", None);

    let escaped = assistant_text.replace('"', "''");
    let ps_script = format!(
        r#"Add-Type -AssemblyName System.Speech; $s = New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.SetOutputToWaveFile('{0}\swt2_tts.wav'); $s.Speak('{1}')"#,
        std::env::temp_dir().display().to_string().replace('\\', "\\\\"),
        escaped
    );
    let ps_path = std::env::temp_dir().join(format!("swt2_conv_ps_{}.ps1", uuid::Uuid::new_v4()));
    let _ = std::fs::write(&ps_path, &ps_script);

    let _output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&ps_path)
        .output();
    let _ = std::fs::remove_file(&ps_path);

    let wav_path = std::env::temp_dir().join("swt2_tts.wav");
    if wav_path.exists() {
        let dest = std::env::temp_dir().join(format!("swt2_conv_{}.wav", uuid::Uuid::new_v4()));
        if std::fs::rename(&wav_path, &dest).is_ok() {
            let path = dest.to_string_lossy().to_string();
            emit_conv(&app, "assistant_audio", "", Some(&path));
        }
    }

    emit_conv(&app, "turn_done", "", None);
    Ok(())
}

fn emit_conv(app: &AppHandle, event_type: &str, text: &str, audio_path: Option<&str>) {
    let _ = app.emit("conversation-event", ConversationEvent {
        event_type: event_type.to_string(),
        text: text.to_string(),
        audio_path: audio_path.map(String::from),
    });
}
