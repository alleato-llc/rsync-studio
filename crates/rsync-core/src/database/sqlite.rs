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

        if current_version < 3 {
            let sql = include_str!("../migrations/v003_settings.sql");
            conn.execute_batch(sql)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (3, datetime('now'))",
                [],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        if current_version < 4 {
            Self::migrate_v004_nest_rsync_options(&conn)?;
            conn.execute(
                "INSERT INTO schema_version (version, applied_at) VALUES (4, datetime('now'))",
                [],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// V004: Restructure flat RsyncOptions JSON into nested sub-structs.
    ///
    /// Done in Rust (not SQL) because SQLite's json_extract returns integers
    /// for JSON booleans, which serde cannot deserialize as `bool`.
    fn migrate_v004_nest_rsync_options(conn: &Connection) -> Result<(), AppError> {
        let mut stmt = conn
            .prepare("SELECT id, options FROM jobs")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for (id, options_json) in &rows {
            let val: serde_json::Value = serde_json::from_str(options_json)
                .map_err(|e| AppError::DatabaseError(format!("Invalid options JSON for job {}: {}", id, e)))?;

            // Skip rows already migrated (idempotent)
            if val.get("core_transfer").is_some() {
                continue;
            }

            let obj = val.as_object().ok_or_else(|| {
                AppError::DatabaseError(format!("Options is not a JSON object for job {}", id))
            })?;

            let b = |key: &str| -> bool {
                obj.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
            };
            let b_true = |key: &str| -> bool {
                obj.get(key).and_then(|v| v.as_bool()).unwrap_or(true)
            };

            let nested = serde_json::json!({
                "core_transfer": {
                    "archive": b_true("archive"),
                    "compress": b("compress"),
                    "partial": b("partial"),
                    "dry_run": b("dry_run"),
                },
                "file_handling": {
                    "delete": b("delete"),
                    "size_only": b("size_only"),
                    "checksum": b("checksum"),
                    "update": b("update"),
                    "whole_file": b("whole_file"),
                    "ignore_existing": b("ignore_existing"),
                    "one_file_system": b("one_file_system"),
                },
                "metadata": {
                    "hard_links": b("hard_links"),
                    "acls": b("acls"),
                    "xattrs": b("xattrs"),
                    "numeric_ids": b("numeric_ids"),
                },
                "output": {
                    "verbose": b("verbose"),
                    "progress": b("progress"),
                    "human_readable": b("human_readable"),
                    "stats": b("stats"),
                    "itemize_changes": b("itemize_changes"),
                },
                "advanced": {
                    "exclude_patterns": obj.get("exclude_patterns").cloned().unwrap_or(serde_json::json!([])),
                    "include_patterns": obj.get("include_patterns").cloned().unwrap_or(serde_json::json!([])),
                    "bandwidth_limit": obj.get("bandwidth_limit").cloned().unwrap_or(serde_json::Value::Null),
                    "custom_args": obj.get("custom_args").cloned().unwrap_or(serde_json::json!([])),
                },
            });

            let new_json = serde_json::to_string(&nested)
                .map_err(|e| AppError::SerializationError(e.to_string()))?;

            conn.execute(
                "UPDATE jobs SET options = ?1 WHERE id = ?2",
                rusqlite::params![new_json, id],
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
