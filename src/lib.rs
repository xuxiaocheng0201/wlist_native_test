use std::fmt::{Debug, Display};

use tokio::sync::{OnceCell, RwLock};
use tokio::task::spawn_blocking;
use tracing_subscriber::filter::LevelFilter;

#[cfg(test)]
mod common;
#[cfg(test)]
mod web;

mod internal {
    use tokio::sync::{RwLockReadGuard, RwLockWriteGuard};

    #[allow(dead_code)]
    pub(crate) enum InitializeGuard {
        Read(RwLockReadGuard<'static, ()>),
        Write(RwLockWriteGuard<'static, ()>),
    }
}

#[allow(private_interfaces)]
pub async fn initialize(unique: bool) -> anyhow::Result<internal::InitializeGuard> {
    static INIT: OnceCell<()> = OnceCell::const_new();
    INIT.get_or_try_init(|| async {
        spawn_blocking(|| {
            tracing_subscriber::fmt().with_max_level(LevelFilter::DEBUG).init();
            wlist_native::common::workspace::initialize("run/data", "run/cache")?;
            wlist_native::common::database::initialize()
        }).await.map_err(Into::into).and_then(|r| r)
    }).await?;
    static UNIQUE_LOCK: RwLock<()> = RwLock::const_new(());
    Ok(if unique {
        internal::InitializeGuard::Read(UNIQUE_LOCK.read().await)
    } else {
        internal::InitializeGuard::Write(UNIQUE_LOCK.write().await)
    })
}

#[allow(private_interfaces)]
pub fn uninitialize(guard: internal::InitializeGuard) -> anyhow::Result<()> {
    drop(guard);
    Ok(())
}


pub fn assert_error<T, E: Debug + Display + Send + Sync + 'static>(result: anyhow::Result<T>) -> anyhow::Result<Option<T>> {
    match result {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.downcast_ref::<E>().is_some() => Ok(None),
        Err(e) => Err(e),
    }
}
