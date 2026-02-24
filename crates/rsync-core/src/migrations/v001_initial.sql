CREATE TABLE jobs (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    description TEXT,
    source      TEXT NOT NULL,
    destination TEXT NOT NULL,
    backup_mode TEXT NOT NULL,
    options     TEXT NOT NULL,
    ssh_config  TEXT,
    schedule    TEXT,
    enabled     INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE invocations (
    id                TEXT PRIMARY KEY NOT NULL,
    job_id            TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    started_at        TEXT NOT NULL,
    finished_at       TEXT,
    status            TEXT NOT NULL,
    bytes_transferred INTEGER NOT NULL DEFAULT 0,
    files_transferred INTEGER NOT NULL DEFAULT 0,
    total_files       INTEGER NOT NULL DEFAULT 0,
    snapshot_path     TEXT,
    command_executed  TEXT NOT NULL,
    exit_code         INTEGER,
    trigger           TEXT NOT NULL,
    log_file_path     TEXT
);
CREATE INDEX idx_invocations_job_id ON invocations(job_id);

CREATE TABLE snapshots (
    id             TEXT PRIMARY KEY NOT NULL,
    job_id         TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    invocation_id  TEXT NOT NULL REFERENCES invocations(id) ON DELETE CASCADE,
    snapshot_path  TEXT NOT NULL,
    link_dest_path TEXT,
    created_at     TEXT NOT NULL,
    size_bytes     INTEGER NOT NULL DEFAULT 0,
    file_count     INTEGER NOT NULL DEFAULT 0,
    is_latest      INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_snapshots_job_id ON snapshots(job_id);
