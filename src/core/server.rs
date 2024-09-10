#[tokio::test]
async fn test() -> anyhow::Result<()> {
    let guard = super::initialize(false).await?;
    let server = wlist_native::core::server::WlistServer::start("localhost:5322").await?;
    assert!(server.local_addr().ip().is_loopback()); assert_eq!(server.local_addr().port(), 5322);
    let manager = wlist_native::core::client::WlistClientManager::new(server.local_addr()).await?;
    {
        let mut client = manager.get().await?;
        let mut client = Some(&mut client);
        let client = &mut client;

        let result = wlist_native::core::client::users::users_login(client, "123".to_string(), "123".to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::PasswordMismatchedError>(result)?;

        let result = wlist_native::core::client::users::users_login(client, "admin".to_string(), "123".to_string()).await;
        crate::assert_error::<_, wlist_native::common::exceptions::PasswordMismatchedError>(result)?;

        wlist_native::core::client::users::users_login(client, "admin".to_string(), guard.password.to_string()).await?;

        wlist_native::core::client::users::users_logout(client).await?;
        wlist_native::core::client::users::users_logout(client).await?;
    }
    drop(manager);
    server.stop().await?;
    super::uninitialize(guard).await
}
