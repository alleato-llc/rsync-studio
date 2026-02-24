use crate::models::job::{RsyncOptions, SshConfig, StorageLocation};

pub fn build_rsync_args(
    source: &StorageLocation,
    destination: &StorageLocation,
    options: &RsyncOptions,
    ssh_config: Option<&SshConfig>,
    link_dest: Option<&str>,
) -> Vec<String> {
    let mut args = Vec::new();

    if options.archive {
        args.push("-a".to_string());
    }
    if options.compress {
        args.push("-z".to_string());
    }
    if options.verbose {
        args.push("-v".to_string());
    }
    if options.delete {
        args.push("--delete".to_string());
    }
    if options.dry_run {
        args.push("--dry-run".to_string());
    }
    if options.partial {
        args.push("--partial".to_string());
    }
    if options.progress {
        args.push("--progress".to_string());
    }
    if options.human_readable {
        args.push("-h".to_string());
    }

    for pattern in &options.exclude_patterns {
        args.push(format!("--exclude={}", pattern));
    }

    for pattern in &options.include_patterns {
        args.push(format!("--include={}", pattern));
    }

    if let Some(limit) = options.bandwidth_limit {
        args.push(format!("--bwlimit={}", limit));
    }

    if let Some(link) = link_dest {
        args.push(format!("--link-dest={}", link));
    }

    // Build SSH command if needed
    if let Some(ssh) = ssh_config {
        if let Some(ref custom_cmd) = ssh.custom_ssh_command {
            args.push(format!("-e {}", custom_cmd));
        } else {
            let mut ssh_parts = vec!["ssh".to_string()];

            if ssh.port != 22 {
                ssh_parts.push(format!("-p {}", ssh.port));
            }

            if let Some(ref key) = ssh.identity_file {
                ssh_parts.push(format!("-i {}", key));
            }

            if !ssh.strict_host_key_checking {
                ssh_parts.push("-o StrictHostKeyChecking=no".to_string());
            }

            if ssh_parts.len() > 1 {
                args.push("-e".to_string());
                args.push(ssh_parts.join(" "));
            }
        }
    }

    for arg in &options.custom_args {
        args.push(arg.clone());
    }

    args.push(source.to_rsync_path());
    args.push(destination.to_rsync_path());

    args
}

#[cfg(test)]
mod tests {
    use super::*;

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
                archive: true,
                ..default_opts()
            },
            None,
            None,
        );
        assert!(args.contains(&"-a".to_string()));
    }

    #[test]
    fn test_all_flags_enabled() {
        let options = RsyncOptions {
            archive: true,
            compress: true,
            verbose: true,
            delete: true,
            dry_run: true,
            partial: true,
            progress: true,
            human_readable: true,
            ..default_opts()
        };
        let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None);

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
        let args = build_rsync_args(&source, &local("/local/"), &default_opts(), None, None);
        assert!(args.contains(&"admin@server.example.com:/data/backup/".to_string()));
    }

    #[test]
    fn test_remote_rsync_paths() {
        let dest = StorageLocation::RemoteRsync {
            host: "rsync.example.com".to_string(),
            module: "backups".to_string(),
            path: "daily/".to_string(),
        };
        let args = build_rsync_args(&local("/src/"), &dest, &default_opts(), None, None);
        assert!(args.contains(&"rsync://rsync.example.com/backups/daily/".to_string()));
    }

    #[test]
    fn test_exclude_patterns() {
        let options = RsyncOptions {
            exclude_patterns: vec!["*.log".to_string(), "tmp/".to_string(), ".git".to_string()],
            ..default_opts()
        };
        let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None);

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
        );
        assert!(args.contains(&"--link-dest=/prev/snapshot".to_string()));
    }

    #[test]
    fn test_bandwidth_limit() {
        let options = RsyncOptions {
            bandwidth_limit: Some(1000),
            ..default_opts()
        };
        let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None);
        assert!(args.contains(&"--bwlimit=1000".to_string()));
    }

    #[test]
    fn test_custom_args_appended() {
        let options = RsyncOptions {
            custom_args: vec!["--checksum".to_string(), "--info=progress2".to_string()],
            ..default_opts()
        };
        let args = build_rsync_args(&local("/src/"), &local("/dst/"), &options, None, None);

        // Custom args should be before source/dest (which are last two)
        let checksum_pos = args.iter().position(|a| a == "--checksum").unwrap();
        let src_pos = args.iter().position(|a| a == "/src/").unwrap();
        assert!(checksum_pos < src_pos);
        assert!(args.contains(&"--info=progress2".to_string()));
    }
}
