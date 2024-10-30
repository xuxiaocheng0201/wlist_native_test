use wlist_native::common::data::files::options::Duplicate;
use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::files::files_move;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };

    let result = files_move(c!(guard), file, file, Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_move(c!(guard), file, root, Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let list = super::list::list(guard, root, None).await?;
    let empty = list.files[1].get_location(root.storage);

    let chunk = &list.files[0];
    let result = files_move(c!(guard), chunk.get_location(root.storage), empty, Duplicate::Error).await;
    if let Some(info) = crate::may_error::<_, wlist_native::common::exceptions::ComplexOperationError>(result)? {
        assert_eq!(info.id, chunk.id);
        assert_eq!(info.parent_id, empty.file_id);
        assert_eq!(info.is_directory, false);
        assert_eq!(info.name, chunk.name);
        // assert_ne!(info.update_time, chunk.update_time);
        files_move(c!(guard), info.get_location(root.storage), root, Duplicate::Error).await?;
    }

    let hello = &list.files[2];
    let result = files_move(c!(guard), hello.get_location(root.storage), empty, Duplicate::Error).await;
    if let Some(info) = crate::may_error::<_, wlist_native::common::exceptions::ComplexOperationError>(result)? {
        // assert_eq!(info.id, chunk.id); // May not eq for move directory.
        assert_eq!(info.parent_id, empty.file_id);
        assert_eq!(info.is_directory, true);
        assert_eq!(info.name, hello.name);
        // assert_ne!(info.update_time, hello.update_time);
        files_move(c!(guard), info.get_location(root.storage), root, Duplicate::Error).await?;
    }

    // TODO: test duplicate

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let _ = (guard, root); // Nothing to test.
    Ok(())
}
