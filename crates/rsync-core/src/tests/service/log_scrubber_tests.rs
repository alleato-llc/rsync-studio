#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    use crate::services::log_scrubber::{scrub_apply, scrub_scan};

    fn setup_log_dir(files: &[(&str, &str)]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for (name, content) in files {
            fs::write(dir.path().join(name), content).unwrap();
        }
        dir
    }

    #[test]
    fn scan_finds_matches_in_log_files() {
        let dir = setup_log_dir(&[
            ("a.log", "password=secret123\nall good\npassword=secret123 again"),
            ("b.log", "nothing here"),
            ("c.log", "also has secret123"),
        ]);

        let results = scrub_scan(dir.path().to_str().unwrap(), "secret123").unwrap();
        assert_eq!(results.len(), 2);

        let a = results.iter().find(|r| r.file_path.contains("a.log")).unwrap();
        assert_eq!(a.match_count, 2);

        let c = results.iter().find(|r| r.file_path.contains("c.log")).unwrap();
        assert_eq!(c.match_count, 1);
    }

    #[test]
    fn scan_empty_pattern_returns_error() {
        let dir = setup_log_dir(&[("a.log", "content")]);
        let result = scrub_scan(dir.path().to_str().unwrap(), "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn scan_nonexistent_dir_returns_error() {
        let result = scrub_scan("/nonexistent/path/1234567890", "pattern");
        assert!(result.is_err());
    }

    #[test]
    fn scan_ignores_non_log_files() {
        let dir = setup_log_dir(&[
            ("a.log", "secret"),
            ("b.txt", "secret"),
            ("c.json", "secret"),
        ]);

        let results = scrub_scan(dir.path().to_str().unwrap(), "secret").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].file_path.contains("a.log"));
    }

    #[test]
    fn scan_no_matches_returns_empty() {
        let dir = setup_log_dir(&[("a.log", "nothing interesting")]);
        let results = scrub_scan(dir.path().to_str().unwrap(), "secret").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn apply_replaces_with_asterisks() {
        let dir = setup_log_dir(&[
            ("a.log", "user=admin password=secret123 done"),
        ]);
        let file_path = dir.path().join("a.log").to_string_lossy().to_string();

        let results = scrub_apply("secret123", &[file_path.clone()]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].replacements, 1);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "user=admin password=********* done");
        assert!(!content.contains("secret123"));
    }

    #[test]
    fn apply_skips_missing_files() {
        let results = scrub_apply("pattern", &["/nonexistent/file.log".to_string()]).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn apply_empty_pattern_returns_error() {
        let result = scrub_apply("", &["file.log".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must not be empty"));
    }

    #[test]
    fn apply_multiple_occurrences() {
        let dir = setup_log_dir(&[
            ("a.log", "key=abc key=abc key=abc"),
        ]);
        let file_path = dir.path().join("a.log").to_string_lossy().to_string();

        let results = scrub_apply("abc", &[file_path.clone()]).unwrap();
        assert_eq!(results[0].replacements, 3);

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "key=*** key=*** key=***");
    }
}
