use wlist_native::common::data::files::options::Duplicate;
use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::files::files_rename;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };

    let result = files_rename(c!(guard), file, "".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_rename(c!(guard), file, "a".repeat(32768), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_rename(c!(guard), file, "a".repeat(32767), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_rename(c!(guard), file, "file".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_rename(c!(guard), root, "file".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let list = super::list::list(guard, root, None).await?;

    let chunk = &list.files[0];
    let result = files_rename(c!(guard), chunk.get_location(root.storage), "file.txt".to_string(), Duplicate::Error).await;
    if let Some(info) = crate::may_error::<_, wlist_native::common::exceptions::ComplexOperationError>(result)? {
        assert_eq!(info.id, chunk.id);
        assert_eq!(info.parent_id, root.file_id);
        assert_eq!(info.is_directory, false);
        assert_eq!(info.name.as_str(), "file.txt");
        // assert_ne!(info.update_time, chunk.update_time);
        files_rename(c!(guard), info.get_location(root.storage), "chunk.txt".to_string(), Duplicate::Error).await?;
    }

    let empty = &list.files[1];
    let result = files_rename(c!(guard), empty.get_location(root.storage), "directory".to_string(), Duplicate::Error).await;
    if let Some(info) = crate::may_error::<_, wlist_native::common::exceptions::ComplexOperationError>(result)? {
        // assert_eq!(info.id, chunk.id); // May not eq for rename directory.
        assert_eq!(info.parent_id, root.file_id);
        assert_eq!(info.is_directory, true);
        assert_eq!(info.name.as_str(), "directory");
        // assert_ne!(info.update_time, empty.update_time);
        files_rename(c!(guard), info.get_location(root.storage), "empty".to_string(), Duplicate::Error).await?;
    }

    // TODO: test duplicate

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let _ = (guard, root); // Nothing to test.
    Ok(())
}
