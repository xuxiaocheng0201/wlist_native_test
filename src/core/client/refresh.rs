use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::files::tokens::RefreshToken;
use wlist_native::core::client::refresh::{refresh_cancel, refresh_check, refresh_confirm, refresh_is_paused, refresh_pause, refresh_progress, refresh_request, refresh_resume};

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let result = refresh_request(c!(guard), FileLocation { storage: 0, file_id: 0, is_directory: true, }).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = refresh_request(c!(guard), FileLocation { storage: 0, file_id: 0, is_directory: false, }).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    Ok(())
}

pub async fn refresh(guard: &InitializeGuard, token: RefreshToken) -> anyhow::Result<()> {
    refresh_confirm(c!(guard), token.clone()).await?;
    loop {
        let result = refresh_progress(c!(guard), token.clone()).await;
        let result = crate::may_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
        let Some(progress) = result else { break };
        assert!(progress.loaded_files <= progress.total_files);
        assert!(progress.loaded_directories <= progress.total_directories);
        tracing::debug!(?progress, "refreshing");
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }
    let ok = refresh_check(c!(guard), token).await?;
    assert_eq!(ok, true);
    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let confirmation = refresh_request(c!(guard), root).await?;
    refresh(guard, confirmation.token).await?;
    // TODO: test pause

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    // refresh_test_cancel
    let confirmation = refresh_request(c!(guard), root).await?;
    let result = refresh_pause(c!(guard), confirmation.token.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    let result = refresh_resume(c!(guard), confirmation.token.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    let result = refresh_is_paused(c!(guard), confirmation.token.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    let result = refresh_progress(c!(guard), confirmation.token.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    let result = refresh_check(c!(guard), confirmation.token.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    refresh_cancel(c!(guard), confirmation.token).await?;

    let confirmation = refresh_request(c!(guard), root).await?;
    refresh(guard, confirmation.token).await?;
    Ok(())
}
