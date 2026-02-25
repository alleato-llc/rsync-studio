use crate::models::itemize::{DifferenceKind, FileType, ItemizedChange, TransferType};

/// Parse a single rsync `--itemize-changes` output line into an `ItemizedChange`.
///
/// Returns `None` for lines that don't match the itemize format (e.g. progress lines,
/// summary lines, error messages).
///
/// The itemize format is a code of 11 characters (rsync <3.2) or 12 characters
/// (rsync 3.2+) followed by a space and the file path:
///   rsync <3.2:  `YXcstpoguax path/to/file`   (11 chars)
///   rsync 3.2+:  `YXcstpoguaxn path/to/file`  (12 chars)
///
/// Lines starting with `*` are message lines (e.g., `*deleting   path/to/file`).
pub fn parse_itemize_line(line: &str) -> Option<ItemizedChange> {
    if line.is_empty() {
        return None;
    }

    // Handle message lines like "*deleting   path/to/file"
    if line.starts_with('*') {
        let rest = line[1..].trim_start();
        // Find the path after the message word(s) and whitespace
        // e.g. "deleting   path/to/file" → skip "deleting" then whitespace
        let path = rest
            .find(|c: char| c.is_whitespace())
            .and_then(|i| {
                let after_word = &rest[i..];
                let trimmed = after_word.trim_start();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })?;

        return Some(ItemizedChange {
            transfer_type: TransferType::Message,
            file_type: FileType::File, // default for message lines
            differences: vec![],
            path,
        });
    }

    // Minimum length: 11 chars code + 1 space + 1 char path = 13
    if line.len() < 13 {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();

    // Position 0: transfer type
    let transfer_type = match chars[0] {
        '>' => TransferType::Sent,
        '<' => TransferType::Received,
        'c' => TransferType::LocalChange,
        '.' => TransferType::NoUpdate,
        _ => return None,
    };

    // Position 1: file type
    let file_type = match chars[1] {
        'f' => FileType::File,
        'd' => FileType::Directory,
        'L' => FileType::Symlink,
        'D' => FileType::Device,
        'S' => FileType::Special,
        _ => return None,
    };

    // Detect code length: rsync <3.2 uses 11-char codes, rsync 3.2+ uses 12-char codes.
    // Find the space separator after the flags.
    let code_len = if chars.len() > 12 && chars[12] == ' ' {
        12
    } else if chars.len() > 11 && chars[11] == ' ' {
        11
    } else {
        return None;
    };

    let flag_count = code_len - 2; // number of flag positions after YX

    // Positions 2..code_len: difference flags
    let flag_chars = &chars[2..code_len];

    // Check if all flag positions are '+' (newly created)
    let all_plus = flag_chars.iter().all(|&c| c == '+');

    let differences = if all_plus {
        vec![DifferenceKind::NewlyCreated]
    } else {
        let mut diffs = Vec::new();
        // Position 2 (flag index 0): checksum
        if flag_chars[0] == 'c' {
            diffs.push(DifferenceKind::Checksum);
        }
        // Position 3 (flag index 1): size
        if flag_chars[1] == 's' {
            diffs.push(DifferenceKind::Size);
        }
        // Position 4 (flag index 2): timestamp
        if flag_chars[2] == 't' {
            diffs.push(DifferenceKind::Timestamp);
        }
        // Position 5 (flag index 3): permissions
        if flag_chars[3] == 'p' {
            diffs.push(DifferenceKind::Permissions);
        }
        // Position 6 (flag index 4): owner
        if flag_chars[4] == 'o' {
            diffs.push(DifferenceKind::Owner);
        }
        // Position 7 (flag index 5): group
        if flag_chars[5] == 'g' {
            diffs.push(DifferenceKind::Group);
        }
        // Position 8 (flag index 6): unused (u in the format string, skip)
        // Position 9 (flag index 7): acl
        if flag_chars[7] == 'a' {
            diffs.push(DifferenceKind::Acl);
        }
        // Position 10 (flag index 8): extended attributes
        if flag_count > 8 && flag_chars[8] == 'x' {
            diffs.push(DifferenceKind::ExtendedAttributes);
        }
        // Position 11 (flag index 9): rsync 3.2+ additional field (skip)
        diffs
    };

    // Path is everything after the code + space
    let path: String = chars[code_len + 1..].iter().collect();

    Some(ItemizedChange {
        transfer_type,
        file_type,
        differences,
        path,
    })
}
