use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::AppError;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, AppError> {
        let conn = Connection::open(path)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_pragmas()?;
        db.run_migrations()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_pragmas()?;
        db.run_migrations()?;
        Ok(db)
    }

    pub fn conn(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }

    fn init_pragmas(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;"
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY NOT NULL,
                applied_at TEXT NOT NULL
            );"
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let current_version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if current_version < 1 {
            let sql = include_str!("../migrations/v001_initial.sql");
            conn.execute_batch(sql)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (1, datetime('now'))",
                [],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        if current_version < 2 {
            let sql = include_str!("../migrations/v002_run_statistics.sql");
            conn.execute_batch(sql)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (2, datetime('now'))",
                [],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }
}

// Helper functions reused by all repository implementations
pub fn to_json<T: serde::Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|e| AppError::SerializationError(e.to_string()))
}

pub fn from_json<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, AppError> {
    serde_json::from_str(s).map_err(|e| AppError::SerializationError(e.to_string()))
}

pub fn parse_uuid(s: &str) -> Result<uuid::Uuid, AppError> {
    uuid::Uuid::parse_str(s).map_err(|e| AppError::DatabaseError(format!("Invalid UUID: {}", e)))
}

pub fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    s.parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| AppError::DatabaseError(format!("Invalid datetime: {}", e)))
}
