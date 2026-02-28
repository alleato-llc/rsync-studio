use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreTransferOptions {
    #[serde(default = "default_true")]
    pub archive: bool,
    #[serde(default)]
    pub compress: bool,
    #[serde(default)]
    pub partial: bool,
    #[serde(default)]
    pub dry_run: bool,
}

impl Default for CoreTransferOptions {
    fn default() -> Self {
        Self {
            archive: true,
            compress: false,
            partial: false,
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileHandlingOptions {
    #[serde(default)]
    pub delete: bool,
    #[serde(default)]
    pub size_only: bool,
    #[serde(default)]
    pub checksum: bool,
    #[serde(default)]
    pub update: bool,
    #[serde(default)]
    pub whole_file: bool,
    #[serde(default)]
    pub ignore_existing: bool,
    #[serde(default)]
    pub one_file_system: bool,
}

impl Default for FileHandlingOptions {
    fn default() -> Self {
        Self {
            delete: false,
            size_only: false,
            checksum: false,
            update: false,
            whole_file: false,
            ignore_existing: false,
            one_file_system: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataOptions {
    #[serde(default)]
    pub hard_links: bool,
    #[serde(default)]
    pub acls: bool,
    #[serde(default)]
    pub xattrs: bool,
    #[serde(default)]
    pub numeric_ids: bool,
}

impl Default for MetadataOptions {
    fn default() -> Self {
        Self {
            hard_links: false,
            acls: false,
            xattrs: false,
            numeric_ids: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputOptions {
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub progress: bool,
    #[serde(default)]
    pub human_readable: bool,
    #[serde(default)]
    pub stats: bool,
    #[serde(default)]
    pub itemize_changes: bool,
}

impl Default for OutputOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            progress: false,
            human_readable: false,
            stats: false,
            itemize_changes: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdvancedOptions {
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    pub bandwidth_limit: Option<u64>,
    #[serde(default)]
    pub custom_args: Vec<String>,
}

impl Default for AdvancedOptions {
    fn default() -> Self {
        Self {
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            bandwidth_limit: None,
            custom_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RsyncOptions {
    #[serde(default)]
    pub core_transfer: CoreTransferOptions,
    #[serde(default)]
    pub file_handling: FileHandlingOptions,
    #[serde(default)]
    pub metadata: MetadataOptions,
    #[serde(default)]
    pub output: OutputOptions,
    #[serde(default)]
    pub advanced: AdvancedOptions,
}

impl Default for RsyncOptions {
    fn default() -> Self {
        Self {
            core_transfer: CoreTransferOptions::default(),
            file_handling: FileHandlingOptions::default(),
            metadata: MetadataOptions::default(),
            output: OutputOptions::default(),
            advanced: AdvancedOptions::default(),
        }
    }
}
