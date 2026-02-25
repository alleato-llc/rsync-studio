use crate::models::itemize::{DifferenceKind, FileType, ItemizedChange, TransferType};
use crate::services::itemize_parser::parse_itemize_line;

// --- Transfer types (12-char format, rsync 3.2+) ---

#[test]
fn parse_sent() {
    let result = parse_itemize_line(">f.st....... file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::Sent);
}

#[test]
fn parse_received() {
    let result = parse_itemize_line("<f.st....... file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::Received);
}

#[test]
fn parse_local_change() {
    let result = parse_itemize_line("cf.st....... file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::LocalChange);
}

#[test]
fn parse_no_update() {
    let result = parse_itemize_line(".f.st....... file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::NoUpdate);
}

#[test]
fn parse_message_deleting() {
    let result = parse_itemize_line("*deleting   file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::Message);
    assert_eq!(result.path, "file.txt");
    assert!(result.differences.is_empty());
}

// --- File types ---

#[test]
fn parse_file_type_file() {
    let result = parse_itemize_line(">f.......... file.txt").unwrap();
    assert_eq!(result.file_type, FileType::File);
}

#[test]
fn parse_file_type_directory() {
    let result = parse_itemize_line(">d.......... dir/").unwrap();
    assert_eq!(result.file_type, FileType::Directory);
    assert_eq!(result.path, "dir/");
}

#[test]
fn parse_file_type_symlink() {
    let result = parse_itemize_line(">L.......... link").unwrap();
    assert_eq!(result.file_type, FileType::Symlink);
}

#[test]
fn parse_file_type_device() {
    let result = parse_itemize_line(">D.......... dev").unwrap();
    assert_eq!(result.file_type, FileType::Device);
}

#[test]
fn parse_file_type_special() {
    let result = parse_itemize_line(">S.......... special").unwrap();
    assert_eq!(result.file_type, FileType::Special);
}

// --- Individual differences (12-char format) ---

#[test]
fn parse_size_difference() {
    let result = parse_itemize_line(">f.s........ file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Size]);
}

#[test]
fn parse_timestamp_difference() {
    let result = parse_itemize_line(">f..t....... file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Timestamp]);
}

#[test]
fn parse_permissions_difference() {
    let result = parse_itemize_line(">f...p...... file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Permissions]);
}

#[test]
fn parse_owner_difference() {
    let result = parse_itemize_line(">f....o..... file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Owner]);
}

#[test]
fn parse_group_difference() {
    let result = parse_itemize_line(">f.....g.... file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Group]);
}

#[test]
fn parse_checksum_difference() {
    let result = parse_itemize_line(">fc......... file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Checksum]);
}

#[test]
fn parse_acl_difference() {
    let result = parse_itemize_line(">f.......a.. file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::Acl]);
}

#[test]
fn parse_extended_attributes_difference() {
    let result = parse_itemize_line(">f........x. file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::ExtendedAttributes]);
}

// --- Combinations ---

#[test]
fn parse_checksum_size_timestamp() {
    let result = parse_itemize_line(">fcst....... file.txt").unwrap();
    assert_eq!(
        result.differences,
        vec![
            DifferenceKind::Checksum,
            DifferenceKind::Size,
            DifferenceKind::Timestamp,
        ]
    );
}

#[test]
fn parse_multiple_differences() {
    let result = parse_itemize_line(">f.stpog.ax. file.txt").unwrap();
    assert_eq!(
        result.differences,
        vec![
            DifferenceKind::Size,
            DifferenceKind::Timestamp,
            DifferenceKind::Permissions,
            DifferenceKind::Owner,
            DifferenceKind::Group,
            DifferenceKind::Acl,
            DifferenceKind::ExtendedAttributes,
        ]
    );
}

// --- Newly created (12-char format) ---

#[test]
fn parse_newly_created() {
    let result = parse_itemize_line(">f++++++++++ new-file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::NewlyCreated]);
    assert_eq!(result.path, "new-file.txt");
}

// --- No changes ---

#[test]
fn parse_no_changes() {
    let result = parse_itemize_line(".f.......... file.txt").unwrap();
    assert!(result.differences.is_empty());
}

// --- Non-itemize lines (return None) ---

#[test]
fn parse_sending_incremental_file_list() {
    assert!(parse_itemize_line("sending incremental file list").is_none());
}

#[test]
fn parse_progress_line() {
    assert!(
        parse_itemize_line("     32,768 100%   31.25kB/s    0:00:01 (xfr#1, to-chk=0/1)")
            .is_none()
    );
}

#[test]
fn parse_empty_string() {
    assert!(parse_itemize_line("").is_none());
}

#[test]
fn parse_rsync_error() {
    assert!(parse_itemize_line("rsync error: some error occurred").is_none());
}

// --- Paths with spaces ---

#[test]
fn parse_path_with_spaces() {
    let result = parse_itemize_line(">f.st....... path with spaces/file.txt").unwrap();
    assert_eq!(result.path, "path with spaces/file.txt");
    assert_eq!(result.transfer_type, TransferType::Sent);
    assert_eq!(
        result.differences,
        vec![DifferenceKind::Size, DifferenceKind::Timestamp]
    );
}

// --- Message with path containing spaces ---

#[test]
fn parse_message_path_with_spaces() {
    let result = parse_itemize_line("*deleting   path with spaces/old file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::Message);
    assert_eq!(result.path, "path with spaces/old file.txt");
}

// --- Full ItemizedChange structure ---

#[test]
fn parse_full_structure() {
    let result = parse_itemize_line(">fcst....... docs/readme.md").unwrap();
    assert_eq!(
        result,
        ItemizedChange {
            transfer_type: TransferType::Sent,
            file_type: FileType::File,
            differences: vec![
                DifferenceKind::Checksum,
                DifferenceKind::Size,
                DifferenceKind::Timestamp,
            ],
            path: "docs/readme.md".to_string(),
        }
    );
}

// --- 11-char format (rsync <3.2) compatibility ---

#[test]
fn parse_11_char_sent() {
    let result = parse_itemize_line(">f.st...... file.txt").unwrap();
    assert_eq!(result.transfer_type, TransferType::Sent);
    assert_eq!(
        result.differences,
        vec![DifferenceKind::Size, DifferenceKind::Timestamp]
    );
    assert_eq!(result.path, "file.txt");
}

#[test]
fn parse_11_char_newly_created() {
    let result = parse_itemize_line(">f+++++++++ new-file.txt").unwrap();
    assert_eq!(result.differences, vec![DifferenceKind::NewlyCreated]);
    assert_eq!(result.path, "new-file.txt");
}

#[test]
fn parse_11_char_no_changes() {
    let result = parse_itemize_line(".f......... file.txt").unwrap();
    assert!(result.differences.is_empty());
}

#[test]
fn parse_11_char_acl_and_xattr() {
    let result = parse_itemize_line(">f.......ax file.txt").unwrap();
    assert_eq!(
        result.differences,
        vec![DifferenceKind::Acl, DifferenceKind::ExtendedAttributes]
    );
}
