use wlist_native::common::data::storages::information::StorageListInformation;
use wlist_native::common::data::storages::options::{ListStorageOptions, StoragesFilter};
use wlist_native::core::client::storages::{storages_get, storages_list, storages_remove, storages_rename, storages_set_readonly};

use crate::core::c;
use crate::core::client::storages::{INVALID_STORAGE_NAME, VALID_STORAGE_NAME};

pub async fn list(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    assert_eq!(
        storages_list(c!(guard), ListStorageOptions {
            filter: StoragesFilter::All,
            orders: Default::default(),
            offset: 0,
            limit: 1,
        }).await?,
        StorageListInformation {
            total: 0,
            filtered: 0,
            storages: vec![],
        }
    );
    Ok(())
}

pub async fn get(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    let result = storages_get(c!(guard), 0, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_get(c!(guard), 0, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_get(c!(guard), 1, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_get(c!(guard), 1, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}

pub async fn remove(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    let result = storages_remove(c!(guard), 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_remove(c!(guard), 1).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}

pub async fn rename(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    for name in INVALID_STORAGE_NAME.iter() {
        let result = storages_rename(c!(guard), 0, name.to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        let result = storages_rename(c!(guard), 1, name.to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }
    for name in VALID_STORAGE_NAME.iter().map(String::as_str).chain(std::iter::once("storage")) {
        let result = storages_rename(c!(guard), 0, name.to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
        let result = storages_rename(c!(guard), 1, name.to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    }
    Ok(())
}

pub async fn set_readonly(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    let result = storages_set_readonly(c!(guard), 0, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_set_readonly(c!(guard), 0, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_set_readonly(c!(guard), 1, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_set_readonly(c!(guard), 1, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}
