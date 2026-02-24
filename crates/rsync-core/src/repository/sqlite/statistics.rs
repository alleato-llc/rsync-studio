use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use uuid::Uuid;

use crate::database::sqlite::{parse_datetime, parse_uuid};
use crate::error::AppError;
use crate::models::statistics::RunStatistic;
use crate::repository::statistics::StatisticsRepository;

pub struct SqliteStatisticsRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStatisticsRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl StatisticsRepository for SqliteStatisticsRepository {
    fn record_statistic(&self, stat: &RunStatistic) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT INTO run_statistics (id, job_id, invocation_id, recorded_at, files_transferred, bytes_transferred, duration_secs, speedup)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                stat.id.to_string(),
                stat.job_id.to_string(),
                stat.invocation_id.to_string(),
                stat.recorded_at.to_rfc3339(),
                stat.files_transferred as i64,
                stat.bytes_transferred as i64,
                stat.duration_secs,
                stat.speedup,
            ],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn get_statistics_for_job(&self, job_id: &Uuid) -> Result<Vec<RunStatistic>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, invocation_id, recorded_at, files_transferred, bytes_transferred, duration_secs, speedup
                 FROM run_statistics WHERE job_id = ?1 ORDER BY recorded_at DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params![job_id.to_string()], |row| {
                Ok(row_to_statistic(row))
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut stats = Vec::new();
        for row in rows {
            let stat = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            stats.push(stat);
        }
        Ok(stats)
    }

    fn get_all_statistics(&self) -> Result<Vec<RunStatistic>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, job_id, invocation_id, recorded_at, files_transferred, bytes_transferred, duration_secs, speedup
                 FROM run_statistics ORDER BY recorded_at DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_statistic(row)))
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut stats = Vec::new();
        for row in rows {
            let stat = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            stats.push(stat);
        }
        Ok(stats)
    }

    fn delete_statistics_for_job(&self, job_id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "DELETE FROM run_statistics WHERE job_id = ?1",
            rusqlite::params![job_id.to_string()],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn delete_all_statistics(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute("DELETE FROM run_statistics", [])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

fn row_to_statistic(row: &rusqlite::Row) -> Result<RunStatistic, AppError> {
    let id_str: String = row.get(0).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let job_id_str: String = row.get(1).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let inv_id_str: String = row.get(2).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let recorded_str: String = row.get(3).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let files: i64 = row.get(4).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let bytes: i64 = row.get(5).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let duration: f64 = row.get(6).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let speedup: Option<f64> = row.get(7).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(RunStatistic {
        id: parse_uuid(&id_str)?,
        job_id: parse_uuid(&job_id_str)?,
        invocation_id: parse_uuid(&inv_id_str)?,
        recorded_at: parse_datetime(&recorded_str)?,
        files_transferred: files as u64,
        bytes_transferred: bytes as u64,
        duration_secs: duration,
        speedup,
    })
}
