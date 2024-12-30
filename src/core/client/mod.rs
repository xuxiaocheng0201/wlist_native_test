use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::storages::StorageType;

mod storages;
mod refresh;
mod list;
mod get;
mod download;
mod check_name;
mod upload;
mod trash;
mod copy;
mod r#move;
mod rename;

macro_rules! add_storage {
    ($f: ident($g: ident, $n: expr, $c: literal)) => {
        wlist_native::core::client::storages::$f(
            super::c!($g), $n.into(), toml::from_str(include_str!($c))?
        ).await
    };
    ($f: ident($g: ident, $n: expr, $c: literal, None)) => {
        wlist_native::core::client::storages::$f(
            super::c!($g), $n.into(), toml::from_str(include_str!($c))?, None
        ).await
    };
    ($f: ident($g: ident, $n: expr, $c: literal, Some($t: literal))) => {
        wlist_native::core::client::storages::$f(
            super::c!($g), $n.into(), toml::from_str(include_str!($c))?, Some(toml::from_str(include_str!($t))?)
        ).await
    };
}

async fn test_none(guard: &super::InitializeGuard) -> anyhow::Result<()> {
    tokio::try_join!(
        storages::test_none(guard),
        refresh::test_none(guard),
        list::test_none(guard),
        get::test_none(guard),
        download::test_none(guard),
        check_name::test_none(guard),
        upload::test_none(guard),
        trash::test_none(guard),
        copy::test_none(guard),
        r#move::test_none(guard),
        rename::test_none(guard),
    )?;
    Ok(())
}

async fn test_wrong(guard: &super::InitializeGuard, storage: StorageType) -> anyhow::Result<()> {
    let name = "storage-wrong";
    let result = match storage {
        StorageType::Mocker => return Ok(()),
        StorageType::Lanzou => add_storage!(storages_lanzou_add(guard, name, "accounts/lanzou_wrong.toml")),
        StorageType::Baidu => add_storage!(storages_baidu_add(guard, name, "accounts/baidu_wrong.toml", None))

    };
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectStorageAccountError>(result)?;
    Ok(())
}

async fn test_normal(guard: &super::InitializeGuard, storage: StorageType) -> anyhow::Result<()> {
    let name = "storage-normal";
    async fn add_storage(name: &str, guard: &super::InitializeGuard, storage: StorageType) -> anyhow::Result<wlist_native::common::data::storages::information::StorageInformation> {
        Ok(match storage {
            StorageType::Mocker => add_storage!(storages_mocker_add(guard, name, "accounts/mocker.toml"))?, // root = 0
            StorageType::Lanzou => add_storage!(storages_lanzou_add(guard, name, "accounts/lanzou_normal.toml"))?,
            StorageType::Baidu => add_storage!(storages_baidu_add(guard, name, "accounts/baidu_normal.toml", Some("accounts/baidu_token.toml")))?,

        })
    }
    let info = add_storage(name, guard, storage).await?;
    // let info = wlist_native::core::client::storages::storages_get(super::c!(guard), 1, false).await?.basic;
    assert_eq!(info.name.as_str(), name);
    assert_eq!(info.read_only, storage.is_share());
    assert_eq!(info.storage_type, storage);
    assert_eq!(info.available, true);
    crate::assert_error::<_, wlist_native::common::exceptions::DuplicateStorageError>(add_storage(name, guard, storage).await)?;

    let info = storages::test_single(guard, &info).await?;
    let root = FileLocation { storage: info.id, file_id: info.root_directory_id, is_directory: true };
    refresh::test_normal(guard, root).await?;
    list::test_normal(guard, root).await?;
    get::test_normal(guard, root).await?;
    download::test_normal(guard, root).await?;
    check_name::test_normal(guard, root).await?;
    upload::test_normal(guard, root).await?;
    trash::test_normal(guard, root).await?;
    copy::test_normal(guard, root).await?;
    r#move::test_normal(guard, root).await?;
    rename::test_normal(guard, root).await?;

    {
        // extra test for files_get on root
        let info = wlist_native::core::client::storages::storages_get(super::c!(guard), info.id, false).await?;
        if info.size.is_some() { // fully indexed.
            assert!(
                info.indexed_size == (4 << 10) + (12 << 20) + 12 + 14 ||
                    info.indexed_size == (4 << 10) + (12 << 20) + 12 + 14 + 22,
                "{}", info.indexed_size
            );
            assert_eq!(info.size, Some(info.indexed_size));
            let root = wlist_native::core::client::files::files_get(super::c!(guard), root, true, false).await?;
            assert_eq!(info.as_file_details().basic, root.basic);
            assert_eq!(root.basic.size, info.size);
        }
    }
    match storage {
        StorageType::Mocker => add_storage!(storages_mocker_update(guard, info.id, "accounts/mocker_empty.toml"))?, // root = 3
        StorageType::Lanzou => add_storage!(storages_lanzou_update(guard, info.id, "accounts/lanzou_empty.toml"))?,
        StorageType::Baidu => add_storage!(storages_baidu_update(guard, info.id, "accounts/baidu_empty.toml"))?,

    };
    let info = wlist_native::core::client::storages::storages_get(super::c!(guard), info.id, false).await?.basic;
    let root = FileLocation { storage: info.id, file_id: info.root_directory_id, is_directory: true };

    refresh::test_empty(guard, root).await?;
    list::test_empty(guard, root).await?;
    get::test_empty(guard, root).await?;
    download::test_empty(guard, root).await?;
    check_name::test_empty(guard, root).await?;
    upload::test_empty(guard, root).await?;
    trash::test_empty(guard, root).await?;
    copy::test_empty(guard, root).await?;
    r#move::test_empty(guard, root).await?;
    rename::test_empty(guard, root).await?;

    // Ok(())
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
///      |-- empty.txt (0 size)
///      `-- 中文.zip (22 size, context=0x[50,4b,05,06,00..], md5="76cdb2bad9582d23c1f6f4d868218d6c")
/// ```
#[test_case::test_case(StorageType::Mocker)]
#[test_case::test_case(StorageType::Lanzou)]
#[test_case::test_case(StorageType::Baidu)]
#[tokio::test]
async fn entry_point(storage: StorageType) -> anyhow::Result<()> {
    let guard = super::initialize(true).await?;

    test_none(&guard).await?;
    test_wrong(&guard, storage).await?;
    test_normal(&guard, storage).await?;

    super::uninitialize(guard).await
}
