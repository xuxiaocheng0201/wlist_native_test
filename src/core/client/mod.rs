use std::string::ToString;
use std::sync::LazyLock;

mod storages;

static INVALID_STORAGE_NAME: LazyLock<Vec<String>> = LazyLock::new(|| vec![
    // empty storage name
    "".to_string(),
    // too long storage name
    "a".repeat(32768),
]);

static VALID_STORAGE_NAME: LazyLock<Vec<String>> = LazyLock::new(|| vec![
    // min length storage name
    "1".to_string(),
    // max length storage name
    "a".repeat(32767),
]);
