use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::upload::upload_check_name;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };

    // test_incorrect_parent
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };
    let result = upload_check_name(c!(guard), "chunk.txt".to_string(), file, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_check_name(c!(guard), "".to_string(), file, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_check_name(c!(guard), "a".repeat(32768), file, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;

    // test_incorrect_name
    let result = upload_check_name(c!(guard), "".to_string(), root, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_check_name(c!(guard), "a".repeat(32768), root, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;

    // test_incorrect_storage
    let result = upload_check_name(c!(guard), "chunk.txt".to_string(), root, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = upload_check_name(c!(guard), "chunk.txt".to_string(), root, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

pub async fn check_name(guard: &InitializeGuard, name: String, parent: FileLocation, is_directory: bool) -> anyhow::Result<Option<()>> {
    let result = upload_check_name(c!(guard), name.to_string(), parent, is_directory).await;
    if let Err(e) = &result {
        macro_rules! downcast_ref {
            ($e: ident, $t: ty) => {
                if let Some(e) = $e.downcast_ref::<$t>() {
                    tracing::warn!(?e, "Checking name."); return Ok(None);
                }
            };
        }
        downcast_ref!(e, wlist_native::common::exceptions::NameTooLongError);
        downcast_ref!(e, wlist_native::common::exceptions::InvalidFilenameError);
        downcast_ref!(e, wlist_native::common::exceptions::IllegalSuffixError);
    }
    result.map(Some)
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    upload_check_name(c!(guard), "hello.txt".to_string(), root, false).await?;

    // test_duplicate
    let result = check_name(guard, "chunk.txt".to_string(), root, false).await;
    crate::assert_error_option::<_, wlist_native::common::exceptions::DuplicateFileError>(result)?;
    let result = check_name(guard, "chunk.txt".to_string(), root, true).await;
    crate::assert_error_option::<_, wlist_native::common::exceptions::DuplicateFileError>(result)?;
    let result = check_name(guard, "hello".to_string(), root, false).await;
    crate::assert_error_option::<_, wlist_native::common::exceptions::DuplicateFileError>(result)?;
    let result = check_name(guard, "hello".to_string(), root, true).await;
    crate::assert_error_option::<_, wlist_native::common::exceptions::DuplicateFileError>(result)?;
    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    check_name(guard, "a".to_string(), root, false).await?;
    check_name(guard, "a".repeat(32767), root, false).await?;
    check_name(guard, "a".to_string(), root, true).await?;
    check_name(guard, "a".repeat(32767), root, true).await?;
    check_name(guard, "中文测试".to_string(), root, true).await?;
    check_name(guard, "123//".to_string(), root, true).await?;

    upload_check_name(c!(guard), "1.txt".to_string(), root, false).await?;
    upload_check_name(c!(guard), "a.zip".to_string(), root, false).await?;
    Ok(())
}
