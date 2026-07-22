/// Sidecar manager — resolves and health-checks the whisper-cli binary.
/// No persistent server process; each transcription spawns a one-shot CLI call.
use std::path::PathBuf;
use std::process::Command;

/// Locate the whisper-cli sidecar binary by walking up from the exe.
pub fn find_whisper_cli() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("WHISPER_CLI_PATH") {
        let path = PathBuf::from(&p);
        if path.exists() {
            return Some(path);
        }
    }
    let exe = std::env::current_exe().ok()?;
    let mut current = exe.parent()?.to_path_buf();
    loop {
        for name in &[
            "audiocpp_server-x86_64-pc-windows-gnu.exe",
            "audiocpp_server.exe",
            "whisper-cli.exe",
        ] {
            let candidate = current.join("binaries").join(name);
            if candidate.exists() && candidate.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
                return Some(candidate);
            }
        }
        // Also check for sidecar in target/debug (dev mode)
        let debug_candidate = current.join("target").join("debug").join("audiocpp_server.exe");
        if debug_candidate.exists() && debug_candidate.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
            return Some(debug_candidate);
        }
        if current.parent().is_none() {
            break;
        }
        current = current.parent()?.to_path_buf();
    }
    None
}

#[tauri::command]
pub async fn health_check() -> Result<serde_json::Value, String> {
    match find_whisper_cli() {
        Some(path) => {
            let version = Command::new(&path)
                .arg("--version")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stderr).trim().to_string())
                .unwrap_or_default();
            let backend = if version.is_empty() { "whisper.cpp" } else { "whisper.cpp" };
            Ok(serde_json::json!({
                "status": "ok",
                "backend": backend,
                "models": [],
            }))
        }
        None => Err("whisper-cli binary not found".into()),
    }
}
