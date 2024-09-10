use tokio::sync::OnceCell;

mod helper;
mod server;
mod client;

macro_rules! c {
    () => { &mut None };
}
use c;

struct InitializeGuard {
    parent: crate::InitializeGuard,
    password: &'static str,
}

async fn initialize() -> anyhow::Result<InitializeGuard> {
    static INIT: OnceCell<String> = OnceCell::const_new();
    let guard = crate::initialize(true).await?;
    let password = INIT.get_or_try_init(|| async {
        let password = wlist_native::core::server::users::reset_admin_password().await?;
        wlist_native::core::client::users::users_login(c!(), "admin".to_string(), password.clone()).await?;
        Ok::<_, anyhow::Error>(password)
    }).await?.as_str();
    Ok(InitializeGuard { parent: guard, password })
}

#[inline]
async fn uninitialize(guard: InitializeGuard) -> anyhow::Result<()> {
    crate::uninitialize(guard.parent)
}
