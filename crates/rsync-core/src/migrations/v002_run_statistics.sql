CREATE TABLE run_statistics (
    id                TEXT PRIMARY KEY,
    job_id            TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    invocation_id     TEXT NOT NULL REFERENCES invocations(id) ON DELETE CASCADE,
    recorded_at       TEXT NOT NULL,
    files_transferred INTEGER NOT NULL DEFAULT 0,
    bytes_transferred INTEGER NOT NULL DEFAULT 0,
    duration_secs     REAL NOT NULL DEFAULT 0,
    speedup           REAL
);
CREATE INDEX idx_run_statistics_job_id ON run_statistics(job_id);
