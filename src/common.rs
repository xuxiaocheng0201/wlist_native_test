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

#[tokio::test]
async fn initialize() -> anyhow::Result<()> {
    let guard = crate::initialize(false).await?;
    crate::uninitialize(guard)
}
