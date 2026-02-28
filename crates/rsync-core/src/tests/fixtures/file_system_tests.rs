use std::path::{Path, PathBuf};

use crate::tests::test_file_system::TestFileSystem;
use crate::file_system::FileSystem;

#[test]
fn test_create_and_read_file() {
    let fs = TestFileSystem::new().with_file("/data/hello.txt", "Hello, world!");

    assert!(fs.exists(Path::new("/data/hello.txt")));
    assert!(fs.is_file(Path::new("/data/hello.txt")));
    assert_eq!(
        fs.read_to_string(Path::new("/data/hello.txt")).unwrap(),
        "Hello, world!"
    );
}

#[test]
fn test_create_dir_all() {
    let fs = TestFileSystem::new();
    fs.create_dir_all(Path::new("/a/b/c/d")).unwrap();

    assert!(fs.is_dir(Path::new("/a")));
    assert!(fs.is_dir(Path::new("/a/b")));
    assert!(fs.is_dir(Path::new("/a/b/c")));
    assert!(fs.is_dir(Path::new("/a/b/c/d")));
}

#[test]
fn test_hard_link_shares_inode() {
    let fs = TestFileSystem::new().with_file("/src/file.txt", "content");
    fs.hard_link(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
        .unwrap();

    assert!(fs.are_hard_linked("/src/file.txt", "/dst/file.txt"));
}

#[test]
fn test_hard_link_independence() {
    let fs = TestFileSystem::new().with_file("/src/file.txt", "original");
    fs.hard_link(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
        .unwrap();

    // Both should have the same content
    assert_eq!(fs.file_content("/src/file.txt").unwrap(), "original");
    assert_eq!(fs.file_content("/dst/file.txt").unwrap(), "original");

    // Hard links share inode
    assert!(fs.are_hard_linked("/src/file.txt", "/dst/file.txt"));
}

#[test]
fn test_symlink_create_and_read() {
    let fs = TestFileSystem::new().with_file("/data/target.txt", "content");
    fs.create_symlink(Path::new("/data/target.txt"), Path::new("/data/link.txt"))
        .unwrap();

    assert!(fs.is_symlink(Path::new("/data/link.txt")));
    assert_eq!(
        fs.read_link(Path::new("/data/link.txt")).unwrap(),
        PathBuf::from("/data/target.txt")
    );
    assert_eq!(
        fs.symlink_target("/data/link.txt").unwrap(),
        PathBuf::from("/data/target.txt")
    );
}

#[test]
fn test_remove_symlink() {
    let fs = TestFileSystem::new().with_file("/data/target.txt", "content");
    fs.create_symlink(Path::new("/data/target.txt"), Path::new("/data/link.txt"))
        .unwrap();

    assert!(fs.is_symlink(Path::new("/data/link.txt")));
    fs.remove_symlink(Path::new("/data/link.txt")).unwrap();
    assert!(!fs.exists(Path::new("/data/link.txt")));
}

#[test]
fn test_walk_dir() {
    let fs = TestFileSystem::new()
        .with_file("/project/src/main.rs", "fn main() {}")
        .with_file("/project/src/lib.rs", "pub mod foo;")
        .with_file("/project/Cargo.toml", "[package]");

    let entries = fs.walk_dir(Path::new("/project")).unwrap();
    assert!(entries.contains(&PathBuf::from("/project/src")));
    assert!(entries.contains(&PathBuf::from("/project/src/main.rs")));
    assert!(entries.contains(&PathBuf::from("/project/src/lib.rs")));
    assert!(entries.contains(&PathBuf::from("/project/Cargo.toml")));
}

#[test]
fn test_walk_dir_excludes_other_dirs() {
    let fs = TestFileSystem::new()
        .with_file("/a/file1.txt", "a")
        .with_file("/b/file2.txt", "b");

    let entries = fs.walk_dir(Path::new("/a")).unwrap();
    assert!(entries.contains(&PathBuf::from("/a/file1.txt")));
    assert!(!entries.contains(&PathBuf::from("/b/file2.txt")));
}

#[test]
fn test_copy_file_creates_independent_copy() {
    let fs = TestFileSystem::new().with_file("/src/file.txt", "data");
    fs.copy_file(Path::new("/src/file.txt"), Path::new("/dst/file.txt"))
        .unwrap();

    assert_eq!(fs.file_content("/dst/file.txt").unwrap(), "data");
    assert!(!fs.are_hard_linked("/src/file.txt", "/dst/file.txt"));
}

#[test]
fn test_available_space() {
    let fs = TestFileSystem::new().with_available_space(1024 * 1024);
    assert_eq!(
        fs.available_space(Path::new("/")).unwrap(),
        1024 * 1024
    );
}

#[test]
fn test_file_not_found() {
    let fs = TestFileSystem::new();
    let result = fs.read_to_string(Path::new("/nonexistent.txt"));
    assert!(result.is_err());
}

#[test]
fn test_write_and_read() {
    let fs = TestFileSystem::new();
    fs.write(Path::new("/data/test.txt"), "written content")
        .unwrap();

    assert_eq!(
        fs.read_to_string(Path::new("/data/test.txt")).unwrap(),
        "written content"
    );
}

#[test]
fn test_remove_dir_all() {
    let fs = TestFileSystem::new()
        .with_file("/dir/a.txt", "a")
        .with_file("/dir/sub/b.txt", "b")
        .with_dir("/dir/empty");

    fs.remove_dir_all(Path::new("/dir")).unwrap();

    assert!(!fs.exists(Path::new("/dir")));
    assert!(!fs.exists(Path::new("/dir/a.txt")));
    assert!(!fs.exists(Path::new("/dir/sub/b.txt")));
    assert!(!fs.exists(Path::new("/dir/empty")));
}

#[test]
fn test_auto_create_parents() {
    let fs = TestFileSystem::new().with_file("/a/b/c.txt", "content");

    assert!(fs.is_dir(Path::new("/a")));
    assert!(fs.is_dir(Path::new("/a/b")));
    assert!(fs.is_file(Path::new("/a/b/c.txt")));
}
