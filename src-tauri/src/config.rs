/// Application configuration — mirrors swt/config/config.dist.yaml.
///
/// Loaded from config.yaml next to the executable; falls back to embedded
/// defaults if the file is missing.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub asr: AsrSection,
    pub vad: VadSection,
    pub tts: TtsSection,
    pub realtime_asr: RealtimeAsrSection,
    pub api_server: ApiServerSection,
    pub enhance: EnhanceSection,
    pub export: ExportSection,
    pub ui: UiSection,
    pub advanced: AdvancedSection,
    pub llm: LlmSection,
    pub gpu: GpuSection,
    pub system: SystemSection,
    pub updater: UpdaterSection,
    pub conversation: ConversationSection,
    pub voice_input: VoiceInputSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AsrSection {
    pub default_model: String,
    pub default_model_cpp: String,
    pub model_path: String,
    pub device: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VadSection {
    pub model: String,
    pub min_silence_duration_ms: u32,
    pub max_segment_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TtsSection {
    pub default_model: String,
    pub voice: String,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RealtimeAsrSection {
    pub sample_rate: u32,
    pub frame_ms: u32,
    pub language: String,
    pub partial_throttle_ms: u32,
    pub vad: ConversationVadSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversationVadSection {
    pub mode: String,
    pub energy_threshold: f64,
    pub min_speech_ms: u32,
    pub min_silence_ms: u32,
    #[serde(default)]
    pub max_speech_s: f64,
    #[serde(default)]
    pub speech_pad_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiServerSection {
    pub port: u16,
    pub cpp_port: u16,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EnhanceSection {
    pub denoise: String,
    pub aec: String,
    pub separation: String,
    pub punctuation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSection {
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSection {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedSection {
    pub auto_save_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmSection {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
    pub correction_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GpuSection {
    pub pack_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SystemSection {
    pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdaterSection {
    pub auto_check: bool,
    pub github_owner: String,
    pub github_repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversationSection {
    pub llm_system_prompt: String,
    pub vad_mode: String,
    pub vad_energy_threshold: f64,
    pub vad_min_speech_ms: u32,
    pub vad_min_silence_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceInputSection {
    pub enabled: bool,
    pub hotkey: String,
    pub vad_mode: String,
    pub energy_threshold: f64,
}

// ── Default impls ──────────────────────────────────────────────

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            asr: AsrSection::default(),
            vad: VadSection::default(),
            tts: TtsSection::default(),
            realtime_asr: RealtimeAsrSection::default(),
            api_server: ApiServerSection::default(),
            enhance: EnhanceSection::default(),
            export: ExportSection::default(),
            ui: UiSection::default(),
            advanced: AdvancedSection::default(),
            llm: LlmSection::default(),
            gpu: GpuSection::default(),
            system: SystemSection::default(),
            updater: UpdaterSection::default(),
            conversation: ConversationSection::default(),
            voice_input: VoiceInputSection::default(),
        }
    }
}

impl Default for AsrSection {
    fn default() -> Self {
        Self { default_model: "Qwen/Qwen3-ASR-0.6B-GGUF".into(), default_model_cpp: "Qwen/Qwen3-ASR-0.6B-GGUF".into(), model_path: "llm".into(), device: "cuda:0".into(), language: "auto".into() }
    }
}
impl Default for VadSection {
    fn default() -> Self { Self { model: "".into(), min_silence_duration_ms: 100, max_segment_time: 60000 } }
}
impl Default for TtsSection {
    fn default() -> Self { Self { default_model: "Qwen/Qwen3-TTS-0.6B-GGUF".into(), voice: "default".into(), speed: 1.0 } }
}
impl Default for RealtimeAsrSection {
    fn default() -> Self { Self { sample_rate: 16000, frame_ms: 20, language: "auto".into(), partial_throttle_ms: 80, vad: ConversationVadSection::default() } }
}
impl Default for ConversationVadSection {
    fn default() -> Self { Self { mode: "energy".into(), energy_threshold: 0.02, min_speech_ms: 250, min_silence_ms: 500, max_speech_s: 15.0, speech_pad_ms: 160 } }
}
impl Default for ApiServerSection {
    fn default() -> Self { Self { port: 8000, cpp_port: 8080, api_key: "".into() } }
}
impl Default for EnhanceSection {
    fn default() -> Self { Self { denoise: "".into(), aec: "".into(), separation: "".into(), punctuation: "".into() } }
}
impl Default for ExportSection {
    fn default() -> Self { Self { output_dir: "".into() } }
}
impl Default for UiSection {
    fn default() -> Self { Self { theme: "light".into(), language: "zh".into() } }
}
impl Default for AdvancedSection {
    fn default() -> Self { Self { auto_save_history: true } }
}
impl Default for LlmSection {
    fn default() -> Self { Self { api_url: "https://api.deepseek.com/v1".into(), api_key: "".into(), model: "deepseek-v4-flash".into(), enabled: false, correction_prompt: "".into() } }
}
impl Default for GpuSection {
    fn default() -> Self { Self { pack_url: "".into() } }
}
impl Default for SystemSection {
    fn default() -> Self { Self { initialized: false } }
}
impl Default for UpdaterSection {
    fn default() -> Self { Self { auto_check: true, github_owner: "".into(), github_repo: "".into() } }
}
impl Default for ConversationSection {
    fn default() -> Self { Self { llm_system_prompt: "你是一个友好的语音助手。请用简洁的中文回答。".into(), vad_mode: "energy".into(), vad_energy_threshold: 0.02, vad_min_speech_ms: 250, vad_min_silence_ms: 500 } }
}
impl Default for VoiceInputSection {
    fn default() -> Self { Self { enabled: false, hotkey: "Ctrl+Alt+Space".into(), vad_mode: "energy".into(), energy_threshold: 0.02 } }
}

// ── Config file path ─────────────────────────────────────────

fn config_path() -> PathBuf {
    // Next to the executable, or in CWD for development
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.yaml")))
        .unwrap_or_else(|| PathBuf::from("config.yaml"))
}

/// Load config from config.yaml, falling back to defaults.
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(yaml) => {
                match serde_yaml::from_str(&yaml) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("[config] parse error: {}, using defaults", e),
                }
            }
            Err(e) => eprintln!("[config] read error: {}, using defaults", e),
        }
    }
    AppConfig::default()
}

/// Return a copy with sensitive fields masked for the frontend.
impl AppConfig {
    pub fn sanitized(&self) -> Self {
        let mut c = self.clone();
        if !c.llm.api_key.is_empty() {
            c.llm.api_key = "••••••••".into();
        }
        if !c.api_server.api_key.is_empty() {
            c.api_server.api_key = "••••••••".into();
        }
        c
    }
}

/// Save config to config.yaml (atomic write).
pub fn save_config(cfg: &AppConfig) -> Result<(), crate::error::AppError> {
    let yaml = serde_yaml::to_string(cfg)?;
    let path = config_path();
    let tmp = path.with_extension("yaml.tmp");
    std::fs::write(&tmp, &yaml).map_err(|e| crate::error::AppError::config(e.to_string()))?;
    std::fs::rename(&tmp, &path).map_err(|e| crate::error::AppError::config(e.to_string()))?;
    Ok(())
}
