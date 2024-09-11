use wlist_native::common::data::storages::StorageType;

mod storages;

macro_rules! add_storage {
    ($f: ident($g: ident, $n: expr, $c: literal)) => {
        wlist_native::core::client::storages::$f(
            super::c!($g), $n.to_string(), toml::from_str(include_str!($c))?
        ).await
    };
}

async fn test_empty(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    tokio::try_join!(
        storages::test_empty(&guard),
        // TODO
    )?;
    Ok(())
}

async fn test_wrong(guard: &super::InitializeGuard, storage: StorageType) -> anyhow::Result<()> {
    let name = "storage-wrong";
    let result = match storage {
        StorageType::Lanzou => add_storage!(storages_lanzou_add(guard, name, "accounts/lanzou_wrong.toml")),
    };
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectStorageAccountError>(result)?;
    Ok(())
}

async fn test_normal(guard: &super::InitializeGuard, storage: StorageType) -> anyhow::Result<()> {
    let name = "storage-normal";
    let info = match storage {
        StorageType::Lanzou => add_storage!(storages_lanzou_add(guard, name, "accounts/lanzou.toml"))?,
    };
    assert_eq!(info.name.as_str(), name);
    assert_eq!(info.read_only, storage.is_share());
    assert_eq!(info.storage_type, storage);
    assert_eq!(info.available, true);

    // tokio::try_join!(
    // // TODO
    // )?;

    wlist_native::core::client::storages::storages_remove(super::c!(guard), info.id).await
}

#[test_case::test_case(StorageType::Lanzou)]
#[tokio::test]
async fn entry_point(storage: StorageType) -> anyhow::Result<()> {
    let guard = super::initialize(true).await?;

    test_empty(&guard).await?;
    test_wrong(&guard, storage).await?;
    test_normal(&guard, storage).await?;

    super::uninitialize(guard).await
}
