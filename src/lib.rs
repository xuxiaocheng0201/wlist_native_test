use std::fmt::{Debug, Display};

use tokio::sync::{OnceCell, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

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
        tracing_subscriber::registry::Registry::default().with(tracing_subscriber::fmt::layer().with_filter(
            tracing_subscriber::filter::Targets::new()
                .with_target("wlist_native_test", Level::TRACE)
                .with_target("core_server_storages_database", Level::TRACE)
                .with_target("core_server_storages_lock", Level::TRACE)
                .with_target("core_server_storages_impl_lanzou", Level::TRACE)
                .with_target("", Level::INFO)
        )).init();
        wlist_native::common::initialize("run/data", "run/cache").await
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
