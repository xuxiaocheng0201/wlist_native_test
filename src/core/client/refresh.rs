use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::storages::information::StorageInformation;
use wlist_native::core::client::refresh::{refresh_cancel, refresh_check, refresh_confirm, refresh_progress, refresh_request};

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let result = refresh_request(c!(guard), FileLocation { storage: 0, file_id: 0, is_directory: true, }).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = refresh_request(c!(guard), FileLocation { storage: 0, file_id: 0, is_directory: false, }).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, info: &StorageInformation) -> anyhow::Result<()> {
    let root = FileLocation { storage: info.id, file_id: info.root_directory_id, is_directory: true };

    let confirmation = refresh_request(c!(guard), root).await?;
    tracing::debug!(?confirmation, "refresh test normal");
    refresh_confirm(c!(guard), confirmation.token.clone()).await?;
    loop {
        let result = refresh_progress(c!(guard), confirmation.token.clone()).await;
        let result = crate::may_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
        let Some(progress) = result else { break };
        assert!(progress.loaded_files <= progress.total_files);
        assert!(progress.loaded_directories <= progress.total_directories);
        tracing::debug!(?progress, "refresh test normal");
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    let ok = refresh_check(c!(guard), confirmation.token).await?;
    assert_eq!(ok, true);

    // TODO: test pause

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, info: &StorageInformation) -> anyhow::Result<()> {
    let root = FileLocation { storage: info.id, file_id: info.root_directory_id, is_directory: true };

    // refresh_test_cancel
    let confirmation = refresh_request(c!(guard), root).await?;
    refresh_cancel(c!(guard), confirmation.token).await?;

    Ok(())
}
