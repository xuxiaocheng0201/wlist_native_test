use bytes::Bytes;
use indexmap::IndexMap;
use wlist_native::common::data::Direction;
use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::files::options::{Duplicate, FilesFilter};
use wlist_native::common::data::trashes::options::{ListTrashOptions, TrashesOrder};
use wlist_native::core::client::trash::{trash_delete, trash_delete_all, trash_get, trash_list, trash_refresh, trash_restore, trash_trash};
use wlist_native::core::client::upload::upload_mkdir;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };

    let result = trash_list(c!(guard), 0, ListTrashOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 1,
    }).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_refresh(c!(guard), 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_get(c!(guard), root, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = trash_get(c!(guard), root, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_trash(c!(guard), root).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_restore(c!(guard), root, 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_delete(c!(guard), root).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = trash_delete_all(c!(guard), 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    // test_list_empty
    let list = trash_list(c!(guard), root.storage, ListTrashOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 1,
    }).await?.unwrap_left(); // this is tested after refresh, so needn't refresh.
    assert_eq!(list.total, 0);
    assert_eq!(list.filtered, 0);
    assert_eq!(list.files.len(), 0);

    let restore = super::upload::upload(guard, root, "ToRestore.txt".to_string(), Bytes::from_static(b"to restore."), Duplicate::Error).await?;
    let delete = super::upload::upload(guard, root, "ToDelete.txt".to_string(), Bytes::from_static(b"to delete."), Duplicate::Error).await?;
    let directory = upload_mkdir(c!(guard), root, "ToDirectory".to_string(), Duplicate::Error).await?;

    // test_trash
    let restore_trash = trash_trash(c!(guard), restore.get_location(root.storage)).await?;
    assert_eq!(restore_trash.id, restore.id);
    assert_eq!(restore_trash.is_directory, false);
    assert_eq!(restore_trash.size, Some(11));
    assert!(restore_trash.trash_time.is_some());
    let delete_trash = trash_trash(c!(guard), delete.get_location(root.storage)).await?;
    assert_eq!(delete_trash.id, delete.id);
    assert_eq!(delete_trash.is_directory, false);
    assert_eq!(delete_trash.size, Some(10));
    assert!(delete_trash.trash_time.is_some());
    let directory_trash = trash_trash(c!(guard), directory.get_location(root.storage)).await?;
    assert_eq!(directory_trash.id, directory.id);
    assert_eq!(directory_trash.is_directory, true);
    assert_eq!(directory_trash.size, Some(0));
    assert!(directory_trash.trash_time.is_some());

    // test_restore
    let restore = trash_restore(c!(guard), restore_trash.get_location(root.storage), root.file_id).await?;
    assert_eq!(restore.id, restore_trash.id);
    assert_eq!(restore.is_directory, false);
    assert_eq!(restore.parent_id, root.file_id);
    assert_eq!(restore.size, Some(11));
    trash_trash(c!(guard), restore.get_location(root.storage)).await?;

    // test_delete
    trash_delete(c!(guard), delete_trash.get_location(root.storage)).await?;

    // test_list
    let list = trash_list(c!(guard), root.storage, ListTrashOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(TrashesOrder::Directory, Direction::ASCEND)]), offset: 0, limit: 3,
    }).await?.unwrap_left();
    assert_eq!(list.total, 2);
    assert_eq!(list.filtered, 2);
    assert_eq!(list.files.len(), 2);
    assert_eq!(list.files[0].name.as_str(), "ToDirectory");
    assert_eq!(list.files[1].name.as_str(), "ToRestore.txt");

    // test_delete_all
    trash_delete_all(c!(guard), root.storage).await?; // TODO: operation is too complex?
    let list = trash_list(c!(guard), root.storage, ListTrashOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 1,
    }).await?.unwrap_left();
    assert_eq!(list.total, 0);
    assert_eq!(list.filtered, 0);
    assert_eq!(list.files.len(), 0);

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let _ = (guard, root); // Nothing to test.
    Ok(())
}
