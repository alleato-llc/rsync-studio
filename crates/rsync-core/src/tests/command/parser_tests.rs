use crate::models::job::{
    AdvancedOptions, CoreTransferOptions, FileHandlingOptions, MetadataOptions, OutputOptions,
    RsyncOptions, StorageLocation,
};
use crate::services::command_builder::build_rsync_args;
use crate::services::command_parser::{
    parse_rsync_command, parse_ssh_command, parse_storage_location, to_job_definition,
};

#[test]
fn parse_simple_command() {
    let parsed = parse_rsync_command("rsync -a /src/ /dst/").unwrap();
    assert_eq!(parsed.source.as_deref(), Some("/src/"));
    assert_eq!(parsed.destination.as_deref(), Some("/dst/"));
    assert!(parsed.flags.contains(&"archive".to_string()));
}

#[test]
fn parse_combined_short_flags() {
    let parsed = parse_rsync_command("rsync -avz /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert!(parsed.flags.contains(&"verbose".to_string()));
    assert!(parsed.flags.contains(&"compress".to_string()));
}

#[test]
fn parse_long_flags() {
    let parsed =
        parse_rsync_command("rsync --archive --delete --dry-run /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert!(parsed.flags.contains(&"delete".to_string()));
    assert!(parsed.flags.contains(&"dry_run".to_string()));
}

#[test]
fn parse_exclude_patterns() {
    let parsed = parse_rsync_command(
        "rsync -a --exclude=*.log --exclude=tmp/ /src/ /dst/",
    )
    .unwrap();
    assert_eq!(parsed.exclude_patterns, vec!["*.log", "tmp/"]);
}

#[test]
fn parse_include_patterns() {
    let parsed =
        parse_rsync_command("rsync -a --include=*.rs --include=*.toml /src/ /dst/")
            .unwrap();
    assert_eq!(parsed.include_patterns, vec!["*.rs", "*.toml"]);
}

#[test]
fn parse_bandwidth_limit() {
    let parsed =
        parse_rsync_command("rsync -a --bwlimit=1000 /src/ /dst/").unwrap();
    assert_eq!(parsed.bandwidth_limit, Some(1000));
}

#[test]
fn parse_link_dest() {
    let parsed =
        parse_rsync_command("rsync -a --link-dest=/prev /src/ /dst/").unwrap();
    assert_eq!(parsed.link_dest.as_deref(), Some("/prev"));
}

#[test]
fn parse_ssh_flag() {
    let parsed = parse_rsync_command(
        r#"rsync -a -e "ssh -p 2222 -i /home/user/.ssh/key" /src/ user@host:/dst/"#,
    )
    .unwrap();
    assert_eq!(
        parsed.ssh_command.as_deref(),
        Some("ssh -p 2222 -i /home/user/.ssh/key")
    );
    assert_eq!(parsed.destination.as_deref(), Some("user@host:/dst/"));
}

#[test]
fn parse_remote_ssh_paths() {
    let loc = parse_storage_location("admin@server.example.com:/data/backup/");
    assert_eq!(
        loc,
        StorageLocation::RemoteSsh {
            user: "admin".to_string(),
            host: "server.example.com".to_string(),
            port: 22,
            path: "/data/backup/".to_string(),
            identity_file: None,
        }
    );
}

#[test]
fn parse_remote_rsync_paths() {
    let loc = parse_storage_location("rsync://rsync.example.com/backups/daily/");
    assert_eq!(
        loc,
        StorageLocation::RemoteRsync {
            host: "rsync.example.com".to_string(),
            module: "backups".to_string(),
            path: "daily/".to_string(),
        }
    );
}

#[test]
fn parse_local_path() {
    let loc = parse_storage_location("/home/user/docs/");
    assert_eq!(
        loc,
        StorageLocation::Local {
            path: "/home/user/docs/".to_string()
        }
    );
}

#[test]
fn parse_without_rsync_prefix() {
    let parsed = parse_rsync_command("-avz /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert_eq!(parsed.source.as_deref(), Some("/src/"));
}

#[test]
fn parse_capital_p_flag() {
    let parsed = parse_rsync_command("rsync -aP /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert!(parsed.flags.contains(&"partial".to_string()));
    assert!(parsed.flags.contains(&"progress".to_string()));
}

#[test]
fn parse_unknown_flags_go_to_custom() {
    let parsed =
        parse_rsync_command("rsync -a --weird-flag --foo=bar /src/ /dst/").unwrap();
    assert!(parsed.custom_args.contains(&"--weird-flag".to_string()));
    assert!(parsed.custom_args.contains(&"--foo=bar".to_string()));
}

#[test]
fn parse_new_short_flags() {
    let parsed = parse_rsync_command("rsync -aHAXWxiSRKLkbym46 /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert!(parsed.flags.contains(&"hard_links".to_string()));
    assert!(parsed.flags.contains(&"acls".to_string()));
    assert!(parsed.flags.contains(&"xattrs".to_string()));
    assert!(parsed.flags.contains(&"whole_file".to_string()));
    assert!(parsed.flags.contains(&"one_file_system".to_string()));
    assert!(parsed.flags.contains(&"itemize_changes".to_string()));
    assert!(parsed.flags.contains(&"sparse".to_string()));
    assert!(parsed.flags.contains(&"relative".to_string()));
    assert!(parsed.flags.contains(&"keep_dirlinks".to_string()));
    assert!(parsed.flags.contains(&"copy_links".to_string()));
    assert!(parsed.flags.contains(&"copy_dirlinks".to_string()));
    assert!(parsed.flags.contains(&"backup".to_string()));
    assert!(parsed.flags.contains(&"fuzzy".to_string()));
    assert!(parsed.flags.contains(&"prune_empty_dirs".to_string()));
    assert!(parsed.flags.contains(&"ipv4".to_string()));
    assert!(parsed.flags.contains(&"ipv6".to_string()));
}

#[test]
fn parse_new_long_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --inplace --append-verify --sparse --delete-before --fake-super --ipv6 /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"inplace".to_string()));
    assert!(parsed.flags.contains(&"append_verify".to_string()));
    assert!(parsed.flags.contains(&"sparse".to_string()));
    assert!(parsed.flags.contains(&"delete_before".to_string()));
    assert!(parsed.flags.contains(&"fake_super".to_string()));
    assert!(parsed.flags.contains(&"ipv6".to_string()));
}

#[test]
fn parse_value_flags_recognized() {
    let parsed = parse_rsync_command(
        "rsync -a --max-size=100M --compress-level=9 --chmod=ugo=rwX /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"max_size".to_string()));
    assert!(parsed.flags.contains(&"compress_level".to_string()));
    assert!(parsed.flags.contains(&"chmod".to_string()));
}

#[test]
fn parse_deletion_variants() {
    let parsed = parse_rsync_command(
        "rsync -a --delete-before --delete-excluded --force --ignore-errors /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"delete_before".to_string()));
    assert!(parsed.flags.contains(&"delete_excluded".to_string()));
    assert!(parsed.flags.contains(&"force".to_string()));
    assert!(parsed.flags.contains(&"ignore_errors".to_string()));

    let parsed2 = parse_rsync_command(
        "rsync -a --delete-during --delete-delay --delete-after /src/ /dst/"
    ).unwrap();
    assert!(parsed2.flags.contains(&"delete_during".to_string()));
    assert!(parsed2.flags.contains(&"delete_delay".to_string()));
    assert!(parsed2.flags.contains(&"delete_after".to_string()));
}

#[test]
fn parse_symlink_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --copy-links --copy-dirlinks --keep-dirlinks --safe-links /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"copy_links".to_string()));
    assert!(parsed.flags.contains(&"copy_dirlinks".to_string()));
    assert!(parsed.flags.contains(&"keep_dirlinks".to_string()));
    assert!(parsed.flags.contains(&"safe_links".to_string()));
}

#[test]
fn parse_metadata_negation_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --no-perms --no-times --no-owner --no-group --fake-super --super /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"no_perms".to_string()));
    assert!(parsed.flags.contains(&"no_times".to_string()));
    assert!(parsed.flags.contains(&"no_owner".to_string()));
    assert!(parsed.flags.contains(&"no_group".to_string()));
    assert!(parsed.flags.contains(&"fake_super".to_string()));
    assert!(parsed.flags.contains(&"super_".to_string()));
}

#[test]
fn parse_transfer_behavior_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --append --existing --delay-updates --relative --no-relative --backup --msgs2stderr /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"append".to_string()));
    assert!(parsed.flags.contains(&"existing".to_string()));
    assert!(parsed.flags.contains(&"delay_updates".to_string()));
    assert!(parsed.flags.contains(&"relative".to_string()));
    assert!(parsed.flags.contains(&"no_relative".to_string()));
    assert!(parsed.flags.contains(&"backup".to_string()));
    assert!(parsed.flags.contains(&"msgs2stderr".to_string()));
}

#[test]
fn parse_networking_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --blocking-io --ipv4 --ipv6 /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"blocking_io".to_string()));
    assert!(parsed.flags.contains(&"ipv4".to_string()));
    assert!(parsed.flags.contains(&"ipv6".to_string()));
}

#[test]
fn parse_value_carrying_flags_into_flags_vec() {
    let parsed = parse_rsync_command(
        "rsync -a --files-from=/tmp/list --log-file=/tmp/log --timeout=30 --contimeout=10 --address=127.0.0.1 --port=873 /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"files_from".to_string()));
    assert!(parsed.flags.contains(&"log_file".to_string()));
    assert!(parsed.flags.contains(&"timeout".to_string()));
    assert!(parsed.flags.contains(&"contimeout".to_string()));
    assert!(parsed.flags.contains(&"address".to_string()));
    assert!(parsed.flags.contains(&"port".to_string()));
}

#[test]
fn parse_more_value_carrying_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --rsync-path=/usr/local/bin/rsync --suffix=.bak --temp-dir=/tmp --compare-dest=/ref --copy-dest=/ref2 --filter='- *.o' --chown=root:root --skip-compress=gz/jpg --exclude-from=/tmp/exc --include-from=/tmp/inc --iconv=utf8 --info=progress2 --debug=del --out-format='%n' --max-delete=100 --min-size=1K /src/ /dst/"
    ).unwrap();
    assert!(parsed.flags.contains(&"rsync_path".to_string()));
    assert!(parsed.flags.contains(&"suffix".to_string()));
    assert!(parsed.flags.contains(&"temp_dir".to_string()));
    assert!(parsed.flags.contains(&"compare_dest".to_string()));
    assert!(parsed.flags.contains(&"copy_dest".to_string()));
    assert!(parsed.flags.contains(&"filter".to_string()));
    assert!(parsed.flags.contains(&"chown".to_string()));
    assert!(parsed.flags.contains(&"skip_compress".to_string()));
    assert!(parsed.flags.contains(&"exclude_from".to_string()));
    assert!(parsed.flags.contains(&"include_from".to_string()));
    assert!(parsed.flags.contains(&"iconv".to_string()));
    assert!(parsed.flags.contains(&"info".to_string()));
    assert!(parsed.flags.contains(&"debug".to_string()));
    assert!(parsed.flags.contains(&"out_format".to_string()));
    assert!(parsed.flags.contains(&"max_delete".to_string()));
    assert!(parsed.flags.contains(&"min_size".to_string()));
}

#[test]
fn parse_del_alias_for_delete_during() {
    let parsed = parse_rsync_command("rsync -a --del /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"delete_during".to_string()));
}

#[test]
fn parse_prune_empty_dirs_long() {
    let parsed = parse_rsync_command("rsync -a --prune-empty-dirs --fuzzy /src/ /dst/").unwrap();
    assert!(parsed.flags.contains(&"prune_empty_dirs".to_string()));
    assert!(parsed.flags.contains(&"fuzzy".to_string()));
}

#[test]
fn to_job_definition_all_11_promoted_flags() {
    let parsed = parse_rsync_command(
        "rsync -a --checksum --update --whole-file --ignore-existing --one-file-system --hard-links --acls --xattrs --numeric-ids --stats --itemize-changes /src/ /dst/"
    ).unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.file_handling.checksum);
    assert!(job.options.file_handling.update);
    assert!(job.options.file_handling.whole_file);
    assert!(job.options.file_handling.ignore_existing);
    assert!(job.options.file_handling.one_file_system);
    assert!(job.options.metadata.hard_links);
    assert!(job.options.metadata.acls);
    assert!(job.options.metadata.xattrs);
    assert!(job.options.metadata.numeric_ids);
    assert!(job.options.output.stats);
    assert!(job.options.output.itemize_changes);
}

#[test]
fn roundtrip_all_11_promoted_flags() {
    let source = StorageLocation::Local { path: "/src/".to_string() };
    let dest = StorageLocation::Local { path: "/dst/".to_string() };
    let opts = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            ..Default::default()
        },
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
        ..RsyncOptions::default()
    };

    let args = build_rsync_args(&source, &dest, &opts, None, None, false);
    let cmd = format!("rsync {}", args.join(" "));

    let parsed = parse_rsync_command(&cmd).unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.file_handling.checksum);
    assert!(job.options.file_handling.update);
    assert!(job.options.file_handling.whole_file);
    assert!(job.options.file_handling.ignore_existing);
    assert!(job.options.file_handling.one_file_system);
    assert!(job.options.metadata.hard_links);
    assert!(job.options.metadata.acls);
    assert!(job.options.metadata.xattrs);
    assert!(job.options.metadata.numeric_ids);
    assert!(job.options.output.stats);
    assert!(job.options.output.itemize_changes);
}

#[test]
fn roundtrip_file_handling_group() {
    let source = StorageLocation::Local { path: "/src/".to_string() };
    let dest = StorageLocation::Local { path: "/dst/".to_string() };
    let opts = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            ..Default::default()
        },
        file_handling: FileHandlingOptions {
            delete: true,
            size_only: true,
            checksum: true,
            update: true,
            whole_file: true,
            ignore_existing: true,
            one_file_system: true,
        },
        ..RsyncOptions::default()
    };

    let args = build_rsync_args(&source, &dest, &opts, None, None, false);
    let cmd = format!("rsync {}", args.join(" "));

    let parsed = parse_rsync_command(&cmd).unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.file_handling.delete, "delete");
    assert!(job.options.file_handling.size_only, "size_only");
    assert!(job.options.file_handling.checksum, "checksum");
    assert!(job.options.file_handling.update, "update");
    assert!(job.options.file_handling.whole_file, "whole_file");
    assert!(job.options.file_handling.ignore_existing, "ignore_existing");
    assert!(job.options.file_handling.one_file_system, "one_file_system");
}

#[test]
fn roundtrip_metadata_group() {
    let source = StorageLocation::Local { path: "/src/".to_string() };
    let dest = StorageLocation::Local { path: "/dst/".to_string() };
    let opts = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            ..Default::default()
        },
        metadata: MetadataOptions {
            hard_links: true,
            acls: true,
            xattrs: true,
            numeric_ids: true,
        },
        ..RsyncOptions::default()
    };

    let args = build_rsync_args(&source, &dest, &opts, None, None, false);
    let cmd = format!("rsync {}", args.join(" "));

    let parsed = parse_rsync_command(&cmd).unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.core_transfer.archive, "archive");
    assert!(job.options.metadata.hard_links, "hard_links");
    assert!(job.options.metadata.acls, "acls");
    assert!(job.options.metadata.xattrs, "xattrs");
    assert!(job.options.metadata.numeric_ids, "numeric_ids");
}

#[test]
fn roundtrip_output_group() {
    let source = StorageLocation::Local { path: "/src/".to_string() };
    let dest = StorageLocation::Local { path: "/dst/".to_string() };
    let opts = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            ..Default::default()
        },
        output: OutputOptions {
            verbose: true,
            progress: true,
            human_readable: true,
            stats: true,
            itemize_changes: true,
        },
        ..RsyncOptions::default()
    };

    let args = build_rsync_args(&source, &dest, &opts, None, None, false);
    let cmd = format!("rsync {}", args.join(" "));

    let parsed = parse_rsync_command(&cmd).unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.output.verbose, "verbose");
    assert!(job.options.output.progress, "progress");
    assert!(job.options.output.human_readable, "human_readable");
    assert!(job.options.output.stats, "stats");
    assert!(job.options.output.itemize_changes, "itemize_changes");
}

#[test]
fn to_job_definition_basic() {
    let parsed = parse_rsync_command("rsync -avz --delete /src/ /dst/").unwrap();
    let job = to_job_definition(&parsed).unwrap();
    assert!(job.options.core_transfer.archive);
    assert!(job.options.output.verbose);
    assert!(job.options.core_transfer.compress);
    assert!(job.options.file_handling.delete);
    assert_eq!(
        job.transfer.source,
        StorageLocation::Local {
            path: "/src/".to_string()
        }
    );
    assert_eq!(
        job.transfer.destination,
        StorageLocation::Local {
            path: "/dst/".to_string()
        }
    );
}

#[test]
fn to_job_definition_with_ssh() {
    let parsed = parse_rsync_command(
        r#"rsync -a -e "ssh -p 2222" /src/ admin@server:/backup/"#,
    )
    .unwrap();
    let job = to_job_definition(&parsed).unwrap();
    let ssh = job.ssh_config.unwrap();
    assert_eq!(ssh.port, 2222);
    assert!(matches!(
        job.transfer.destination,
        StorageLocation::RemoteSsh { .. }
    ));
}

#[test]
fn parse_ssh_command_parts() {
    let config = parse_ssh_command("ssh -p 2222 -i /key -o StrictHostKeyChecking=no");
    let config = config.unwrap();
    assert_eq!(config.port, 2222);
    assert_eq!(config.identity_file.as_deref(), Some("/key"));
    assert!(!config.strict_host_key_checking);
}

#[test]
fn roundtrip_simple_command() {
    let source = StorageLocation::Local {
        path: "/src/".to_string(),
    };
    let dest = StorageLocation::Local {
        path: "/dst/".to_string(),
    };
    let opts = RsyncOptions {
        core_transfer: CoreTransferOptions {
            archive: true,
            compress: true,
            ..Default::default()
        },
        output: OutputOptions {
            verbose: true,
            ..Default::default()
        },
        file_handling: FileHandlingOptions {
            delete: true,
            ..Default::default()
        },
        advanced: AdvancedOptions {
            exclude_patterns: vec!["*.log".to_string()],
            bandwidth_limit: Some(500),
            ..Default::default()
        },
        ..RsyncOptions::default()
    };

    let args = build_rsync_args(&source, &dest, &opts, None, None, false);
    let cmd = format!("rsync {}", args.join(" "));

    let parsed = parse_rsync_command(&cmd).unwrap();
    assert!(parsed.flags.contains(&"archive".to_string()));
    assert!(parsed.flags.contains(&"compress".to_string()));
    assert!(parsed.flags.contains(&"verbose".to_string()));
    assert!(parsed.flags.contains(&"delete".to_string()));
    assert_eq!(parsed.exclude_patterns, vec!["*.log"]);
    assert_eq!(parsed.bandwidth_limit, Some(500));
    assert_eq!(parsed.source.as_deref(), Some("/src/"));
    assert_eq!(parsed.destination.as_deref(), Some("/dst/"));
}
