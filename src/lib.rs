use std::fmt::{Debug, Display};

use tokio::sync::{OnceCell, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_blocking;
use tracing_subscriber::filter::LevelFilter;

#[cfg(test)]
mod common;
#[cfg(test)]
mod web;
#[cfg(test)]
mod core;


#[allow(dead_code)]
pub enum InitializeGuard {
    Read(RwLockReadGuard<'static, ()>),
    Write(RwLockWriteGuard<'static, ()>),
}

pub async fn initialize(unique: bool) -> anyhow::Result<InitializeGuard> {
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
        InitializeGuard::Read(UNIQUE_LOCK.read().await)
    } else {
        InitializeGuard::Write(UNIQUE_LOCK.write().await)
    })
}

pub fn uninitialize(guard: InitializeGuard) -> anyhow::Result<()> {
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
