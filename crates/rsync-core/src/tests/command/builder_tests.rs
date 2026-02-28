use crate::models::job::{
    AdvancedOptions, CoreTransferOptions, FileHandlingOptions, MetadataOptions, OutputOptions,
    RsyncOptions, SshConfig, StorageLocation,
};
use crate::services::command_builder::build_rsync_args;

fn local(path: &str) -> StorageLocation {
    StorageLocation::Local {
        path: path.to_string(),
    }
}

fn default_opts() -> RsyncOptions {
    RsyncOptions::default()
}

#[test]
fn test_basic_archive_args() {
    let args = build_rsync_args(
        &local("/src/"),
        &local("/dst/"),
        &RsyncOptions {
            core_transfer: CoreTransferOptions {
                archive: true,
                ..Default::default()
            },
            ..default_opts()
        },
        None,
        None,
        false,
    );
    assert!(args.contains(&"-a".to_string()));
}

#[test]
fn test_all_flags_enabled() {
    let options = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            compress: true,
            partial: true,
            dry_run: true,
        },
        output: OutputOptions {
            verbose: true,
            progress: true,
            human_readable: true,
            ..Default::default()
        },
        file_handling: FileHandlingOptions {
            delete: true,
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);

    assert!(args.contains(&"-a".to_string()));
    assert!(args.contains(&"-z".to_string()));
    assert!(args.contains(&"-v".to_string()));
    assert!(args.contains(&"--delete".to_string()));
    assert!(args.contains(&"--dry-run".to_string()));
    assert!(args.contains(&"--partial".to_string()));
    assert!(args.contains(&"--progress".to_string()));
    assert!(args.contains(&"-h".to_string()));
}

#[test]
fn test_ssh_config_produces_e_flag() {
    let ssh = SshConfig {
        port: 2222,
        identity_file: Some("/home/user/.ssh/key".to_string()),
        strict_host_key_checking: true,
        custom_ssh_command: None,
    };
    let args = build_rsync_args(
        &local("/src/"),
        &local("/dst/"),
        &default_opts(),
        Some(&ssh),
        None,
        false,
    );

    assert!(args.contains(&"-e".to_string()));
    let ssh_cmd = args
        .iter()
        .find(|a| a.starts_with("ssh"))
        .expect("should have ssh command");
    assert!(ssh_cmd.contains("-p 2222"));
    assert!(ssh_cmd.contains("-i /home/user/.ssh/key"));
}

#[test]
fn test_local_paths() {
    let args = build_rsync_args(
        &local("/home/user/docs/"),
        &local("/backup/docs/"),
        &default_opts(),
        None,
        None,
        false,
    );
    assert!(args.contains(&"/home/user/docs/".to_string()));
    assert!(args.contains(&"/backup/docs/".to_string()));
}

#[test]
fn test_remote_ssh_paths() {
    let source = StorageLocation::RemoteSsh {
        user: "admin".to_string(),
        host: "server.example.com".to_string(),
        port: 22,
        path: "/data/backup/".to_string(),
        identity_file: None,
    };
    let args = build_rsync_args(&source, &local("/local/"), &default_opts(), None, None, false);
    assert!(args.contains(&"admin@server.example.com:/data/backup/".to_string()));
}

#[test]
fn test_remote_rsync_paths() {
    let dest = StorageLocation::RemoteRsync {
        host: "rsync.example.com".to_string(),
        module: "backups".to_string(),
        path: "daily/".to_string(),
    };
    let args = build_rsync_args(&local("/src/"), &dest, &default_opts(), None, None, false);
    assert!(args.contains(&"rsync://rsync.example.com/backups/daily/".to_string()));
}

#[test]
fn test_exclude_patterns() {
    let options = RsyncOptions {
        advanced: AdvancedOptions {
            exclude_patterns: vec!["*.log".to_string(), "tmp/".to_string(), ".git".to_string()],
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);

    assert!(args.contains(&"--exclude=*.log".to_string()));
    assert!(args.contains(&"--exclude=tmp/".to_string()));
    assert!(args.contains(&"--exclude=.git".to_string()));
}

#[test]
fn test_link_dest() {
    let args = build_rsync_args(
        &local("/src/"),
        &local("/dst/"),
        &default_opts(),
        None,
        Some("/prev/snapshot"),
        false,
    );
    assert!(args.contains(&"--link-dest=/prev/snapshot".to_string()));
}

#[test]
fn test_bandwidth_limit() {
    let options = RsyncOptions {
        advanced: AdvancedOptions {
            bandwidth_limit: Some(1000),
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--bwlimit=1000".to_string()));
}

#[test]
fn test_custom_args_appended() {
    let options = RsyncOptions {
        advanced: AdvancedOptions {
            custom_args: vec!["--checksum".to_string(), "--info=progress2".to_string()],
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);

    // Custom args should be before source/dest (which are last two)
    let checksum_pos = args.iter().position(|a| a == "--checksum").unwrap();
    let src_pos = args.iter().position(|a| a == "/src/").unwrap();
    assert!(checksum_pos < src_pos);
    assert!(args.contains(&"--info=progress2".to_string()));
}

#[test]
fn test_auto_trailing_slash_appends() {
    let args = build_rsync_args(
        &local("/home/user/docs"),
        &local("/backup/docs"),
        &default_opts(),
        None,
        None,
        true,
    );
    assert!(args.contains(&"/home/user/docs/".to_string()));
    assert!(args.contains(&"/backup/docs/".to_string()));
}

#[test]
fn test_size_only_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions {
            size_only: true,
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--size-only".to_string()));
}

#[test]
fn test_size_only_disabled_by_default() {
    let args = build_rsync_args(
        &local("/src/"),
        &local("/dst/"),
        &default_opts(),
        None,
        None,
        false,
    );
    assert!(!args.contains(&"--size-only".to_string()));
}

#[test]
fn test_checksum_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions { checksum: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--checksum".to_string()));
}

#[test]
fn test_update_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions { update: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--update".to_string()));
}

#[test]
fn test_whole_file_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions { whole_file: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--whole-file".to_string()));
}

#[test]
fn test_ignore_existing_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions { ignore_existing: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--ignore-existing".to_string()));
}

#[test]
fn test_one_file_system_flag() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions { one_file_system: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--one-file-system".to_string()));
}

#[test]
fn test_hard_links_flag() {
    let options = RsyncOptions {
        metadata: MetadataOptions { hard_links: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--hard-links".to_string()));
}

#[test]
fn test_acls_flag() {
    let options = RsyncOptions {
        metadata: MetadataOptions { acls: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--acls".to_string()));
}

#[test]
fn test_xattrs_flag() {
    let options = RsyncOptions {
        metadata: MetadataOptions { xattrs: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--xattrs".to_string()));
}

#[test]
fn test_numeric_ids_flag() {
    let options = RsyncOptions {
        metadata: MetadataOptions { numeric_ids: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--numeric-ids".to_string()));
}

#[test]
fn test_stats_flag() {
    let options = RsyncOptions {
        output: OutputOptions { stats: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--stats".to_string()));
}

#[test]
fn test_itemize_changes_flag() {
    let options = RsyncOptions {
        output: OutputOptions { itemize_changes: true, ..Default::default() },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--itemize-changes".to_string()));
}

#[test]
fn test_all_11_promoted_flags_together() {
    let options = RsyncOptions {
        file_handling: FileHandlingOptions {
            checksum: true,
            update: true,
            whole_file: true,
            ignore_existing: true,
            one_file_system: true,
            ..Default::default()
        },
        metadata: MetadataOptions {
            hard_links: true,
            acls: true,
            xattrs: true,
            numeric_ids: true,
        },
        output: OutputOptions {
            stats: true,
            itemize_changes: true,
            ..Default::default()
        },
        ..default_opts()
    };
    let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None, false);
    assert!(args.contains(&"--checksum".to_string()));
    assert!(args.contains(&"--update".to_string()));
    assert!(args.contains(&"--whole-file".to_string()));
    assert!(args.contains(&"--ignore-existing".to_string()));
    assert!(args.contains(&"--one-file-system".to_string()));
    assert!(args.contains(&"--hard-links".to_string()));
    assert!(args.contains(&"--acls".to_string()));
    assert!(args.contains(&"--xattrs".to_string()));
    assert!(args.contains(&"--numeric-ids".to_string()));
    assert!(args.contains(&"--stats".to_string()));
    assert!(args.contains(&"--itemize-changes".to_string()));
}

#[test]
fn test_auto_trailing_slash_no_double() {
    let args = build_rsync_args(
        &local("/home/user/docs/"),
        &local("/backup/docs/"),
        &default_opts(),
        None,
        None,
        true,
    );
    assert!(args.contains(&"/home/user/docs/".to_string()));
    assert!(args.contains(&"/backup/docs/".to_string()));
    // Ensure no double slashes
    assert!(!args.iter().any(|a| a.ends_with("//")));
}
