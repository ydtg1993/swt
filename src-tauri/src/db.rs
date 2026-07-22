/// SQLite database for transcription history.
use rusqlite::{Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: i64,
    pub source: String,
    pub file_name: String,
    pub file_duration: f64,
    pub model_name: String,
    pub language: String,
    pub full_text: String,
    pub word_count: i64,
    pub segments_json: String,
    pub created_at: String,
}

/// Thread-safe database connection.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: &PathBuf) -> SqlResult<Self> {
        std::fs::create_dir_all(app_dir).ok();
        let db_path = app_dir.join("history.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL DEFAULT 'file',
                file_name TEXT NOT NULL DEFAULT '',
                file_duration REAL NOT NULL DEFAULT 0.0,
                model_name TEXT NOT NULL DEFAULT '',
                language TEXT NOT NULL DEFAULT 'auto',
                full_text TEXT NOT NULL DEFAULT '',
                word_count INTEGER NOT NULL DEFAULT 0,
                segments_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[allow(dead_code)]
    pub fn insert(&self, record: &HistoryRecord) -> SqlResult<i64> {
        let conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e))
        })?;
        conn.execute(
            "INSERT INTO history (source, file_name, file_duration, model_name, language, full_text, word_count, segments_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                record.source,
                record.file_name,
                record.file_duration,
                record.model_name,
                record.language,
                record.full_text,
                record.word_count,
                record.segments_json,
                record.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list(&self, limit: i64, offset: i64) -> SqlResult<Vec<HistoryRecord>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, file_name, file_duration, model_name, language, full_text, word_count, segments_json, created_at
             FROM history ORDER BY id DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit, offset], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                file_name: row.get(2)?,
                file_duration: row.get(3)?,
                model_name: row.get(4)?,
                language: row.get(5)?,
                full_text: row.get(6)?,
                word_count: row.get(7)?,
                segments_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    pub fn search(&self, query: &str, limit: i64) -> SqlResult<Vec<HistoryRecord>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e)))?;
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, source, file_name, file_duration, model_name, language, full_text, word_count, segments_json, created_at
             FROM history WHERE file_name LIKE ?1 OR full_text LIKE ?1
             ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern, limit], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                file_name: row.get(2)?,
                file_duration: row.get(3)?,
                model_name: row.get(4)?,
                language: row.get(5)?,
                full_text: row.get(6)?,
                word_count: row.get(7)?,
                segments_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    pub fn count(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e)))?;
        conn.query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))
    }

    pub fn delete(&self, id: i64) -> SqlResult<usize> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e)))?;
        conn.execute("DELETE FROM history WHERE id = ?1", [id])
    }

    pub fn get_by_id(&self, id: i64) -> SqlResult<Option<HistoryRecord>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(format!("DB lock poisoned: {}", e)))?;
        let mut stmt = conn.prepare(
            "SELECT id, source, file_name, file_duration, model_name, language, full_text, word_count, segments_json, created_at
             FROM history WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(HistoryRecord {
                id: row.get(0)?,
                source: row.get(1)?,
                file_name: row.get(2)?,
                file_duration: row.get(3)?,
                model_name: row.get(4)?,
                language: row.get(5)?,
                full_text: row.get(6)?,
                word_count: row.get(7)?,
                segments_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }
}
