/// Static model catalog — mirrors audio.cpp supported models.
/// GGUF models: downloadable. safetensors models: require audiocpp_server build.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub detail_description: String,
    pub languages: Vec<String>,
    pub estimated_size: String,
    pub gpu_requirement: String,
    pub cpu_suitable: String,
    pub category: String,
    pub model_family: String,
    pub local_dir_name: String,
    pub gguf_repo_id: String,
    pub gguf_filename: String,
    pub downloaded: bool,
    #[serde(default)]
    pub download_mode: String,
    #[serde(default)]
    pub config_repo_id: String,
    #[serde(default)]
    pub config_filenames: Vec<String>,
}

pub fn all_models() -> &'static HashMap<String, ModelInfo> {
    use std::sync::OnceLock;
    static CATALOG: OnceLock<HashMap<String, ModelInfo>> = OnceLock::new();
    CATALOG.get_or_init(|| build_catalog())
}

fn m_asr(
    id: &str, name: &str, desc: &str, detail: &str, langs: &[&str],
    size: &str, gpu: &str, cpu: &str, family: &str, dir: &str,
    repo: &str, file: &str, dm: &str, cfg_repo: &str, cfg_files: &[&str],
) -> ModelInfo {
    ModelInfo {
        id: id.into(), name: name.into(), description: desc.into(),
        detail_description: detail.into(),
        languages: langs.iter().map(|s| s.to_string()).collect(),
        estimated_size: size.into(), gpu_requirement: gpu.into(),
        cpu_suitable: cpu.into(), category: "asr_cpp".into(),
        model_family: family.into(), local_dir_name: dir.into(),
        gguf_repo_id: repo.into(), gguf_filename: file.into(),
        downloaded: false, download_mode: dm.into(),
        config_repo_id: cfg_repo.into(),
        config_filenames: cfg_files.iter().map(|s| s.to_string()).collect(),
    }
}

fn m_tts(
    id: &str, name: &str, desc: &str, detail: &str, langs: &[&str],
    size: &str, gpu: &str, cpu: &str, family: &str, dir: &str,
    repo: &str, file: &str, dm: &str, cfg_repo: &str, cfg_files: &[&str],
) -> ModelInfo {
    let mut m = m_asr(id, name, desc, detail, langs, size, gpu, cpu, family, dir, repo, file, dm, cfg_repo, cfg_files);
    m.category = "tts".into();
    m
}

fn m_other(
    cat: &str, id: &str, name: &str, desc: &str, detail: &str, langs: &[&str],
    size: &str, gpu: &str, cpu: &str, family: &str, dir: &str,
    repo: &str, file: &str, dm: &str, cfg_repo: &str, cfg_files: &[&str],
) -> ModelInfo {
    let mut m = m_asr(id, name, desc, detail, langs, size, gpu, cpu, family, dir, repo, file, dm, cfg_repo, cfg_files);
    m.category = cat.into();
    m
}

fn build_catalog() -> HashMap<String, ModelInfo> {
    let mut m = HashMap::new();

    // ═══════════════════════════════════════════════════════════
    //  ASR (automatic speech recognition)
    // ═══════════════════════════════════════════════════════════

    m.insert("Qwen/Qwen3-ASR-0.6B-GGUF".into(), m_asr(
        "Qwen/Qwen3-ASR-0.6B-GGUF", "Qwen3-ASR 0.6B (GGUF)",
        "Qwen3-ASR 轻量, 30+ 语言",
        "Qwen3-ASR 0.6B GGUF\n• 30 语言: 中/英/日/韩/粤/法/德/俄...\n• CUDA/CPU 多后端",
        &["zh","en","ja","ko","yue","fr","de","ru","ar","es","pt","id","it","th","vi","tr","hi","ms","nl","sv","da","fi","pl","cs","fil","fa","el","ro","hu","mk"],
        "~1.0 GB", "2GB+", "流畅", "qwen3_asr", "Qwen3-ASR-0.6B-GGUF",
        "Qwen/Qwen3-ASR-0.6B-GGUF", "qwen3-asr-0.6b.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Qwen/Qwen3-ASR-1.7B-GGUF".into(), m_asr(
        "Qwen/Qwen3-ASR-1.7B-GGUF", "Qwen3-ASR 1.7B (GGUF)",
        "Qwen3-ASR 高精度, 30+ 语言",
        "Qwen3-ASR 1.7B GGUF\n• 30 语言\n• 高精度中文识别, 内置标点",
        &["zh","en","ja","ko","yue","fr","de","ru","ar","es","pt","id","it","th","vi","tr","hi","ms","nl","sv","da","fi","pl","cs","fil","fa","el","ro","hu","mk"],
        "~2.4 GB", "6GB+", "可运行但慢", "qwen3_asr", "Qwen3-ASR-1.7B-GGUF",
        "Qwen/Qwen3-ASR-1.7B-GGUF", "qwen3-asr-1.7b.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Nvidia/Nemotron-ASR-GGUF".into(), m_asr(
        "Nvidia/Nemotron-ASR-GGUF", "Nemotron 3.5 ASR (GGUF)",
        "NVIDIA Nemotron 3.5 ASR, 100+ 语言, 流式",
        "NVIDIA Nemotron 3.5 ASR 0.6B GGUF\n• 100+ 语言\n• 流式转写支持\n• GGUF + configs",
        &["auto","zh","en","ja","ko","yue","fr","de","ru"],
        "~1.2 GB", "4GB+", "流畅", "nemotron_asr", "Nemotron-ASR-GGUF",
        "0x3/nemotron-3.5-asr-streaming-0.6b-GGUF", "nemotron-3.5-asr-streaming-0.6b-q4_k.gguf",
        "gguf_multi", "nvidia/nemotron-3.5-asr-streaming-0.6b",
        &["config.json","processor_config.json","tokenizer.json"],
    ));

    m.insert("Higgs/Higgs-Audio-STT-GGUF".into(), m_asr(
        "Higgs/Higgs-Audio-STT-GGUF", "Higgs Audio STT (GGUF)",
        "Higgs Audio v3 STT, 英语",
        "Higgs Audio v3 STT GGUF\n• 英语专精",
        &["en"],
        "~2.4 GB", "6GB+", "可运行但慢", "higgs_audio_stt", "Higgs-Audio-STT-GGUF",
        "Higgs/Higgs-Audio-STT-GGUF", "higgs-audio-stt-1.7b.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Nvidia/Citrinet-256-GGUF".into(), m_asr(
        "Nvidia/Citrinet-256-GGUF", "Citrinet-256 (GGUF)",
        "NVIDIA Citrinet-256, 轻量快速",
        "NVIDIA Citrinet-256 GGUF\n• 极轻量\n• 适合低配设备",
        &["en"],
        "~0.5 GB", "1GB+", "流畅", "citrinet_asr", "Citrinet-256-GGUF",
        "Nvidia/Citrinet-256-GGUF", "citrinet-256.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Hviske/Hviske-ASR-GGUF".into(), m_asr(
        "Hviske/Hviske-ASR-GGUF", "Hviske ASR v5.3 (GGUF)",
        "Hviske ASR v5.3, 丹麦语",
        "Hviske ASR v5.3 GGUF\n• 丹麦语专精",
        &["da"],
        "~1.0 GB", "2GB+", "流畅", "hviske_asr", "Hviske-ASR-GGUF",
        "audio-cpp/audio.cpp-gguf", "hviske-asr-v5.3.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("VibeVoice/VibeVoice-ASR-GGUF".into(), m_asr(
        "VibeVoice/VibeVoice-ASR-GGUF", "VibeVoice ASR (GGUF)",
        "VibeVoice ASR, 自动语言检测",
        "VibeVoice ASR GGUF\n• 自动语言检测\n• 多语言支持",
        &["auto","zh","en"],
        "~1.5 GB", "4GB+", "可运行", "vibevoice_asr", "VibeVoice-ASR-GGUF",
        "audio-cpp/audio.cpp-gguf", "vibevoice-asr.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Voxtral/Voxtral-Realtime-GGUF".into(), m_asr(
        "Voxtral/Voxtral-Realtime-GGUF", "Voxtral Mini 4B (GGUF)",
        "Voxtral Mini 4B Realtime, 流式 ASR, 11x 实时",
        "Voxtral Mini 4B Realtime GGUF Q8_0\n• 纯 CPU RTF 0.064 (15.7x)\n• CUDA TTFT ~171 ms\n• 自动语言",
        &["auto"],
        "~4 GB", "8GB+", "流畅 (15.7x实时)", "voxtral_realtime", "Voxtral-Realtime-GGUF",
        "audio-cpp/audio.cpp-gguf", "voxtral-mini-4b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  TTS (text to speech)
    // ═══════════════════════════════════════════════════════════

    m.insert("Qwen/Qwen3-TTS-0.6B-GGUF".into(), m_tts(
        "Qwen/Qwen3-TTS-0.6B-GGUF", "Qwen3-TTS 0.6B (GGUF)",
        "Qwen3-TTS 轻量, 10+ 语言",
        "Qwen3-TTS 0.6B\n• 10+ 语言\n• 语音合成/克隆/设计",
        &["zh","en","fr","de","it","ja","ko","pt","ru","es"],
        "~1.0 GB", "2GB+", "流畅", "qwen3_tts", "Qwen3-TTS-0.6B-GGUF",
        "Qwen/Qwen3-TTS-0.6B-GGUF", "qwen3-tts-0.6b.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Qwen/Qwen3-TTS-1.7B-GGUF".into(), m_tts(
        "Qwen/Qwen3-TTS-1.7B-GGUF", "Qwen3-TTS 1.7B (GGUF)",
        "Qwen3-TTS 高音质, 10+ 语言",
        "Qwen3-TTS 1.7B\n• 高音质\n• 语音合成/克隆/设计",
        &["zh","en","fr","de","it","ja","ko","pt","ru","es"],
        "~2.4 GB", "6GB+", "可运行但慢", "qwen3_tts", "Qwen3-TTS-1.7B-GGUF",
        "Qwen/Qwen3-TTS-1.7B-GGUF", "qwen3-tts-1.7b.Q5_K_M.gguf",
        "gguf", "", &[],
    ));

    m.insert("Chatterbox/Chatterbox-GGUF".into(), m_tts(
        "Chatterbox/Chatterbox-GGUF", "Chatterbox 0.5B (GGUF)",
        "Chatterbox TTS + 语音克隆, 17 语言",
        "Chatterbox 0.5B\n• 17 语言\n• 语音克隆 + 语音转换\n• RTF 0.15 (6.7x 实时)",
        &["ar","da","de","el","en","es","fi","fr","hi","it","ko","ms","nl","no","pl","pt","sv","sw","tr"],
        "~1.5 GB", "4GB+", "可运行", "chatterbox", "Chatterbox-GGUF",
        "audio-cpp/audio.cpp-gguf", "chatterbox.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("FishAudio/Fish-S2-Pro-GGUF".into(), m_tts(
        "FishAudio/Fish-S2-Pro-GGUF", "Fish Audio S2 Pro (GGUF)",
        "Fish Audio S2 Pro, 中/英/日/韩/西",
        "Fish Audio S2 Pro GGUF Q8_0\n• 中英日韩西\n• 语音克隆",
        &["zh","en","ja","ko","es"],
        "~2 GB", "4GB+", "可运行", "fish_audio", "Fish-S2-Pro-GGUF",
        "audio-cpp/audio.cpp-gguf", "fish-audio-s2-pro.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Higgs/Higgs-TTS-4B-GGUF".into(), m_tts(
        "Higgs/Higgs-TTS-4B-GGUF", "Higgs Audio TTS 4B (GGUF)",
        "Higgs Audio v3 TTS 4B, 自动语言",
        "Higgs Audio v3 TTS 4B GGUF Q8_0\n• 自动语言检测\n• 语音克隆",
        &["auto"],
        "~6 GB", "12GB+", "不可", "higgs_audio_tts", "Higgs-TTS-4B-GGUF",
        "audio-cpp/audio.cpp-gguf", "higgs-audio-v3-tts-4b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("MioTTS/MioTTS-1.7B-GGUF".into(), m_tts(
        "MioTTS/MioTTS-1.7B-GGUF", "MioTTS 1.7B (GGUF)",
        "MioTTS, 英/日 语音克隆",
        "MioTTS 1.7B\n• 英/日\n• 语音克隆\n• RTF 0.17 (6.0x 实时)",
        &["en","ja"],
        "~2.5 GB", "6GB+", "可运行但慢", "miotts", "MioTTS-1.7B-GGUF",
        "audio-cpp/audio.cpp-gguf", "miotts-1.7b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("OmniVoice/OmniVoice-GGUF".into(), m_tts(
        "OmniVoice/OmniVoice-GGUF", "OmniVoice (GGUF)",
        "OmniVoice, 646+ 语言, 语音设计",
        "OmniVoice (Qwen3-0.6B based)\n• 646+ 语言\n• 语音克隆 + 语音设计\n• RTF 0.05 (20x 实时)",
        &["auto"],
        "~1.5 GB", "4GB+", "流畅", "omnivoice", "OmniVoice-GGUF",
        "audio-cpp/audio.cpp-gguf", "omnivoice.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("PocketTTS/PocketTTS-GGUF".into(), m_tts(
        "PocketTTS/PocketTTS-GGUF", "PocketTTS 100M (GGUF)",
        "PocketTTS, 极速 31x 实时, 4 语言",
        "PocketTTS 100M\n• 英/德/意/葡/西\n• RTF 0.03 (31x 实时, one-shot)\n• RTF 0.02 (48x 实时, long-form)",
        &["en","de","it","pt","es"],
        "~0.3 GB", "1GB+", "极速", "pocket_tts", "PocketTTS-GGUF",
        "audio-cpp/audio.cpp-gguf", "pocket-tts.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Vevo2/Vevo2-GGUF".into(), m_tts(
        "Vevo2/Vevo2-GGUF", "Vevo2 (GGUF)",
        "Vevo2 TTS + 歌声, 中/英, 5x faster",
        "Vevo2 (Qwen2.5-0.5B AR)\n• 中/英 TTS + 唱歌\n• 语音/歌声转换 + 编辑\n• RTF 0.12 (8.7x 实时)",
        &["zh","en"],
        "~2 GB", "4GB+", "可运行", "vevo2", "Vevo2-GGUF",
        "audio-cpp/audio.cpp-gguf", "vevo2.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("VibeVoice/VibeVoice-1.5B-GGUF".into(), m_tts(
        "VibeVoice/VibeVoice-1.5B-GGUF", "VibeVoice 1.5B (GGUF)",
        "VibeVoice 1.5B, 多说话人 TTS, 中/英",
        "VibeVoice 1.5B\n• 中/英多说话人对话 TTS\n• RTF 0.25 (4x 实时)\n• 10 步扩散 94 分钟播客 18 分钟完成",
        &["zh","en"],
        "~3 GB", "8GB+", "可运行", "vibevoice", "VibeVoice-1.5B-GGUF",
        "audio-cpp/audio.cpp-gguf", "vibevoice-1.5b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("VibeVoice/VibeVoice-7B-GGUF".into(), m_tts(
        "VibeVoice/VibeVoice-7B-GGUF", "VibeVoice 7B (GGUF)",
        "VibeVoice 7B, 高音质多说话人 TTS",
        "VibeVoice 7B\n• 中/英多说话人对话 TTS\n• 最高音质",
        &["zh","en"],
        "~12 GB", "24GB+", "不可", "vibevoice", "VibeVoice-7B-GGUF",
        "audio-cpp/audio.cpp-gguf", "vibevoice-7b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("VoxCPM2/VoxCPM2-GGUF".into(), m_tts(
        "VoxCPM2/VoxCPM2-GGUF", "VoxCPM2 2B (GGUF)",
        "VoxCPM2, 30 语言, 48kHz",
        "VoxCPM2 2B 48kHz\n• 30 语言\n• 语音克隆 + 设计\n• RTF 0.23 (4.3x 实时)",
        &["ar","da","de","el","en","es","fi","fr","he","hi","id","it","ja","km","ko","lo","ms","my","nl","no","pl","pt","ru","sv","sw","th","tl","tr","vi","zh"],
        "~4 GB", "8GB+", "可运行但慢", "voxcpm2", "VoxCPM2-GGUF",
        "audio-cpp/audio.cpp-gguf", "voxcpm2.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("IndexTTS2/IndexTTS2-GGUF".into(), m_tts(
        "IndexTTS2/IndexTTS2-GGUF", "IndexTTS-2 (GGUF)",
        "IndexTTS-2, 中/英 情感语音合成",
        "IndexTTS-2\n• 中/英\n• 语音克隆 + 情感表达\n• RTF 0.33 (3x 实时)",
        &["zh","en"],
        "~3 GB", "6GB+", "可运行但慢", "index_tts2", "IndexTTS2-GGUF",
        "audio-cpp/audio.cpp-gguf", "index-tts2.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Irodori/Irodori-TTS-500M-GGUF".into(), m_tts(
        "Irodori/Irodori-TTS-500M-GGUF", "Irodori-TTS 500M (GGUF)",
        "Irodori-TTS, 日语 TTS",
        "Irodori-TTS 500M v3\n• 日语专精",
        &["ja"],
        "~1 GB", "2GB+", "流畅", "irodori_tts", "Irodori-TTS-500M-GGUF",
        "audio-cpp/audio.cpp-gguf", "irodori-tts-500m.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Irodori/Irodori-TTS-600M-VD-GGUF".into(), m_tts(
        "Irodori/Irodori-TTS-600M-VD-GGUF", "Irodori-TTS 600M VD (GGUF)",
        "Irodori-TTS 日语语音设计",
        "Irodori-TTS 600M v3 VoiceDesign\n• 日语\n• 语音设计功能",
        &["ja"],
        "~1.2 GB", "2GB+", "流畅", "irodori_tts", "Irodori-TTS-600M-VD-GGUF",
        "audio-cpp/audio.cpp-gguf", "irodori-tts-600m-vd.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("MOSS/MOSS-TTS-Nano-GGUF".into(), m_tts(
        "MOSS/MOSS-TTS-Nano-GGUF", "MOSS-TTS Nano 100M (GGUF)",
        "MOSS-TTS Nano, 极轻量, 自动语言",
        "MOSS-TTS Nano 100M\n• 自动语言检测\n• 极轻量\n• RTF 0.11 (9x 实时)",
        &["auto"],
        "~0.2 GB", "1GB+", "极速", "moss_tts_nano", "MOSS-TTS-Nano-GGUF",
        "audio-cpp/audio.cpp-gguf", "moss-tts-nano.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("MOSS/MOSS-TTS-Local-GGUF".into(), m_tts(
        "MOSS/MOSS-TTS-Local-GGUF", "MOSS-TTS Local (GGUF)",
        "MOSS-TTS Local Transformer v1.5",
        "MOSS-TTS Local Transformer v1.5\n• 自动语言 + 语言提示\n• RTF 0.20 (5x 实时)",
        &["auto"],
        "~1 GB", "2GB+", "流畅", "moss_tts_local", "MOSS-TTS-Local-GGUF",
        "audio-cpp/audio.cpp-gguf", "moss-tts-local.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("Supertonic/Supertonic-3-GGUF".into(), m_tts(
        "Supertonic/Supertonic-3-GGUF", "Supertonic 3 (GGUF)",
        "Supertonic 3, 30+ 语言, 200x 实时!",
        "Supertonic 3\n• 30+ 语言\n• CUDA RTF 0.005 (188x 实时!)\n• CPU RTF 0.162 (6x 实时)\n• 10 小时音频 3 分钟生成",
        &["en","ko","ja","ar","bg","cs","da","de","el","es","et","fi","fr","hi","hr","hu","id","it","lt","lv","nl","pl","pt","ro","ru","sk","sl","sv","tr","uk","vi"],
        "~2 GB", "4GB+", "流畅 (6x实时)", "supertonic", "Supertonic-3-GGUF",
        "audio-cpp/audio.cpp-gguf", "supertonic-3.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("OuteTTS/OuteTTS-1B-GGUF".into(), m_tts(
        "OuteTTS/OuteTTS-1B-GGUF", "OuteTTS 1.0 1B (GGUF)",
        "OuteTTS, 24 语言, 语音克隆",
        "Llama-OuteTTS 1.0 1B\n• 24 语言\n• 语音克隆\n• 社区贡献 (Mirek)",
        &["en","ar","zh","nl","fr","de","it","ja","ko","lt","ru","es","pt","be","bn","ka","hu","lv","fa","pl","sw","ta","uk"],
        "~2 GB", "4GB+", "可运行", "outetts", "OuteTTS-1B-GGUF",
        "mirek190/audio.cpp", "outetts-1b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    m.insert("VieNeu/VieNeu-TTS-GGUF".into(), m_tts(
        "VieNeu/VieNeu-TTS-GGUF", "VieNeu-TTS v3 Turbo (GGUF)",
        "VieNeu-TTS, 越/英, 社区贡献",
        "VieNeu-TTS v3 Turbo\n• 越/英\n• 语音克隆\n• 社区贡献 (Phuoc)",
        &["vi","en"],
        "~1 GB", "2GB+", "流畅", "vietneu_tts", "VieNeu-TTS-GGUF",
        "audio-cpp/audio.cpp-gguf", "vietneu-tts-v3.Q8_0.gguf",
        "gguf", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  VAD (voice activity detection)
    // ═══════════════════════════════════════════════════════════

    m.insert("MarbleNet/MarbleNet-VAD".into(), m_other(
        "vad", "MarbleNet/MarbleNet-VAD", "MarbleNet VAD",
        "MarbleNet VAD", "MarbleNet VAD\n• 语言无关\n• 语音活动检测",
        &["lang_agnostic"],
        "~0.1 GB", "1GB+", "流畅", "marblenet_vad", "MarbleNet-VAD",
        "", "", "safetensors", "", &[],
    ));

    m.insert("Silero/Silero-VAD".into(), m_other(
        "vad", "Silero/Silero-VAD", "Silero VAD",
        "Silero VAD", "Silero VAD\n• 语言无关\n• 经典 VAD 模型",
        &["lang_agnostic"],
        "~0.05 GB", "1GB+", "流畅", "silero_vad", "Silero-VAD",
        "", "", "safetensors", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  Diarization / Alignment
    // ═══════════════════════════════════════════════════════════

    m.insert("Sortformer/Sortformer-Diar".into(), m_other(
        "diar", "Sortformer/Sortformer-Diar", "Sortformer 4spk Diar",
        "Sortformer 4 说话人分离, 英语",
        "Sortformer 4spk v1\n• 英语说话人分离",
        &["en"],
        "~1 GB", "2GB+", "可运行", "sortformer_diar", "Sortformer-Diar",
        "", "", "safetensors", "", &[],
    ));

    m.insert("Qwen3/Qwen3-Aligner-0.6B".into(), m_other(
        "align", "Qwen3/Qwen3-Aligner-0.6B", "Qwen3 Forced Aligner 0.6B",
        "Qwen3 强制对齐, 11 语言",
        "Qwen3 Forced Aligner 0.6B\n• 11 语言强制对齐\n• 精确时间戳",
        &["zh","yue","en","de","es","fr","it","pt","ru","ko","ja"],
        "~1 GB", "2GB+", "流畅", "qwen3_forced_aligner", "Qwen3-Aligner-0.6B",
        "audio-cpp/audio.cpp-gguf", "qwen3-aligner-0.6b.Q8_0.gguf",
        "gguf", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  Voice Conversion / Codec
    // ═══════════════════════════════════════════════════════════

    m.insert("SeedVC/SeedVC-MLX".into(), m_other(
        "vc", "SeedVC/SeedVC-MLX", "SeedVC Voice Conversion",
        "SeedVC 语音转换, 语言无关",
        "SeedVC XLS-R + HiFT / Whisper-small + BigVGAN\n• 语言无关语音转换",
        &["lang_agnostic"],
        "~2 GB", "4GB+", "可运行", "seed_vc", "SeedVC-MLX",
        "", "", "safetensors", "", &[],
    ));

    m.insert("MioCodec/MioCodec-v2".into(), m_other(
        "codec", "MioCodec/MioCodec-v2", "MioCodec v2 25Hz",
        "MioCodec 音频编解码, 25Hz 44.1kHz",
        "MioCodec v2\n• 25 Hz, 44.1 kHz\n• 语音转换后端",
        &["lang_agnostic"],
        "~0.5 GB", "1GB+", "流畅", "miocodec", "MioCodec-v2",
        "", "", "safetensors", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  Music Generation
    // ═══════════════════════════════════════════════════════════

    m.insert("ACE-Step/ACE-Step-1.5".into(), m_other(
        "music", "ACE-Step/ACE-Step-1.5", "ACE-Step 1.5",
        "ACE-Step 音乐生成/编辑, 50+ 语言",
        "ACE-Step 1.5 Turbo/Base\n• 50+ 语言音乐生成\n• 音乐编辑",
        &["auto"],
        "~4 GB", "8GB+", "可运行但慢", "ace_step", "ACE-Step-1.5",
        "", "", "safetensors", "", &[],
    ));

    m.insert("HeartMuLa/HeartMuLa-3B".into(), m_other(
        "music", "HeartMuLa/HeartMuLa-3B", "HeartMuLa 3B",
        "HeartMuLa 音乐生成, 中/英/日/韩/西",
        "HeartMuLa-oss-3B\n• 中/英/日/韩/西\n• HeartCodec-oss",
        &["zh","en","ja","ko","es"],
        "~6 GB", "12GB+", "不可", "heartmula", "HeartMuLa-3B",
        "", "", "safetensors", "", &[],
    ));

    m.insert("StableAudio/SA3-Small-Music".into(), m_other(
        "music", "StableAudio/SA3-Small-Music", "Stable Audio 3 Small Music",
        "Stable Audio 3 Small, 音乐生成",
        "Stable Audio 3 Small Music\n• 音乐生成",
        &["en"],
        "~2 GB", "4GB+", "可运行", "stable_audio", "SA3-Small-Music",
        "", "", "safetensors", "", &[],
    ));

    m.insert("StableAudio/SA3-Small-SFX".into(), m_other(
        "music", "StableAudio/SA3-Small-SFX", "Stable Audio 3 Small SFX",
        "Stable Audio 3 Small, 音效生成",
        "Stable Audio 3 Small SFX\n• 音效生成",
        &["en"],
        "~2 GB", "4GB+", "可运行", "stable_audio", "SA3-Small-SFX",
        "", "", "safetensors", "", &[],
    ));

    m.insert("StableAudio/SA3-Medium".into(), m_other(
        "music", "StableAudio/SA3-Medium", "Stable Audio 3 Medium",
        "Stable Audio 3 Medium, 高质量音乐生成",
        "Stable Audio 3 Medium\n• 高质量音乐生成",
        &["en"],
        "~6 GB", "12GB+", "不可", "stable_audio", "SA3-Medium",
        "", "", "safetensors", "", &[],
    ));

    // ═══════════════════════════════════════════════════════════
    //  Source Separation
    // ═══════════════════════════════════════════════════════════

    m.insert("HTDemucs/HTDemucs".into(), m_other(
        "sep", "HTDemucs/HTDemucs", "HTDemucs",
        "HTDemucs 音源分离, 语言无关",
        "HTDemucs / HTDemucs_ft\n• 音源分离 (人声/伴奏/鼓/贝斯)",
        &["lang_agnostic"],
        "~0.5 GB", "2GB+", "流畅", "htdemucs", "HTDemucs",
        "", "", "safetensors", "", &[],
    ));

    m.insert("MelBandRoformer/MBR-MLX".into(), m_other(
        "sep", "MelBandRoformer/MBR-MLX", "Mel-Band RoFormer",
        "Mel-Band RoFormer 人声分离",
        "Mel-Band RoFormer MLX\n• 人声分离\n• 语言无关",
        &["lang_agnostic"],
        "~1 GB", "2GB+", "流畅", "mel_band_roformer", "MBR-MLX",
        "", "", "safetensors", "", &[],
    ));

    m
}

/// Resolve the LLM directory (shared with Python SWT).
pub fn llm_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SWT2_LLM_DIR") {
        let p = PathBuf::from(&dir);
        if p.exists() {
            return p;
        }
    }
    let exe_path = std::env::current_exe().unwrap_or_default();
    let mut current = exe_path.parent().map(|p| p.to_path_buf());
    while let Some(ref dir) = current {
        let llm = dir.join("llm");
        if llm.exists() { return llm; }
        let swt_llm = dir.join("swt").join("llm");
        if swt_llm.exists() { return swt_llm; }
        if dir.parent().is_none() { break; }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    let mut current = exe_path.parent().map(|p| p.to_path_buf());
    while let Some(ref dir) = current {
        if dir.join("src-tauri").exists() || dir.join("Cargo.toml").exists() {
            return dir.join("llm");
        }
        if dir.parent().is_none() { break; }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    exe_path.parent().map(|p| p.join("llm")).unwrap_or_else(|| PathBuf::from("llm"))
}

pub fn scan_local_models() -> Vec<String> {
    let dir = llm_dir();
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                let has_gguf = std::fs::read_dir(entry.path())
                    .map(|mut e| e.any(|f| f.ok().map_or(false, |ff| ff.file_name().to_string_lossy().ends_with(".gguf"))))
                    .unwrap_or(false);
                let has_config = entry.path().join("config.json").exists();
                if has_gguf || has_config {
                    found.push(name);
                }
            }
        }
    }
    found
}

pub fn get_models_with_status(category_filter: Option<&str>) -> Vec<ModelInfo> {
    let local = scan_local_models();
    let local_set: std::collections::HashSet<_> = local.iter().collect();
    build_catalog()
        .into_iter()
        .filter_map(|(_, mut info)| {
            if let Some(cat) = category_filter {
                if info.category != cat {
                    return None;
                }
            }
            info.downloaded = local_set.contains(&info.local_dir_name);
            Some(info)
        })
        .collect()
}
