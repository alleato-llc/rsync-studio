use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::error::AppError;
use crate::traits::settings_repository::SettingsRepository;

pub struct SqliteSettingsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteSettingsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn get_setting(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT value FROM settings WHERE key = ?1")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![key], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    fn set_setting(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn delete_setting(&self, key: &str) -> Result<(), AppError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "DELETE FROM settings WHERE key = ?1",
            rusqlite::params![key],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

// Make `optional()` available on rusqlite::Result
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
