use crate::db::{Database, HistoryRecord};
use tauri::State;

#[tauri::command]
pub fn get_history(
    db: State<'_, Database>,
    limit: i64,
    offset: i64,
) -> Result<Vec<HistoryRecord>, String> {
    db.list(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_history(
    db: State<'_, Database>,
    query: String,
    limit: i64,
) -> Result<Vec<HistoryRecord>, String> {
    db.search(&query, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_history_count(db: State<'_, Database>) -> Result<i64, String> {
    db.count().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_history(db: State<'_, Database>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_history_by_id(
    db: State<'_, Database>,
    id: i64,
) -> Result<Option<HistoryRecord>, String> {
    db.get_by_id(id).map_err(|e| e.to_string())
}
