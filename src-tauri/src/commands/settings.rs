use crate::config::{load_config, save_config, AppConfig};
use tauri::State;
use std::sync::Mutex;

/// In-memory config cache to avoid disk I/O on every get.
pub struct ConfigState {
    pub inner: Mutex<AppConfig>,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(load_config()),
        }
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let cfg = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(cfg.sanitized())
}

#[tauri::command]
pub fn set_config(
    state: State<'_, ConfigState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let mut cfg = state.inner.lock().map_err(|e| e.to_string())?;
    set_nested(&mut cfg, &key, value)?;
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reset_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let defaults = AppConfig::default();
    {
        let mut cfg = state.inner.lock().map_err(|e| e.to_string())?;
        *cfg = defaults.clone();
    }
    save_config(&defaults).map_err(|e| e.to_string())?;
    Ok(defaults)
}

/// Set a nested config value using dot-notation keys like "asr.device" or "ui.theme".
fn set_nested(cfg: &mut AppConfig, key: &str, value: serde_json::Value) -> Result<(), String> {
    match key {
        "asr.default_model" => cfg.asr.default_model = str_val(value)?,
        "asr.device" => cfg.asr.device = str_val(value)?,
        "asr.language" => cfg.asr.language = str_val(value)?,
        "tts.default_model" => cfg.tts.default_model = str_val(value)?,
        "tts.voice" => cfg.tts.voice = str_val(value)?,
        "tts.speed" => cfg.tts.speed = num_val(value)?,
        "ui.theme" => cfg.ui.theme = str_val(value)?,
        "ui.language" => cfg.ui.language = str_val(value)?,
        "llm.api_url" => cfg.llm.api_url = str_val(value)?,
        "llm.api_key" => cfg.llm.api_key = str_val(value)?,
        "llm.model" => cfg.llm.model = str_val(value)?,
        "llm.enabled" => cfg.llm.enabled = bool_val(value)?,
        "llm.correction_prompt" => cfg.llm.correction_prompt = str_val(value)?,
        "export.output_dir" => cfg.export.output_dir = str_val(value)?,
        "advanced.auto_save_history" => cfg.advanced.auto_save_history = bool_val(value)?,
        "api_server.port" => cfg.api_server.port = num_val::<u16>(value)?,
        "api_server.api_key" => cfg.api_server.api_key = str_val(value)?,
        "conversation.llm_system_prompt" => cfg.conversation.llm_system_prompt = str_val(value)?,
        "conversation.vad_mode" => cfg.conversation.vad_mode = str_val(value)?,
        "conversation.vad_energy_threshold" => cfg.conversation.vad_energy_threshold = num_val(value)?,
        "voice_input.enabled" => cfg.voice_input.enabled = bool_val(value)?,
        "voice_input.hotkey" => cfg.voice_input.hotkey = str_val(value)?,
        "vad.min_silence_duration_ms" => cfg.vad.min_silence_duration_ms = num_val(value)?,
        "vad.max_segment_time" => cfg.vad.max_segment_time = num_val(value)?,
        "enhance.denoise" => cfg.enhance.denoise = str_val(value)?,
        "enhance.aec" => cfg.enhance.aec = str_val(value)?,
        "enhance.separation" => cfg.enhance.separation = str_val(value)?,
        "enhance.punctuation" => cfg.enhance.punctuation = str_val(value)?,
        "system.initialized" => cfg.system.initialized = bool_val(value)?,
        "updater.auto_check" => cfg.updater.auto_check = bool_val(value)?,
        _ => return Err(format!("Unknown config key: {}", key)),
    }
    Ok(())
}

fn str_val(v: serde_json::Value) -> Result<String, String> {
    match v {
        serde_json::Value::String(s) => Ok(s),
        _ => Err("Expected string value".into()),
    }
}

fn num_val<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(v).map_err(|e| e.to_string())
}

fn bool_val(v: serde_json::Value) -> Result<bool, String> {
    match v {
        serde_json::Value::Bool(b) => Ok(b),
        _ => Err("Expected boolean value".into()),
    }
}
