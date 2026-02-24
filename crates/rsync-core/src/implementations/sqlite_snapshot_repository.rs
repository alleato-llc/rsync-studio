use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use uuid::Uuid;

use crate::error::AppError;
use crate::implementations::database::{parse_datetime, parse_uuid};
use crate::models::backup::SnapshotRecord;
use crate::traits::snapshot_repository::SnapshotRepository;

pub struct SqliteSnapshotRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteSnapshotRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl SnapshotRepository for SqliteSnapshotRepository {
    fn create_snapshot(&self, snapshot: &SnapshotRecord) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT INTO snapshots (id, job_id, invocation_id, snapshot_path, link_dest_path, created_at, size_bytes, file_count, is_latest)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                snapshot.id.to_string(),
                snapshot.job_id.to_string(),
                snapshot.invocation_id.to_string(),
                snapshot.snapshot_path,
                snapshot.link_dest_path,
                snapshot.created_at.to_rfc3339(),
                snapshot.size_bytes as i64,
                snapshot.file_count as i64,
                snapshot.is_latest as i32,
            ],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn get_latest_snapshot_for_job(
        &self,
        job_id: &Uuid,
    ) -> Result<Option<SnapshotRecord>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, invocation_id, snapshot_path, link_dest_path, created_at, size_bytes, file_count, is_latest
                 FROM snapshots WHERE job_id = ?1 ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let result = stmt
            .query_row(rusqlite::params![job_id.to_string()], |row| {
                Ok(row_to_snapshot(row))
            });

        match result {
            Ok(snapshot) => Ok(Some(snapshot?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::DatabaseError(e.to_string())),
        }
    }

    fn list_snapshots_for_job(&self, job_id: &Uuid) -> Result<Vec<SnapshotRecord>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, invocation_id, snapshot_path, link_dest_path, created_at, size_bytes, file_count, is_latest
                 FROM snapshots WHERE job_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![job_id.to_string()], |row| {
                Ok(row_to_snapshot(row))
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut snapshots = Vec::new();
        for row in rows {
            let snapshot = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            snapshots.push(snapshot);
        }
        Ok(snapshots)
    }

    fn delete_snapshot(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = conn
            .execute(
                "DELETE FROM snapshots WHERE id = ?1",
                rusqlite::params![id.to_string()],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!("Snapshot {} not found", id)));
        }
        Ok(())
    }
}

fn row_to_snapshot(row: &rusqlite::Row) -> Result<SnapshotRecord, AppError> {
    let id_str: String = row.get(0).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let job_id_str: String = row.get(1).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let inv_id_str: String = row.get(2).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let snapshot_path: String = row.get(3).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let link_dest_path: Option<String> = row.get(4).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let created_str: String = row.get(5).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let size_bytes: i64 = row.get(6).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let file_count: i64 = row.get(7).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let is_latest: i32 = row.get(8).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(SnapshotRecord {
        id: parse_uuid(&id_str)?,
        job_id: parse_uuid(&job_id_str)?,
        invocation_id: parse_uuid(&inv_id_str)?,
        snapshot_path,
        link_dest_path,
        created_at: parse_datetime(&created_str)?,
        size_bytes: size_bytes as u64,
        file_count: file_count as u64,
        is_latest: is_latest != 0,
    })
}
