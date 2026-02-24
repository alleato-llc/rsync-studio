use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use uuid::Uuid;

use crate::database::sqlite::{from_json, parse_datetime, parse_uuid, to_json};
use crate::error::AppError;
use crate::models::job::JobDefinition;
use crate::repository::job::JobRepository;

pub struct SqliteJobRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteJobRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl JobRepository for SqliteJobRepository {
    fn create_job(&self, job: &JobDefinition) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT INTO jobs (id, name, description, source, destination, backup_mode, options, ssh_config, schedule, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                job.id.to_string(),
                job.name,
                job.description,
                to_json(&job.source)?,
                to_json(&job.destination)?,
                to_json(&job.backup_mode)?,
                to_json(&job.options)?,
                job.ssh_config.as_ref().map(|s| to_json(s)).transpose()?,
                job.schedule.as_ref().map(|s| to_json(s)).transpose()?,
                job.enabled as i32,
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn get_job(&self, id: &Uuid) -> Result<JobDefinition, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, source, destination, backup_mode, options, ssh_config, schedule, enabled, created_at, updated_at
                 FROM jobs WHERE id = ?1",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        stmt.query_row(rusqlite::params![id.to_string()], |row| {
            Ok(row_to_job(row))
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Job {} not found", id))
            }
            _ => AppError::DatabaseError(e.to_string()),
        })?
    }

    fn list_jobs(&self) -> Result<Vec<JobDefinition>, AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, source, destination, backup_mode, options, ssh_config, schedule, enabled, created_at, updated_at
                 FROM jobs ORDER BY name",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| Ok(row_to_job(row)))
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut jobs = Vec::new();
        for row in rows {
            let job = row.map_err(|e| AppError::DatabaseError(e.to_string()))??;
            jobs.push(job);
        }
        Ok(jobs)
    }

    fn update_job(&self, job: &JobDefinition) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = conn
            .execute(
                "UPDATE jobs SET name = ?1, description = ?2, source = ?3, destination = ?4, backup_mode = ?5, options = ?6, ssh_config = ?7, schedule = ?8, enabled = ?9, updated_at = ?10
                 WHERE id = ?11",
                rusqlite::params![
                    job.name,
                    job.description,
                    to_json(&job.source)?,
                    to_json(&job.destination)?,
                    to_json(&job.backup_mode)?,
                    to_json(&job.options)?,
                    job.ssh_config.as_ref().map(|s| to_json(s)).transpose()?,
                    job.schedule.as_ref().map(|s| to_json(s)).transpose()?,
                    job.enabled as i32,
                    job.updated_at.to_rfc3339(),
                    job.id.to_string(),
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!("Job {} not found", job.id)));
        }
        Ok(())
    }

    fn delete_job(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let rows = conn
            .execute("DELETE FROM jobs WHERE id = ?1", rusqlite::params![id.to_string()])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if rows == 0 {
            return Err(AppError::NotFound(format!("Job {} not found", id)));
        }
        Ok(())
    }
}

fn row_to_job(row: &rusqlite::Row) -> Result<JobDefinition, AppError> {
    let id_str: String = row.get(0).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let name: String = row.get(1).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let description: Option<String> = row.get(2).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let source_json: String = row.get(3).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let dest_json: String = row.get(4).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let mode_json: String = row.get(5).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let options_json: String = row.get(6).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let ssh_json: Option<String> = row.get(7).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let schedule_json: Option<String> = row.get(8).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let enabled: i32 = row.get(9).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let created_str: String = row.get(10).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let updated_str: String = row.get(11).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(JobDefinition {
        id: parse_uuid(&id_str)?,
        name,
        description,
        source: from_json(&source_json)?,
        destination: from_json(&dest_json)?,
        backup_mode: from_json(&mode_json)?,
        options: from_json(&options_json)?,
        ssh_config: ssh_json.as_deref().map(from_json).transpose()?,
        schedule: schedule_json.as_deref().map(from_json).transpose()?,
        enabled: enabled != 0,
        created_at: parse_datetime(&created_str)?,
        updated_at: parse_datetime(&updated_str)?,
    })
}
