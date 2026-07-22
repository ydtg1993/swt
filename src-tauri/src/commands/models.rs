use crate::downloader;
use crate::model_registry::{get_models_with_status, llm_dir, ModelInfo};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

/// Global download cancel flags.
pub struct DownloadState {
    pub active: Mutex<Option<(String, Arc<AtomicBool>)>>, // (model_id, cancel flag)
}

impl DownloadState {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
        }
    }
}

#[tauri::command]
pub fn list_models(category: Option<String>) -> Vec<ModelInfo> {
    get_models_with_status(category.as_deref())
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    state: State<'_, DownloadState>,
    model_id: String,
) -> Result<(), String> {
    // Check not already downloading
    {
        let guard = state.active.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("已有下载任务在进行中".into());
        }
    }

    let info = crate::model_registry::all_models()
        .get(&model_id)
        .cloned()
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut guard = state.active.lock().map_err(|e| e.to_string())?;
        *guard = Some((model_id.clone(), cancel.clone()));
    }

    let result = downloader::download_model(app.clone(), info, cancel.clone()).await;

    // Clear active state
    {
        let mut guard = state.active.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }

    result
}

#[tauri::command]
pub fn cancel_download(state: State<'_, DownloadState>) -> Result<(), String> {
    let guard = state.active.lock().map_err(|e| e.to_string())?;
    if let Some((_, cancel)) = guard.as_ref() {
        cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub fn get_active_download(state: State<'_, DownloadState>) -> Result<Option<String>, String> {
    let guard = state.active.lock().map_err(|e| e.to_string())?;
    Ok(guard.as_ref().map(|(id, _)| id.clone()))
}

#[tauri::command]
pub fn delete_model(model_id: String) -> Result<(), String> {
    let info = crate::model_registry::all_models()
        .get(&model_id)
        .cloned()
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;
    let dir = llm_dir().join(&info.local_dir_name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}
