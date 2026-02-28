use serde::{Deserialize, Serialize};

/// A single argument with its explanation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArgumentExplanation {
    /// The raw argument as it appeared in the command
    pub argument: String,
    /// Human-readable explanation
    pub description: String,
    /// Category of the argument
    pub category: ArgCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArgCategory {
    Flag,
    Pattern,
    Path,
    Ssh,
    Performance,
    FileHandling,
    Metadata,
    Output,
    Deletion,
    Unknown,
}

/// Full explanation of a parsed rsync command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandExplanation {
    /// Per-argument explanations
    pub arguments: Vec<ArgumentExplanation>,
    /// Overall summary of what the command does
    pub summary: String,
}

/// Result of parsing an rsync command string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedCommand {
    /// The source path/location as a raw string
    pub source: Option<String>,
    /// The destination path/location as a raw string
    pub destination: Option<String>,
    /// Recognized boolean flags (e.g., "archive", "compress")
    pub flags: Vec<String>,
    /// Exclude patterns found
    pub exclude_patterns: Vec<String>,
    /// Include patterns found
    pub include_patterns: Vec<String>,
    /// Bandwidth limit if specified
    pub bandwidth_limit: Option<u64>,
    /// SSH command string if -e was used
    pub ssh_command: Option<String>,
    /// Link-dest path if specified
    pub link_dest: Option<String>,
    /// Arguments not recognized by the parser
    pub custom_args: Vec<String>,
}
