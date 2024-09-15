use std::string::ToString;
use std::sync::LazyLock;

use wlist_native::common::data::storages::information::StorageInformation;

use super::super::InitializeGuard;

mod none;
mod single;

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
    // normal name
    "storage_name_test".to_string(),
]);

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    tokio::try_join!(
        none::list(guard),
        none::get(guard),
        none::remove(guard),
        none::rename(guard),
        none::set_readonly(guard),
    )?;
    Ok(())
}

pub async fn test_single(guard: &InitializeGuard, info: &StorageInformation) -> anyhow::Result<StorageInformation> {
    tokio::try_join!(
        single::list(guard, info),
        single::get(guard, info),
    )?;
    let info = single::rename(guard, info).await?;
    let info = single::set_readonly(guard, &info).await?;
    Ok(info)
}
