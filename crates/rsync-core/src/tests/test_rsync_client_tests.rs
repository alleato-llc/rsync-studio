use std::path::Path;
use std::rc::Rc;

use crate::models::job::{RsyncOptions, SshConfig, StorageLocation};
use crate::services::command_builder;
use crate::tests::test_file_system::TestFileSystem;
use crate::tests::test_helpers::setup_test_env;
use crate::tests::test_rsync_client::TestRsyncClient;
use crate::traits::file_system::FileSystem;
use crate::traits::rsync_client::{RsyncClient, RsyncError};

#[test]
fn test_basic_sync_copies_files() {
    let (fs, client) = setup_test_env();
    let args = vec!["-a".to_string(), "/src/".to_string(), "/dst/".to_string()];
    client.execute(&args).unwrap();

    // Files are copied
    assert!(fs.is_file(Path::new("/dst/file1.txt")));
    assert!(fs.is_file(Path::new("/dst/file2.txt")));

    // Content is preserved
    assert_eq!(fs.file_content("/dst/file1.txt").unwrap(), "Hello");
    assert_eq!(fs.file_content("/dst/file2.txt").unwrap(), "World");

    // Nested directories are created
    assert!(fs.is_dir(Path::new("/dst/subdir")));
    assert_eq!(
        fs.file_content("/dst/subdir/file3.txt").unwrap(),
        "Nested content"
    );
}

#[test]
fn test_delete_flag_removes_extra_files() {
    let (fs, client) = setup_test_env();

    // Add extra file to destination
    fs.write(Path::new("/dst/extra.txt"), "should be deleted")
        .unwrap();

    let args = vec![
        "-a".to_string(),
        "--delete".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    assert!(!fs.exists(Path::new("/dst/extra.txt")));
    assert!(fs.is_file(Path::new("/dst/file1.txt")));
}

#[test]
fn test_exclude_pattern_skips_files() {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/data.txt", "data")
            .with_file("/src/debug.log", "log content")
            .with_file("/src/error.log", "error content")
            .with_dir("/dst"),
    );
    let client = TestRsyncClient::new(fs.clone());

    let args = vec![
        "-a".to_string(),
        "--exclude=*.log".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    assert!(fs.is_file(Path::new("/dst/data.txt")));
    assert!(!fs.exists(Path::new("/dst/debug.log")));
    assert!(!fs.exists(Path::new("/dst/error.log")));
}

#[test]
fn test_link_dest_hard_links_unchanged() {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/file.txt", "content")
            .with_file("/prev/file.txt", "content") // Same content as source
            .with_dir("/dst"),
    );
    let client = TestRsyncClient::new(fs.clone());

    let args = vec![
        "-a".to_string(),
        "--link-dest=/prev".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    assert!(fs.is_file(Path::new("/dst/file.txt")));
    assert!(fs.are_hard_linked("/prev/file.txt", "/dst/file.txt"));
}

#[test]
fn test_link_dest_copies_changed_files() {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/file.txt", "new content")
            .with_file("/prev/file.txt", "old content") // Different content
            .with_dir("/dst"),
    );
    let client = TestRsyncClient::new(fs.clone());

    let args = vec![
        "-a".to_string(),
        "--link-dest=/prev".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    assert!(fs.is_file(Path::new("/dst/file.txt")));
    assert!(!fs.are_hard_linked("/prev/file.txt", "/dst/file.txt"));
    assert_eq!(fs.file_content("/dst/file.txt").unwrap(), "new content");
}

#[test]
fn test_dry_run_no_filesystem_changes() {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/file.txt", "content")
            .with_dir("/dst"),
    );
    let client = TestRsyncClient::new(fs.clone());

    let args = vec![
        "-a".to_string(),
        "--dry-run".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    // Destination should still be empty (only the /dst dir)
    assert!(!fs.is_file(Path::new("/dst/file.txt")));
}

#[test]
fn test_commands_are_recorded() {
    let (_, client) = setup_test_env();

    let args1 = vec!["-a".to_string(), "/src/".to_string(), "/dst/".to_string()];
    let args2 = vec![
        "-a".to_string(),
        "-v".to_string(),
        "/src/".to_string(),
        "/dst2/".to_string(),
    ];
    client.execute(&args1).unwrap();
    client.execute(&args2).unwrap();

    let commands = client.recorded_commands();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].args, args1);
    assert_eq!(commands[1].args, args2);
}

#[test]
fn test_last_command() {
    let (_, client) = setup_test_env();

    let args1 = vec!["-a".to_string(), "/src/".to_string(), "/dst/".to_string()];
    let args2 = vec![
        "-a".to_string(),
        "-v".to_string(),
        "/src/".to_string(),
        "/dst2/".to_string(),
    ];
    client.execute(&args1).unwrap();
    client.execute(&args2).unwrap();

    let last = client.last_command().unwrap();
    assert_eq!(last.args, args2);
}

#[test]
fn test_forced_error_propagates() {
    let (_, client) = setup_test_env();

    client.set_force_error(Some(RsyncError::ProcessError {
        message: "simulated failure".to_string(),
        exit_code: Some(1),
    }));

    let args = vec!["-a".to_string(), "/src/".to_string(), "/dst/".to_string()];
    let result = client.execute(&args);
    assert!(result.is_err());

    match result.unwrap_err() {
        RsyncError::ProcessError { message, .. } => {
            assert_eq!(message, "simulated failure");
        }
        other => panic!("Expected ProcessError, got {:?}", other),
    }
}

#[test]
fn test_forced_error_resets_after_use() {
    let (_, client) = setup_test_env();

    client.set_force_error(Some(RsyncError::Cancelled));

    let args = vec!["-a".to_string(), "/src/".to_string(), "/dst/".to_string()];

    // First call should fail
    assert!(client.execute(&args).is_err());

    // Second call should succeed
    assert!(client.execute(&args).is_ok());
}

#[test]
fn test_backup_dir_moves_replaced_files() {
    let fs = Rc::new(
        TestFileSystem::new()
            .with_file("/src/file.txt", "new version")
            .with_file("/dst/file.txt", "old version")
            .with_dir("/archive"),
    );
    let client = TestRsyncClient::new(fs.clone());

    let args = vec![
        "-a".to_string(),
        "--backup".to_string(),
        "--backup-dir=/archive".to_string(),
        "/src/".to_string(),
        "/dst/".to_string(),
    ];
    client.execute(&args).unwrap();

    assert_eq!(fs.file_content("/dst/file.txt").unwrap(), "new version");
    assert_eq!(
        fs.file_content("/archive/file.txt").unwrap(),
        "old version"
    );
}

#[test]
fn test_version_returns_string() {
    let (_, client) = setup_test_env();
    let version = client.version().unwrap();
    assert!(!version.is_empty());
    assert!(version.contains("rsync"));
}

#[test]
fn test_build_command_args_mirror_mode() {
    let source = StorageLocation::Local {
        path: "/src/".to_string(),
    };
    let dest = StorageLocation::Local {
        path: "/dst/".to_string(),
    };
    let options = RsyncOptions {
        archive: true,
        delete: true,
        ..RsyncOptions::default()
    };
    let args = command_builder::build_rsync_args(&source, &dest, &options, None, None);

    assert!(args.contains(&"-a".to_string()));
    assert!(args.contains(&"--delete".to_string()));
    assert!(args.contains(&"/src/".to_string()));
    assert!(args.contains(&"/dst/".to_string()));
}

#[test]
fn test_build_command_args_versioned_mode() {
    let source = StorageLocation::Local {
        path: "/src/".to_string(),
    };
    let dest = StorageLocation::Local {
        path: "/dst/".to_string(),
    };
    let options = RsyncOptions {
        archive: true,
        ..RsyncOptions::default()
    };
    let args = command_builder::build_rsync_args(&source, &dest, &options, None, None);

    assert!(args.contains(&"-a".to_string()));
    assert!(args.contains(&"/src/".to_string()));
    assert!(args.contains(&"/dst/".to_string()));
}

#[test]
fn test_build_command_args_snapshot_mode() {
    let source = StorageLocation::Local {
        path: "/src/".to_string(),
    };
    let dest = StorageLocation::Local {
        path: "/dst/current/".to_string(),
    };
    let options = RsyncOptions::default();
    let args =
        command_builder::build_rsync_args(&source, &dest, &options, None, Some("/dst/prev/"));

    assert!(args.contains(&"--link-dest=/dst/prev/".to_string()));
}

#[test]
fn test_build_command_args_ssh_config() {
    let source = StorageLocation::RemoteSsh {
        user: "admin".to_string(),
        host: "server.com".to_string(),
        port: 2222,
        path: "/data/".to_string(),
        identity_file: Some("/home/user/.ssh/id_rsa".to_string()),
    };
    let dest = StorageLocation::Local {
        path: "/backup/".to_string(),
    };
    let ssh_config = SshConfig {
        port: 2222,
        identity_file: Some("/home/user/.ssh/id_rsa".to_string()),
        strict_host_key_checking: true,
        custom_ssh_command: None,
    };
    let options = RsyncOptions::default();
    let args = command_builder::build_rsync_args(&source, &dest, &options, Some(&ssh_config), None);

    assert!(args.contains(&"-e".to_string()));
    let ssh_arg = args
        .iter()
        .find(|a| a.starts_with("ssh"))
        .expect("should have ssh command arg");
    assert!(ssh_arg.contains("-p 2222"));
    assert!(ssh_arg.contains("-i /home/user/.ssh/id_rsa"));
    assert!(args.contains(&"admin@server.com:/data/".to_string()));
}

#[test]
fn test_build_command_args_excludes() {
    let source = StorageLocation::Local {
        path: "/src/".to_string(),
    };
    let dest = StorageLocation::Local {
        path: "/dst/".to_string(),
    };
    let options = RsyncOptions {
        exclude_patterns: vec!["*.log".to_string(), "tmp/".to_string()],
        ..RsyncOptions::default()
    };
    let args = command_builder::build_rsync_args(&source, &dest, &options, None, None);

    assert!(args.contains(&"--exclude=*.log".to_string()));
    assert!(args.contains(&"--exclude=tmp/".to_string()));
}
