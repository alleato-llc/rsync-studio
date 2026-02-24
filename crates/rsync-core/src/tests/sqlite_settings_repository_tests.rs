use crate::database::sqlite::Database;
use crate::repository::settings::SettingsRepository;
use crate::repository::sqlite::settings::SqliteSettingsRepository;

fn setup() -> SqliteSettingsRepository {
    let db = Database::in_memory().unwrap();
    SqliteSettingsRepository::new(db.conn())
}

#[test]
fn test_get_nonexistent_setting() {
    let repo = setup();
    let result = repo.get_setting("nonexistent").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_set_and_get_setting() {
    let repo = setup();
    repo.set_setting("key1", "value1").unwrap();
    let result = repo.get_setting("key1").unwrap();
    assert_eq!(result, Some("value1".to_string()));
}

#[test]
fn test_update_setting() {
    let repo = setup();
    repo.set_setting("key1", "value1").unwrap();
    repo.set_setting("key1", "value2").unwrap();
    let result = repo.get_setting("key1").unwrap();
    assert_eq!(result, Some("value2".to_string()));
}

#[test]
fn test_delete_setting() {
    let repo = setup();
    repo.set_setting("key1", "value1").unwrap();
    repo.delete_setting("key1").unwrap();
    let result = repo.get_setting("key1").unwrap();
    assert_eq!(result, None);
}

#[test]
fn test_delete_nonexistent_setting() {
    let repo = setup();
    // Should not error
    repo.delete_setting("nonexistent").unwrap();
}

#[test]
fn test_multiple_settings() {
    let repo = setup();
    repo.set_setting("a", "1").unwrap();
    repo.set_setting("b", "2").unwrap();
    repo.set_setting("c", "3").unwrap();

    assert_eq!(repo.get_setting("a").unwrap(), Some("1".to_string()));
    assert_eq!(repo.get_setting("b").unwrap(), Some("2".to_string()));
    assert_eq!(repo.get_setting("c").unwrap(), Some("3".to_string()));
}
