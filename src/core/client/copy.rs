use wlist_native::common::data::files::options::Duplicate;
use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::files::{files_copy, files_move};
use wlist_native::core::client::trash::{trash_delete, trash_trash};
use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };

    let result = files_copy(c!(guard), file, file, "file".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_copy(c!(guard), file, root, "".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_copy(c!(guard), file, root, "a".repeat(32768), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_copy(c!(guard), file, root, "a".repeat(32767), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_copy(c!(guard), file, root, "file".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let list = super::list::list(guard, root, None).await?;
    let chunk = &list.files[0];

    let result = files_copy(c!(guard), chunk.get_location(root.storage), root, "file.txt".to_string(), Duplicate::Error).await;
    if let Some(info) = crate::may_error::<_, wlist_native::common::exceptions::ComplexOperationError>(result)? {
        assert_ne!(info.id, chunk.id);
        assert_eq!(info.parent_id, root.file_id);
        assert_eq!(info.is_directory, false);
        assert_eq!(info.name.as_str(), "file.txt");
        // assert_ne!(info.update_time, chunk.update_time);
        let info = trash_trash(c!(guard), info.get_location(root.storage)).await?;
        trash_delete(c!(guard), info.get_location(root.storage)).await?;
    }

    // TODO: test directory
    // TODO: test duplicate

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let _ = (guard, root); // Nothing to test.
    Ok(())
}
