#![allow(dead_code)]

use wlist_native::common::data::storages::StorageType;

mod storages;
mod refresh;

macro_rules! add_storage {
    ($f: ident($g: ident, $n: expr, $c: literal)) => {
        wlist_native::core::client::storages::$f(
            super::c!($g), $n.into(), toml::from_str(include_str!($c))?
        ).await
    };
}

async fn test_none(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    tokio::try_join!(
        storages::test_none(guard),
        refresh::test_none(guard),

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
        StorageType::Lanzou => add_storage!(storages_lanzou_add(guard, name, "accounts/lanzou_normal.toml"))?,

    };
    assert_eq!(info.name.as_str(), name);
    assert_eq!(info.read_only, storage.is_share());
    assert_eq!(info.storage_type, storage);
    assert_eq!(info.available, true);

    let info = storages::test_single(guard, &info).await?;
    tokio::try_join!(
        refresh::test_normal(guard, &info),

    )?;

    match storage {
        StorageType::Lanzou => add_storage!(storages_lanzou_update(guard, info.id, "accounts/lanzou_empty.toml"))?,

    };
    let info = wlist_native::core::client::storages::storages_get(super::c!(guard), info.id, false).await?.basic;

    tokio::try_join!(
        refresh::test_empty(guard, &info),

    )?;

    let result = wlist_native::core::client::storages::storages_remove(super::c!(guard), 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    wlist_native::core::client::storages::storages_remove(super::c!(guard), info.id).await
}

/// For accounts root id:
/// ```text
/// root (rootIdStandard)
///  |-- chunk.txt (4k size, context="@wlist small chunk 32 origin len" * 128, md5="fc6cb96d6681a62e22a2bbd32e5e0519")
///  |-- large.txt (12m size, context="@wlist large file 32 origin len\n" * 393216 (no \r) , md5="99f7ad3d42ac3318dcc92b64beecb179")
///  |-- empty (rootIdEmpty)
///  |-- hello
///      `-- hello.txt (12 size, context="hello world!", md5="fc3ff98e8c6a0d3087d515c0473f8677")
///  |-- recursion
///      `-- inner
///          `-- recursion.txt (14 size, context="recursion test", md5="a1b160de5f20665f2769a6978c64c6ff")
///  `-- special
///      `-- empty.txt (0 size)
/// ```
#[test_case::test_case(StorageType::Lanzou)]
#[tokio::test]
async fn entry_point(storage: StorageType) -> anyhow::Result<()> {
    let guard = super::initialize(true).await?;

    test_empty(&guard).await?;
    test_wrong(&guard, storage).await?;
    test_normal(&guard, storage).await?;

    super::uninitialize(guard).await
}
