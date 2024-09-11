use std::sync::LazyLock;
use super::super::InitializeGuard;

mod empty;

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

pub async fn test_empty(guard: &InitializeGuard) -> anyhow::Result<()> {
    tokio::try_join!(
        empty::list(guard),
        empty::get(guard),
        empty::remove(guard),
        empty::rename(guard),
        empty::set_readonly(guard),
    )?;
    Ok(())
}
