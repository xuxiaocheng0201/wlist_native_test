#[tokio::test]
async fn check_version() -> anyhow::Result<()> {
    let guard = crate::initialize(false).await?;

    let state = wlist_native::web::version::check_version().await?;
    assert_eq!(state, wlist_native::web::version::VersionState::Latest);

    crate::uninitialize(guard)
}

static INVALID_PASSWORDS: &[&str] = &[
    // empty password
    "",
    // too short password
    "12345",
    // too long password
    "123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
];
static VALID_PASSWORDS: &[&str] = &[
    // min len password
    "pwd123",
    // max len password
    "12345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678",
    // wrong password
    "abc123456acb",
    // sql inject
    "abc' OR '1'='1",
];

async fn test_invalid_login(user_id: &str) -> anyhow::Result<()> {
    for password in INVALID_PASSWORDS.iter().chain(VALID_PASSWORDS) {
        let result = wlist_native::web::account::login::login(user_id, *password).await;
        crate::assert_error::<_, wlist_native::common::exceptions::PasswordMismatchedError>(result)?;
    }
    Ok(())
}

static INVALID_USER_IDS: &[&str] = &[
    // empty user_id
    "",
    // admin
    "admin",
];

#[tokio::test]
async fn login_invalid() -> anyhow::Result<()> {
    let guard = crate::initialize(false).await?;

    for user_id in INVALID_USER_IDS {
        test_invalid_login(user_id).await?;
    }

    crate::uninitialize(guard)
}

#[tokio::test]
async fn register() -> anyhow::Result<()> {
    let guard = crate::initialize(true).await?;

    let device_id = format!("test-{:?}", std::time::Instant::now());
    tracing::trace!("Generated device_id: {device_id}");
    let user_id = wlist_native::web::register::as_guest::register_as_guest(&device_id, "123456").await?
        .ok_or(anyhow::anyhow!("Failed to register as guest"))?;

    // test_register_as_guest_invalid
    for password in INVALID_PASSWORDS {
        let result = wlist_native::web::register::as_guest::register_as_guest(&device_id, *password).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }
    // test_register_as_guest_duplicate
    for password in VALID_PASSWORDS {
        let result = wlist_native::web::register::as_guest::register_as_guest(&device_id, *password).await?;
        assert_eq!(result, None);
    }

    test_invalid_login(&user_id).await?;
    wlist_native::web::account::login::login(&user_id, "123456").await?;

    test_with_login().await?;

    wlist_native::web::account::logout::logout().await?;
    wlist_native::web::account::login::login(&user_id, "123456").await?;

    // test_unregister_invalid
    for password in INVALID_PASSWORDS.iter().chain(VALID_PASSWORDS) {
        let result = wlist_native::web::register::unregister::unregister(*password).await;
        crate::assert_error::<_, wlist_native::common::exceptions::PasswordMismatchedError>(result)?;
    }
    wlist_native::web::register::unregister::unregister("123456").await?;

    crate::uninitialize(guard)
}

static INVALID_NICKNAME: &[&str] = &[
    // empty nickname
    "",
    // too long nickname
    "123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789",
];
static VALID_NICKNAME: &[&str] = &[
    // min len nickname
    "a",
    // max len nickname
    "12345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678",
    // normal nickname
    "test123",
];

async fn test_with_login() -> anyhow::Result<()> {
    let nickname = wlist_native::web::user::nickname::get_nickname().await?;

    // test_set_nickname_invalid
    for nickname in INVALID_NICKNAME {
        let result = wlist_native::web::user::nickname::set_nickname(*nickname).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }
    // test_set_nickname_valid
    for nickname in VALID_NICKNAME {
        wlist_native::web::user::nickname::set_nickname(*nickname).await?;
        assert_eq!(wlist_native::web::user::nickname::get_nickname().await?, *nickname);
    }

    wlist_native::web::user::nickname::set_nickname("test").await?;
    assert_eq!(wlist_native::web::user::nickname::get_nickname().await?, "test");

    // test_set_nickname_duplicate
    wlist_native::web::user::nickname::set_nickname("test").await?;
    assert_eq!(wlist_native::web::user::nickname::get_nickname().await?, "test");

    wlist_native::web::user::nickname::set_nickname(nickname).await?;

    // test_reset_password_invalid
    for old in INVALID_PASSWORDS.iter().chain(VALID_PASSWORDS) {
        for new in INVALID_PASSWORDS {
            let result = wlist_native::web::user::password::reset_password(*old, *new).await;
            crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        }
        for new in VALID_PASSWORDS.iter().chain(std::iter::once(&"123456")) {
            let result = wlist_native::web::user::password::reset_password(*old, *new).await;
            crate::assert_error::<_, wlist_native::common::exceptions::PasswordMismatchedError>(result)?;
        }
    }
    for password in INVALID_PASSWORDS.iter() {
        let result = wlist_native::web::user::password::reset_password("123456", *password).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }

    // test_reset_password_valid
    let mut old = "123456";
    for password in VALID_PASSWORDS {
        wlist_native::web::user::password::reset_password(old, *password).await?;
        old = password;
    }
    wlist_native::web::user::password::reset_password(old, "123456").await?;
    Ok(())
}

#[tokio::test]
async fn test_without_login() -> anyhow::Result<()> {
    let guard = crate::initialize(false).await?;

    let result = wlist_native::web::account::logout::logout().await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;

    let result = wlist_native::web::user::nickname::get_nickname().await;
    crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;

    for nickname in INVALID_NICKNAME.iter().chain(VALID_NICKNAME) {
        let result = wlist_native::web::user::nickname::set_nickname(*nickname).await;
        crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
    }

    for old in INVALID_PASSWORDS.iter().chain(VALID_PASSWORDS).chain(std::iter::once(&"123456")) {
        for new in INVALID_PASSWORDS.iter().chain(VALID_PASSWORDS).chain(std::iter::once(&"123456")) {
            let result = wlist_native::web::user::password::reset_password(*old, *new).await;
            crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
        }
    }

    crate::uninitialize(guard)
}
