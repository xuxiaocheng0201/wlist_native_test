use std::fmt::{Debug, Display};

use tokio::sync::{OnceCell, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tokio::task::spawn_blocking;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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
            tracing_subscriber::registry::Registry::default().with(tracing_subscriber::fmt::layer().with_filter(
                <tracing_subscriber::filter::Targets as std::str::FromStr>::from_str(
                    "wlist_native_test=trace,core_server_storages_database=trace,=info"
                ).unwrap()
            )).init();
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


pub fn assert_error<T: Debug, E: Debug + Display + Send + Sync + 'static>(result: anyhow::Result<T>) -> anyhow::Result<E> {
    match result {
        Ok(t) => Err(anyhow::anyhow!("expect error but returned ok: {t:?}")),
        Err(e) => e.downcast::<E>(),
    }
}

pub fn assert_error_option<T: Debug, E: Debug + Display + Send + Sync + 'static>(result: anyhow::Result<Option<T>>) -> anyhow::Result<()> {
    match result.transpose() {
        Some(result) => assert_error::<T, E>(result).map(drop),
        None => Ok(()),
    }
}

pub fn may_error<T, E: Display + Debug + Send + Sync + 'static>(result: anyhow::Result<T>) -> anyhow::Result<Option<T>> {
    match result {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.downcast_ref::<E>().is_some() => Ok(None),
        Err(e) => Err(e),
    }
}
