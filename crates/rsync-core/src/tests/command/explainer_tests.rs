use crate::models::command::ArgCategory;
use crate::services::command_explainer::{explain_command, explain_flag};
use crate::services::command_parser::parse_rsync_command;

#[test]
fn explain_basic_mirror() {
    let parsed = parse_rsync_command("rsync -a --delete /src/ /dst/").unwrap();
    let explanation = explain_command(&parsed);

    assert!(explanation.summary.contains("Mirrors"));
    assert!(explanation
        .arguments
        .iter()
        .any(|a| a.argument == "archive"));
    assert!(explanation.arguments.iter().any(|a| a.argument == "delete"));
}

#[test]
fn explain_dry_run_mentioned() {
    let parsed = parse_rsync_command("rsync -an /src/ /dst/").unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation.summary.contains("DRY RUN"));
}

#[test]
fn explain_excludes() {
    let parsed =
        parse_rsync_command("rsync -a --exclude=*.log --exclude=.git /src/ /dst/")
            .unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation.summary.contains("2 pattern(s) are excluded"));
    assert!(explanation
        .arguments
        .iter()
        .any(|a| a.argument == "--exclude=*.log" && a.category == ArgCategory::Pattern));
}

#[test]
fn explain_unknown_args() {
    let parsed =
        parse_rsync_command("rsync -a --weird-flag /src/ /dst/").unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation
        .arguments
        .iter()
        .any(|a| a.argument == "--weird-flag" && a.category == ArgCategory::Unknown));
}

#[test]
fn explain_ssh_command() {
    let parsed = parse_rsync_command(
        r#"rsync -a -e "ssh -p 2222" /src/ user@host:/dst/"#,
    )
    .unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation.summary.contains("SSH"));
    assert!(explanation
        .arguments
        .iter()
        .any(|a| a.category == ArgCategory::Ssh));
}

#[test]
fn explain_bandwidth_limit() {
    let parsed =
        parse_rsync_command("rsync -a --bwlimit=500 /src/ /dst/").unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation.summary.contains("Bandwidth is limited"));
    assert!(explanation
        .arguments
        .iter()
        .any(|a| a.argument == "--bwlimit=500" && a.category == ArgCategory::Performance));
}

#[test]
fn explain_link_dest() {
    let parsed = parse_rsync_command(
        "rsync -a --link-dest=/prev/snap /src/ /dst/",
    )
    .unwrap();
    let explanation = explain_command(&parsed);
    assert!(explanation.summary.contains("hard-link deduplication"));
}

#[test]
fn all_flag_descriptions_are_non_empty() {
    let known_flags = [
        // Core / archive
        "archive", "recursive", "links", "perms", "times", "group", "owner",
        "devices_specials", "devices", "specials",
        // Transfer behavior
        "compress", "dry_run", "partial", "update", "checksum", "whole_file",
        "one_file_system", "ignore_existing", "size_only", "inplace", "append",
        "append_verify", "sparse", "existing", "delay_updates", "relative", "no_relative",
        // Symlink handling
        "copy_links", "copy_dirlinks", "keep_dirlinks", "safe_links",
        // Metadata
        "hard_links", "acls", "xattrs", "numeric_ids",
        "no_perms", "no_times", "no_owner", "no_group",
        "chmod", "chown", "super_", "fake_super", "no_implied_dirs",
        // Output
        "verbose", "human_readable", "progress", "quiet", "itemize_changes", "stats",
        "log_file", "out_format", "info", "debug", "msgs2stderr",
        // Deletion
        "delete", "delete_before", "delete_during", "delete_delay", "delete_after",
        "delete_excluded", "force", "max_delete", "ignore_errors",
        // Backup
        "backup", "backup_dir", "suffix",
        // File selection
        "files_from", "filter", "exclude_from", "include_from",
        "prune_empty_dirs", "max_size", "min_size", "fuzzy",
        // Compression
        "compress_level", "skip_compress",
        // Networking
        "blocking_io", "contimeout", "timeout", "address", "port", "ipv4", "ipv6",
        // Misc
        "temp_dir", "compare_dest", "copy_dest", "rsync_path", "iconv",
    ];

    let unrecognized_msg = "Argument not recognized. Please check the rsync manual for more details.";

    for flag in &known_flags {
        let desc = explain_flag(flag);
        assert!(
            desc != unrecognized_msg,
            "Flag '{}' has no description",
            flag
        );
    }
}

#[test]
fn unrecognized_flag_gets_manual_message() {
    let desc = explain_flag("totally_unknown_flag");
    assert_eq!(desc, "Argument not recognized. Please check the rsync manual for more details.");
}

#[test]
fn flag_categories_are_assigned() {
    let parsed = parse_rsync_command("rsync -avz --delete --stats --hard-links /src/ /dst/").unwrap();
    let explanation = explain_command(&parsed);

    // --delete should be Deletion category
    let delete_arg = explanation.arguments.iter().find(|a| a.argument == "delete").unwrap();
    assert_eq!(delete_arg.category, ArgCategory::Deletion);

    // --stats should be Output category
    let stats_arg = explanation.arguments.iter().find(|a| a.argument == "stats").unwrap();
    assert_eq!(stats_arg.category, ArgCategory::Output);

    // --hard-links should be Metadata category
    let hl_arg = explanation.arguments.iter().find(|a| a.argument == "hard_links").unwrap();
    assert_eq!(hl_arg.category, ArgCategory::Metadata);
}
