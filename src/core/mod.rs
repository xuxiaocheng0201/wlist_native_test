mod helper;
mod server;
mod client;

macro_rules! c {
    ($guard: ident) => { &mut $guard.get_client().await? };
}
use c;

struct InitializeGuard {
    parent: crate::InitializeGuard,
    password: &'static str,
}

impl InitializeGuard {
    async fn get_client<'a>(&self) -> anyhow::Result<Option<&'a mut wlist_native::core::client::WlistClient<'a>>> {
        Ok(None)
    }
}

async fn initialize(unique: bool) -> anyhow::Result<InitializeGuard> {
    static INIT: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();
    let guard = crate::initialize(unique).await?;
    let password = INIT.get_or_try_init(|| async {
        let password = wlist_native::core::server::users::reset_admin_password().await?;
        wlist_native::core::client::users::users_login(&mut None, "admin".to_string(), password.clone()).await?;
        Ok::<_, anyhow::Error>(password)
    }).await?.as_str();
    Ok(InitializeGuard { parent: guard, password })
}

#[inline]
async fn uninitialize(guard: InitializeGuard) -> anyhow::Result<()> {
    crate::uninitialize(guard.parent)
}
