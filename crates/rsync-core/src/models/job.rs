use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::schedule::ScheduleConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum BackupMode {
    Mirror,
    Versioned {
        backup_dir: String,
    },
    Snapshot {
        retention_policy: RetentionPolicy,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetentionPolicy {
    #[serde(default = "default_keep_daily")]
    pub keep_daily: u32,
    #[serde(default = "default_keep_weekly")]
    pub keep_weekly: u32,
    #[serde(default = "default_keep_monthly")]
    pub keep_monthly: u32,
}

fn default_keep_daily() -> u32 {
    7
}
fn default_keep_weekly() -> u32 {
    4
}
fn default_keep_monthly() -> u32 {
    6
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            keep_daily: default_keep_daily(),
            keep_weekly: default_keep_weekly(),
            keep_monthly: default_keep_monthly(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum StorageLocation {
    Local {
        path: String,
    },
    RemoteSsh {
        user: String,
        host: String,
        #[serde(default = "default_ssh_port")]
        port: u16,
        path: String,
        identity_file: Option<String>,
    },
    RemoteRsync {
        host: String,
        module: String,
        path: String,
    },
}

fn default_ssh_port() -> u16 {
    22
}

impl StorageLocation {
    pub fn to_rsync_path(&self) -> String {
        match self {
            StorageLocation::Local { path } => path.clone(),
            StorageLocation::RemoteSsh {
                user, host, path, ..
            } => format!("{}@{}:{}", user, host, path),
            StorageLocation::RemoteRsync {
                host, module, path, ..
            } => format!("rsync://{}/{}/{}", host, module, path),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SshConfig {
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub identity_file: Option<String>,
    #[serde(default = "default_true")]
    pub strict_host_key_checking: bool,
    pub custom_ssh_command: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            port: 22,
            identity_file: None,
            strict_host_key_checking: true,
            custom_ssh_command: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RsyncOptions {
    #[serde(default = "default_true")]
    pub archive: bool,
    #[serde(default)]
    pub compress: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub delete: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub partial: bool,
    #[serde(default)]
    pub progress: bool,
    #[serde(default)]
    pub human_readable: bool,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    pub bandwidth_limit: Option<u64>,
    #[serde(default)]
    pub custom_args: Vec<String>,
}

impl Default for RsyncOptions {
    fn default() -> Self {
        Self {
            archive: true,
            compress: false,
            verbose: false,
            delete: false,
            dry_run: false,
            partial: false,
            progress: false,
            human_readable: false,
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            bandwidth_limit: None,
            custom_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: StorageLocation,
    pub destination: StorageLocation,
    pub backup_mode: BackupMode,
    pub options: RsyncOptions,
    pub ssh_config: Option<SshConfig>,
    pub schedule: Option<ScheduleConfig>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Idle,
    Running,
    Completed,
    Failed,
    Cancelled,
}
