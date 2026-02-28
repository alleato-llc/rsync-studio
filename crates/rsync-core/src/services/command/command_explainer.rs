use crate::models::command::{ArgCategory, ArgumentExplanation, CommandExplanation, ParsedCommand};

/// Get a human-readable description for a recognized flag name.
pub fn explain_flag(flag: &str) -> &'static str {
    match flag {
        // Core / Archive flags
        "archive" => "Archive mode (-a): preserves permissions, timestamps, symlinks, owner, group, and recurses into directories. Equivalent to -rlptgoD.",
        "recursive" => "Recursive (-r): recurses into directories. Already included in archive mode (-a).",
        "links" => "Links (-l): copies symlinks as symlinks. Already included in archive mode (-a).",
        "perms" => "Perms (-p): preserves file permissions. Already included in archive mode (-a).",
        "times" => "Times (-t): preserves file modification times. Already included in archive mode (-a).",
        "group" => "Group (-g): preserves the group ownership of files. Already included in archive mode (-a).",
        "owner" => "Owner (-o): preserves the owner of files. Already included in archive mode (-a).",
        "devices_specials" => "Devices & Specials (-D): preserves device files and special files. Already included in archive mode (-a).",
        "devices" => "Devices (--devices): preserves device files. Part of -D.",
        "specials" => "Specials (--specials): preserves special files. Part of -D.",

        // Transfer behavior
        "compress" => "Compress (-z): compresses data during transfer to reduce bandwidth usage.",
        "dry_run" => "Dry run (-n/--dry-run): simulates the transfer without making any changes. Useful for previewing what would happen.",
        "partial" => "Partial (--partial): keeps partially transferred files so interrupted transfers can resume.",
        "update" => "Update (-u): skips files that are newer on the destination than the source.",
        "checksum" => "Checksum (-c): uses checksums instead of file size and modification time to decide whether to transfer a file.",
        "whole_file" => "Whole file (-W/--whole-file): disables rsync's delta-transfer algorithm and transfers whole files. Faster on fast networks.",
        "one_file_system" => "One file system (-x/--one-file-system): doesn't cross filesystem boundaries when recursing.",
        "ignore_existing" => "Ignore existing (--ignore-existing): skips files that already exist on the destination.",
        "size_only" => "Size only (--size-only): compares files by size only, ignoring modification times. Essential for NAS/SMB mounts where timestamps are unreliable.",
        "inplace" => "In-place (--inplace): updates destination files in-place instead of creating a temporary copy. Reduces disk I/O but less safe if interrupted.",
        "append" => "Append (--append): appends data onto shorter files without verifying existing content. Useful for log files.",
        "append_verify" => "Append-verify (--append-verify): like --append, but verifies the existing data using checksums.",
        "sparse" => "Sparse (-S/--sparse): handles sparse files efficiently, preserving their sparse nature on the destination.",
        "existing" => "Existing (--existing): only updates files that already exist on the destination; does not create new files.",
        "delay_updates" => "Delay updates (--delay-updates): puts updated files into a temporary directory first, then moves them into place at the end for more atomic updates.",
        "relative" => "Relative (-R/--relative): preserves full path information by sending implied directories.",
        "no_relative" => "No relative (--no-relative): disables --relative, sending only the final component of the source path.",

        // Symlink handling
        "copy_links" => "Copy links (-L/--copy-links): transforms symlinks into the files/dirs they point to.",
        "copy_dirlinks" => "Copy dirlinks (-k/--copy-dirlinks): transforms symlinks to directories on the sender into real directories.",
        "keep_dirlinks" => "Keep dirlinks (-K/--keep-dirlinks): treats symlinked dirs on the receiver as real dirs.",
        "safe_links" => "Safe links (--safe-links): ignores symlinks that point outside the copied tree.",

        // Metadata preservation
        "hard_links" => "Hard links (-H/--hard-links): preserves hard links between files.",
        "acls" => "ACLs (-A/--acls): preserves Access Control Lists.",
        "xattrs" => "Extended attributes (-X/--xattrs): preserves extended attributes.",
        "numeric_ids" => "Numeric IDs (--numeric-ids): transfers numeric group and user IDs rather than mapping them by name.",
        "no_perms" => "No perms (--no-perms): does not set file permissions on the destination.",
        "no_times" => "No times (--no-times): does not set file modification times on the destination.",
        "no_owner" => "No owner (--no-owner): does not set file owner on the destination.",
        "no_group" => "No group (--no-group): does not set file group on the destination.",
        "chmod" => "Chmod (--chmod=MODE): applies additional permission changes to affected files.",
        "chown" => "Chown (--chown=USER:GROUP): sets file ownership on the destination.",
        "super_" => "Super (--super): allows the receiving rsync to perform operations requiring super-user privileges.",
        "fake_super" => "Fake super (--fake-super): stores/recovers privileged attributes using extended attributes.",
        "no_implied_dirs" => "No implied dirs (--no-implied-dirs): doesn't send implied directory info, affecting how relative paths are handled.",

        // Output
        "verbose" => "Verbose (-v): increases the amount of information displayed during transfer.",
        "human_readable" => "Human-readable (-h): outputs numbers in a more human-readable format (e.g., 1.5M instead of 1572864).",
        "progress" => "Progress (--progress): shows transfer progress for each file during the sync.",
        "quiet" => "Quiet (-q): suppresses non-error messages during transfer.",
        "itemize_changes" => "Itemize changes (-i/--itemize-changes): outputs a change-summary for all updates.",
        "stats" => "Stats (--stats): prints a set of statistics about the file transfer at the end.",
        "log_file" => "Log file (--log-file=FILE): logs what rsync is doing to the specified file.",
        "out_format" => "Out format (--out-format=FORMAT): uses the specified format string for output.",
        "info" => "Info (--info=FLAGS): fine-grained control over informational output.",
        "debug" => "Debug (--debug=FLAGS): fine-grained control over debug output.",
        "msgs2stderr" => "Messages to stderr (--msgs2stderr): sends all messages to stderr rather than stdout.",

        // Deletion
        "delete" => "Delete (--delete): removes files from the destination that don't exist in the source, making it a true mirror.",
        "delete_before" => "Delete before (--delete-before): deletes files on the destination before the transfer begins.",
        "delete_during" => "Delete during (--delete-during): deletes files on the destination during the transfer (default --delete behavior).",
        "delete_delay" => "Delete delay (--delete-delay): finds files to delete during transfer, but deletes them after.",
        "delete_after" => "Delete after (--delete-after): deletes files on the destination after the transfer completes.",
        "delete_excluded" => "Delete excluded (--delete-excluded): also deletes excluded files from the destination.",
        "force" => "Force (--force): forces deletion of non-empty directories when they are replaced by non-directories.",
        "max_delete" => "Max delete (--max-delete=NUM): limits the number of files deleted on the destination.",
        "ignore_errors" => "Ignore errors (--ignore-errors): deletes files on the destination even when there are I/O errors.",

        // Backup
        "backup" => "Backup (--backup/-b): renames preexisting destination files before replacing them.",
        "backup_dir" => "Backup dir (--backup-dir=DIR): moves replaced/deleted files to the specified backup directory.",
        "suffix" => "Suffix (--suffix=SUFFIX): sets the backup suffix (default ~).",

        // File selection
        "files_from" => "Files from (--files-from=FILE): reads a list of source files from the specified file.",
        "filter" => "Filter (--filter=RULE): adds a file-filtering rule.",
        "exclude_from" => "Exclude from (--exclude-from=FILE): reads exclude patterns from the specified file.",
        "include_from" => "Include from (--include-from=FILE): reads include patterns from the specified file.",
        "prune_empty_dirs" => "Prune empty dirs (-m/--prune-empty-dirs): removes empty directory chains from the file list.",
        "max_size" => "Max size (--max-size=SIZE): skips files larger than the specified size.",
        "min_size" => "Min size (--min-size=SIZE): skips files smaller than the specified size.",
        "fuzzy" => "Fuzzy (--fuzzy/-y): looks for a basis file for any missing destination file using a fuzzy match.",

        // Compression
        "compress_level" => "Compress level (--compress-level=NUM): sets the compression level explicitly.",
        "skip_compress" => "Skip compress (--skip-compress=LIST): skips compression for files with the listed suffixes.",

        // Networking
        "blocking_io" => "Blocking I/O (--blocking-io): uses blocking I/O for the remote shell transport.",
        "contimeout" => "Connection timeout (--contimeout=SECONDS): sets the connection timeout in seconds.",
        "timeout" => "Timeout (--timeout=SECONDS): sets the I/O timeout in seconds.",
        "address" => "Address (--address=ADDRESS): binds to the specified address for outgoing connections.",
        "port" => "Port (--port=PORT): specifies a non-default port for the rsync daemon.",
        "ipv4" => "IPv4 (-4/--ipv4): prefers IPv4 for connections.",
        "ipv6" => "IPv6 (-6/--ipv6): prefers IPv6 for connections.",

        // Misc
        "temp_dir" => "Temp dir (--temp-dir=DIR): creates temporary files in the specified directory.",
        "compare_dest" => "Compare dest (--compare-dest=DIR): compares received files against DIR to skip unchanged files.",
        "copy_dest" => "Copy dest (--copy-dest=DIR): like --compare-dest, but also locally copies unchanged files from DIR.",
        "rsync_path" => "Rsync path (--rsync-path=PROGRAM): specifies the path to rsync on the remote machine.",
        "iconv" => "Iconv (--iconv=CONVERT_SPEC): converts filenames between character sets.",

        _ => "Argument not recognized. Please check the rsync manual for more details.",
    }
}

/// Determine the category for a flag based on its name.
fn flag_category(flag: &str) -> ArgCategory {
    match flag {
        // File handling
        "delete" | "delete_before" | "delete_during" | "delete_delay" | "delete_after"
        | "delete_excluded" | "force" | "max_delete" | "ignore_errors" => ArgCategory::Deletion,

        "checksum" | "update" | "whole_file" | "ignore_existing" | "one_file_system"
        | "size_only" | "inplace" | "append" | "append_verify" | "sparse"
        | "existing" | "delay_updates" | "relative" | "no_relative"
        | "copy_links" | "copy_dirlinks" | "keep_dirlinks" | "safe_links"
        | "files_from" | "filter" | "exclude_from" | "include_from"
        | "prune_empty_dirs" | "max_size" | "min_size" | "fuzzy" => ArgCategory::FileHandling,

        // Metadata
        "hard_links" | "acls" | "xattrs" | "numeric_ids"
        | "no_perms" | "no_times" | "no_owner" | "no_group"
        | "chmod" | "chown" | "super_" | "fake_super" | "no_implied_dirs" => ArgCategory::Metadata,

        // Output
        "verbose" | "human_readable" | "progress" | "quiet"
        | "itemize_changes" | "stats" | "log_file" | "out_format"
        | "info" | "debug" | "msgs2stderr" => ArgCategory::Output,

        // Performance / networking
        "compress" | "compress_level" | "skip_compress"
        | "blocking_io" | "contimeout" | "timeout" | "address" | "port"
        | "ipv4" | "ipv6" => ArgCategory::Performance,

        // Backup
        "backup" | "backup_dir" | "suffix" => ArgCategory::Flag,

        // Misc
        "temp_dir" | "compare_dest" | "copy_dest" | "rsync_path" | "iconv" => ArgCategory::Flag,

        // Everything else (archive sub-flags, etc.)
        _ => ArgCategory::Flag,
    }
}

/// Generate a full explanation for a parsed rsync command.
pub fn explain_command(parsed: &ParsedCommand) -> CommandExplanation {
    let mut arguments = Vec::new();

    // Explain flags
    for flag in &parsed.flags {
        arguments.push(ArgumentExplanation {
            argument: flag.clone(),
            description: explain_flag(flag).to_string(),
            category: flag_category(flag),
        });
    }

    // Explain exclude patterns
    for pattern in &parsed.exclude_patterns {
        arguments.push(ArgumentExplanation {
            argument: format!("--exclude={}", pattern),
            description: format!(
                "Exclude files matching the pattern '{}' from the transfer.",
                pattern
            ),
            category: ArgCategory::Pattern,
        });
    }

    // Explain include patterns
    for pattern in &parsed.include_patterns {
        arguments.push(ArgumentExplanation {
            argument: format!("--include={}", pattern),
            description: format!(
                "Include files matching the pattern '{}' in the transfer (overrides excludes).",
                pattern
            ),
            category: ArgCategory::Pattern,
        });
    }

    // Explain bandwidth limit
    if let Some(limit) = parsed.bandwidth_limit {
        arguments.push(ArgumentExplanation {
            argument: format!("--bwlimit={}", limit),
            description: format!(
                "Limits the transfer bandwidth to {} KB/s to avoid saturating the network.",
                limit
            ),
            category: ArgCategory::Performance,
        });
    }

    // Explain link-dest
    if let Some(ref link_dest) = parsed.link_dest {
        arguments.push(ArgumentExplanation {
            argument: format!("--link-dest={}", link_dest),
            description: format!(
                "Uses '{}' as a reference directory. Unchanged files are hard-linked from this directory instead of being copied, saving disk space.",
                link_dest
            ),
            category: ArgCategory::Performance,
        });
    }

    // Explain SSH command
    if let Some(ref ssh_cmd) = parsed.ssh_command {
        arguments.push(ArgumentExplanation {
            argument: format!("-e \"{}\"", ssh_cmd),
            description: format!(
                "Uses a custom remote shell command: '{}'. This configures how rsync connects to remote hosts.",
                ssh_cmd
            ),
            category: ArgCategory::Ssh,
        });
    }

    // Explain source
    if let Some(ref source) = parsed.source {
        arguments.push(ArgumentExplanation {
            argument: source.clone(),
            description: format!("Source: files will be read from '{}'.", source),
            category: ArgCategory::Path,
        });
    }

    // Explain destination
    if let Some(ref dest) = parsed.destination {
        arguments.push(ArgumentExplanation {
            argument: dest.clone(),
            description: format!("Destination: files will be written to '{}'.", dest),
            category: ArgCategory::Path,
        });
    }

    // Explain custom/unknown args
    for arg in &parsed.custom_args {
        arguments.push(ArgumentExplanation {
            argument: arg.clone(),
            description: "Argument not recognized. Please check the rsync manual for more details.".to_string(),
            category: ArgCategory::Unknown,
        });
    }

    // Build summary
    let summary = build_summary(parsed);

    CommandExplanation {
        arguments,
        summary,
    }
}

fn build_summary(parsed: &ParsedCommand) -> String {
    let mut parts = Vec::new();

    let has_archive = parsed.flags.contains(&"archive".to_string());
    let has_delete = parsed.flags.contains(&"delete".to_string());
    let has_dry_run = parsed.flags.contains(&"dry_run".to_string());
    let has_compress = parsed.flags.contains(&"compress".to_string());

    if has_dry_run {
        parts.push("This is a DRY RUN — no actual changes will be made.".to_string());
    }

    if has_archive && has_delete {
        parts.push("Mirrors the source to the destination, preserving all file attributes and deleting files not present in the source.".to_string());
    } else if has_archive {
        parts.push("Syncs files from source to destination, preserving permissions, timestamps, and other attributes.".to_string());
    } else {
        parts.push("Transfers files from source to destination.".to_string());
    }

    if has_compress {
        parts.push("Data is compressed during transfer.".to_string());
    }

    if !parsed.exclude_patterns.is_empty() {
        parts.push(format!(
            "{} pattern(s) are excluded.",
            parsed.exclude_patterns.len()
        ));
    }

    if parsed.bandwidth_limit.is_some() {
        parts.push("Bandwidth is limited.".to_string());
    }

    if parsed.link_dest.is_some() {
        parts.push("Using hard-link deduplication from a reference snapshot.".to_string());
    }

    if parsed.ssh_command.is_some() {
        parts.push("Connecting via custom SSH configuration.".to_string());
    }

    parts.join(" ")
}
