use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use uuid::Uuid;

use crate::database::sqlite::{from_json, parse_datetime, parse_uuid, to_json};
use crate::error::AppError;
use crate::models::backup::BackupInvocation;
use crate::repository::invocation::InvocationRepository;

pub struct SqliteInvocationRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteInvocationRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl InvocationRepository for SqliteInvocationRepository {
    fn create_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT INTO invocations (id, job_id, started_at, finished_at, status, bytes_transferred, files_transferred, total_files, snapshot_path, command_executed, exit_code, trigger, log_file_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                inv.id.to_string(),
                inv.job_id.to_string(),
                inv.started_at.to_rfc3339(),
                inv.finished_at.map(|dt| dt.to_rfc3339()),
                to_json(&inv.status)?,
                inv.bytes_transferred as i64,
                inv.files_transferred as i64,
                inv.total_files as i64,
                inv.snapshot_path,
                inv.command_executed,
                inv.exit_code,
                to_json(&inv.trigger)?,
                inv.log_file_path,
            ],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn get_invocation(&self, id: &Uuid) -> Result<BackupInvocation, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, finished_at, status, bytes_transferred, files_transferred, total_files, snapshot_path, command_executed, exit_code, trigger, log_file_path
                 FROM invocations WHERE id = ?1",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        stmt.query_row(rusqlite::params![id.to_string()], |row| {
            Ok(row_to_invocation(row))
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Invocation {} not found", id))
            }
            _ => AppError::DatabaseError(e.to_string()),
        })?
    }

    fn list_invocations_for_job(&self, job_id: &Uuid) -> Result<Vec<BackupInvocation>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, finished_at, status, bytes_transferred, files_transferred, total_files, snapshot_path, command_executed, exit_code, trigger, log_file_path
                 FROM invocations WHERE job_id = ?1 ORDER BY started_at DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![job_id.to_string()], |row| {
                Ok(row_to_invocation(row))
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut invocations = Vec::new();
        for row in rows {
            let inv = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            invocations.push(inv);
        }
        Ok(invocations)
    }

    fn list_all_invocations(&self) -> Result<Vec<BackupInvocation>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, started_at, finished_at, status, bytes_transferred, files_transferred, total_files, snapshot_path, command_executed, exit_code, trigger, log_file_path
                 FROM invocations ORDER BY started_at DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_invocation(row)))
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut invocations = Vec::new();
        for row in rows {
            let inv = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            invocations.push(inv);
        }
        Ok(invocations)
    }

    fn update_invocation(&self, inv: &BackupInvocation) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = conn
            .execute(
                "UPDATE invocations SET finished_at = ?1, status = ?2, bytes_transferred = ?3, files_transferred = ?4, total_files = ?5, snapshot_path = ?6, exit_code = ?7, log_file_path = ?8
                 WHERE id = ?9",
                rusqlite::params![
                    inv.finished_at.map(|dt| dt.to_rfc3339()),
                    to_json(&inv.status)?,
                    inv.bytes_transferred as i64,
                    inv.files_transferred as i64,
                    inv.total_files as i64,
                    inv.snapshot_path,
                    inv.exit_code,
                    inv.log_file_path,
                    inv.id.to_string(),
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Invocation {} not found",
                inv.id
            )));
        }
        Ok(())
    }

    fn delete_invocation(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = conn
            .execute(
                "DELETE FROM invocations WHERE id = ?1",
                rusqlite::params![id.to_string()],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Invocation {} not found",
                id
            )));
        }
        Ok(())
    }

    fn delete_invocations_for_job(&self, job_id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "DELETE FROM invocations WHERE job_id = ?1",
            rusqlite::params![job_id.to_string()],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

fn row_to_invocation(row: &rusqlite::Row) -> Result<BackupInvocation, AppError> {
    let id_str: String = row.get(0).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let job_id_str: String = row.get(1).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let started_str: String = row.get(2).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let finished_str: Option<String> = row.get(3).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let status_json: String = row.get(4).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let bytes: i64 = row.get(5).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let files: i64 = row.get(6).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let total: i64 = row.get(7).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let snapshot_path: Option<String> = row.get(8).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let command: String = row.get(9).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let exit_code: Option<i32> = row.get(10).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let trigger_json: String = row.get(11).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let log_file_path: Option<String> = row.get(12).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(BackupInvocation {
        id: parse_uuid(&id_str)?,
        job_id: parse_uuid(&job_id_str)?,
        started_at: parse_datetime(&started_str)?,
        finished_at: finished_str.as_deref().map(parse_datetime).transpose()?,
        status: from_json(&status_json)?,
        bytes_transferred: bytes as u64,
        files_transferred: files as u64,
        total_files: total as u64,
        snapshot_path,
        command_executed: command,
        exit_code,
        trigger: from_json(&trigger_json)?,
        log_file_path,
    })
}
