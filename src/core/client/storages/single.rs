use wlist_native::common::data::storages::information::{StorageInformation, StorageListInformation};
use wlist_native::common::data::storages::options::{ListStorageOptions, StoragesFilter};
use wlist_native::core::client::storages::{storages_get, storages_list, storages_rename, storages_set_readonly};

use crate::core::c;

async fn test_list(guard: &super::InitializeGuard, filter: StoragesFilter, not_filtered: bool, info: &StorageInformation) -> anyhow::Result<()> {
    let list = storages_list(c!(guard), ListStorageOptions {
        filter, orders: Default::default(),
        offset: 0,
        limit: 2,
    }).await?;
    assert_eq!(list.total, 1);
    if not_filtered {
        assert_eq!(list.filtered, 1);
        assert_eq!(list.storages.len(), 1);
        assert_eq!(&list.storages[0], info);
    } else {
        assert_eq!(list.filtered, 0);
        assert_eq!(list.storages.len(), 0);
    }
    Ok(())
}

pub async fn list(guard: &super::InitializeGuard, info: &StorageInformation) -> anyhow::Result<()> {
    test_list(guard, StoragesFilter::Readonly, info.storage_type.is_share(), info).await?;
    test_list(guard, StoragesFilter::Writable, !info.storage_type.is_share(), info).await?;
    test_list(guard, StoragesFilter::Shared, info.storage_type.is_share(), info).await?;
    test_list(guard, StoragesFilter::Private, !info.storage_type.is_share(), info).await?;
    test_list(guard, StoragesFilter::ReadonlyPrivate, false, info).await?;
    test_list(guard, StoragesFilter::Owned, true, info).await?;
    test_list(guard, StoragesFilter::All, true, info).await?;

    // test_offset
    assert_eq!(
        storages_list(c!(guard), ListStorageOptions {
            filter: StoragesFilter::All,
            orders: Default::default(),
            offset: 1,
            limit: 1,
        }).await?,
        StorageListInformation {
            total: 1,
            filtered: 1,
            storages: vec![],
        }
    );
    // test_limit
    assert_eq!(
        storages_list(c!(guard), ListStorageOptions {
            filter: StoragesFilter::All,
            orders: Default::default(),
            offset: 0,
            limit: 0,
        }).await?,
        StorageListInformation {
            total: 1,
            filtered: 1,
            storages: vec![],
        }
    );
    Ok(())
}

pub async fn get(guard: &super::InitializeGuard, info: &StorageInformation) -> anyhow::Result<()> {
    let detail = storages_get(c!(guard), info.id, false).await?;
    assert_eq!(&detail.basic, info);
    assert_eq!(detail.indexed_size, 0); // Newly created storage has no indexed data
    tracing::debug!(?info, ?detail, "Got storage detail.");

    let checked = storages_get(c!(guard), info.id, true).await?;
    assert_eq!(checked, detail); // Nothing changed after checked.

    let result = storages_get(c!(guard), 0, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_get(c!(guard), 0, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}

pub async fn rename(guard: &super::InitializeGuard, info: &StorageInformation) -> anyhow::Result<StorageInformation> {
    // test_rename_invalid
    for new_name in super::INVALID_STORAGE_NAME.iter() {
        let result = storages_rename(c!(guard), info.id, new_name.clone()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        let result = storages_rename(c!(guard), 0, new_name.clone()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }

    // test_rename_valid
    let name = info.name.to_string();
    let mut info = info.clone();
    for new_name in super::VALID_STORAGE_NAME.iter().chain(std::iter::once(&name)) {
        storages_rename(c!(guard), info.id, new_name.clone()).await?;
        let detail = storages_get(c!(guard), info.id, false).await?;
        assert_eq!(detail.basic.name.as_str(), new_name);
        assert_eq!(detail.basic.id, info.id);
        assert_eq!(detail.basic.read_only, info.read_only);
        assert_eq!(detail.basic.storage_type, info.storage_type);
        assert_eq!(detail.basic.available, info.available);
        assert_eq!(detail.basic.create_time, info.create_time);
        assert_ne!(detail.basic.update_time, info.update_time);
        assert_eq!(detail.basic.root_directory_id, info.root_directory_id);
        info = detail.basic;
    }

    // test_rename_duplicate
    storages_rename(c!(guard), info.id, name.clone()).await?;
    let detail = storages_get(c!(guard), info.id, false).await?;
    assert_eq!(detail.basic.name.as_str(), name);
    assert_eq!(detail.basic.id, info.id);
    assert_eq!(detail.basic.read_only, info.read_only);
    assert_eq!(detail.basic.storage_type, info.storage_type);
    assert_eq!(detail.basic.available, info.available);
    assert_eq!(detail.basic.create_time, info.create_time);
    assert_eq!(detail.basic.update_time, info.update_time);
    assert_eq!(detail.basic.root_directory_id, info.root_directory_id);
    Ok(info)
}

pub async fn set_readonly(guard: &super::InitializeGuard, info: &StorageInformation) -> anyhow::Result<StorageInformation> {
    let info = if info.storage_type.is_share() {
        // test_set_readonly_invalid
        let result = storages_set_readonly(c!(guard), info.id, false).await;
        crate::assert_error::<_, wlist_native::common::exceptions::StorageTypeMismatchedError>(result)?;

        // test_set_readonly_twice
        storages_set_readonly(c!(guard), info.id, true).await?;
        let detail = storages_get(c!(guard), info.id, false).await?;
        assert_eq!(&detail.basic, info);
        detail.basic
    } else {
        // test_set_readonly_twice
        storages_set_readonly(c!(guard), info.id, false).await?;
        let detail = storages_get(c!(guard), info.id, false).await?;
        assert_eq!(&detail.basic, info);

        storages_set_readonly(c!(guard), info.id, true).await?;
        let detail_readonly = storages_get(c!(guard), info.id, false).await?;
        test_list(guard, StoragesFilter::Readonly, true, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::Writable, false, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::Shared, false, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::Private, true, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::ReadonlyPrivate, true, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::Owned, false, &detail_readonly.basic).await?;
        test_list(guard, StoragesFilter::All, true, &detail_readonly.basic).await?;
        assert_eq!(detail_readonly.basic.name, info.name);
        assert_eq!(detail_readonly.basic.id, info.id);
        assert_ne!(detail_readonly.basic.read_only, info.read_only);
        assert_eq!(detail_readonly.basic.storage_type, info.storage_type);
        assert_eq!(detail_readonly.basic.available, info.available);
        assert_eq!(detail_readonly.basic.create_time, info.create_time);
        assert_ne!(detail_readonly.basic.update_time, info.update_time);
        assert_eq!(detail_readonly.basic.root_directory_id, info.root_directory_id);
        assert_eq!(detail_readonly.size, detail.size);
        assert_eq!(detail_readonly.indexed_size, detail.indexed_size);
        assert_eq!(detail_readonly.total_size, detail.total_size);
        assert_eq!(detail_readonly.upload_flow, detail.upload_flow);
        assert_eq!(detail_readonly.download_flow, detail.download_flow);
        assert_eq!(detail_readonly.max_size_per_file, detail.max_size_per_file);

        storages_set_readonly(c!(guard), info.id, false).await?;
        let detail_revert = storages_get(c!(guard), info.id, false).await?;
        test_list(guard, StoragesFilter::Readonly, false, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::Writable, true, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::Shared, false, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::Private, true, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::ReadonlyPrivate, false, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::Owned, true, &detail_revert.basic).await?;
        test_list(guard, StoragesFilter::All, true, &detail_revert.basic).await?;
        assert_eq!(detail_revert.basic.name, info.name);
        assert_eq!(detail_revert.basic.id, info.id);
        assert_eq!(detail_revert.basic.read_only, info.read_only);
        assert_eq!(detail_revert.basic.storage_type, info.storage_type);
        assert_eq!(detail_revert.basic.available, info.available);
        assert_eq!(detail_revert.basic.create_time, info.create_time);
        assert_ne!(detail_revert.basic.update_time, detail_readonly.basic.update_time);
        assert_eq!(detail_revert.basic.root_directory_id, info.root_directory_id);
        assert_eq!(detail_revert.size, detail.size);
        assert_eq!(detail_revert.indexed_size, detail.indexed_size);
        assert_eq!(detail_revert.total_size, detail.total_size);
        assert_eq!(detail_revert.upload_flow, detail.upload_flow);
        assert_eq!(detail_revert.download_flow, detail.download_flow);
        assert_eq!(detail_revert.max_size_per_file, detail.max_size_per_file);

        detail_revert.basic
    };

    let result = storages_set_readonly(c!(guard), 0, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = storages_set_readonly(c!(guard), 0, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(info)
}
