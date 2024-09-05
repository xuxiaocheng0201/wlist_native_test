use tokio::sync::OnceCell;
use tokio::task::spawn_blocking;

#[test]
fn versions() {
    assert_eq!(
        wlist_native::common::versions::get_common_api_version(),
        "0.2.0"
    );
    assert_eq!(
        wlist_native::common::versions::get_core_api_version(),
        "0.2.0"
    );
    assert_eq!(
        wlist_native::common::versions::get_web_api_version(),
        "0.2.0"
    );
}

pub async fn initialize() -> anyhow::Result<()> {
    static INIT: OnceCell<()> = OnceCell::const_new();
    INIT.get_or_try_init(|| async {
        spawn_blocking(|| {
            tracing_subscriber::fmt().with_max_level("debug").init();
            wlist_native::common::workspace::initialize("run/data", "run/cache")?;
            wlist_native::common::database::initialize()
        }).await.map_err(Into::into).and_then(|r| r)
    }).await.map(|()| ())
}
