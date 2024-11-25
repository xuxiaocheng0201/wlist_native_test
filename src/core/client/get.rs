use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::files::information::FileDetailsInformation;
use wlist_native::core::client::download::download_cancel;
use wlist_native::core::client::files::files_get;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let file = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let directory = FileLocation { storage: 0, file_id: 0, is_directory: false, };

    let result = files_get(c!(guard), file, true, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_get(c!(guard), directory, true, false).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_get(c!(guard), file, true, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_get(c!(guard), directory, true, true).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

#[allow(dead_code)]
pub async fn get(guard: &InitializeGuard, parent: Option<i64>, location: FileLocation, check: bool) -> anyhow::Result<FileDetailsInformation> {
    loop {
        let error = match files_get(c!(guard), location, false, check).await {
            Ok(information) => break Ok(information), Err(error) => error,
        };
        if error.downcast_ref::<wlist_native::common::exceptions::FileNotFoundError>().is_some() {
            if let Some(parent) = parent {
                let parent = FileLocation { storage: location.storage, file_id: parent, is_directory: true, };
                Box::pin(get(guard, None, parent, check)).await?;
                super::list::list(guard, parent, None).await?;
                continue;
            }
        }
        break Err(error);
    }
}

pub fn assert_md5(expected: Option<&str>, information: &FileDetailsInformation) {
    match expected {
        Some(expected) => if let Some(md5) = information.md5.as_ref() {
            assert_eq!(expected, md5.as_str());
        },
        None => assert_eq!(None, information.md5),
    }
}

pub async fn close_thumbnail(guard: &InitializeGuard, information: &FileDetailsInformation) -> anyhow::Result<()> {
    if let Some(thumbnail) = information.thumbnail.as_ref() {
        download_cancel(c!(guard), thumbnail.token.clone()).await?;
    }
    Ok(())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let information = files_get(c!(guard), root, false, false).await?;
    assert_eq!(information.path, Vec::<String>::new());
    close_thumbnail(guard, &information).await?;

    let list = super::list::list(guard, root, None).await?;

    let location = FileLocation { storage: root.storage, file_id: list.files[0].id, is_directory: false, };
    let information = files_get(c!(guard), location, false, false).await?;
    assert_eq!(information.basic.name.as_str(), "chunk.txt");
    assert_md5(Some("fc6cb96d6681a62e22a2bbd32e5e0519"), &information);
    assert_eq!(information.path, Vec::<String>::new());
    close_thumbnail(guard, &information).await?;

    let location = FileLocation { storage: root.storage, file_id: list.files[3].id, is_directory: false, };
    let information = files_get(c!(guard), location, false, false).await?;
    assert_eq!(information.basic.name.as_str(), "large.txt");
    assert_md5(Some("99f7ad3d42ac3318dcc92b64beecb179"), &information);
    assert_eq!(information.path, Vec::<String>::new());
    close_thumbnail(guard, &information).await?;

    let location = FileLocation { storage: root.storage, file_id: list.files[2].id, is_directory: true, };
    let information = files_get(c!(guard), location, false, false).await?;
    assert_eq!(information.basic.name.as_str(), "hello");
    assert_md5(None, &information);
    assert_eq!(information.path, Vec::<String>::new());
    close_thumbnail(guard, &information).await?;

    let hello = super::list::list(guard, location, None).await?;
    let location = FileLocation { storage: root.storage, file_id: hello.files[0].id, is_directory: false, };
    let information = files_get(c!(guard), location, false, false).await?;
    assert_eq!(information.basic.name.as_str(), "hello.txt");
    assert_md5(Some("fc3ff98e8c6a0d3087d515c0473f8677"), &information);
    assert_eq!(information.path, vec!["hello".to_string()]);
    close_thumbnail(guard, &information).await?;

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let information = files_get(c!(guard), root, false, false).await?;
    assert_eq!(information.basic.id, information.basic.parent_id);
    close_thumbnail(guard, &information).await?;
    Ok(())
}
