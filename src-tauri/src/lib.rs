mod audio;
mod commands;
mod config;
mod db;
mod downloader;
mod error;
mod model_registry;
mod sidecar;

use commands::models::DownloadState;
use commands::settings::ConfigState;
use db::Database;
use std::path::PathBuf;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! SWT2 backend is ready.", name)
}

fn app_data_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::new(&app_data_dir()).expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ConfigState::new())
        .manage(DownloadState::new())
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            greet,
            // Health (checks whisper-cli availability)
            sidecar::health_check,
            // Settings
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::reset_config,
            // History
            commands::history::get_history,
            commands::history::search_history,
            commands::history::get_history_count,
            commands::history::delete_history,
            commands::history::get_history_by_id,
            // Models
            commands::models::list_models,
            commands::models::start_download,
            commands::models::cancel_download,
            commands::models::get_active_download,
            commands::models::delete_model,
            // Transcription (whisper-cli via CLI, no server)
            commands::transcribe::transcribe_file,
            commands::transcribe::transcribe_batch,
            // Streaming (whisper-stream via CLI)
            commands::stream::stream_transcribe,
            // TTS (placeholder — Windows SAPI via PowerShell)
            commands::tts::synthesize,
            commands::tts::get_voices,
            // Conversation
            commands::conversation::conversation_turn,
            // API Server
            commands::api_server::start_api_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
