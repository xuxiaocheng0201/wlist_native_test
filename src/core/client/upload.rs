use std::cmp::min;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::Rng;
use tokio::sync::watch::channel;
use tokio::task::JoinSet;
use tracing::warn;

use wlist_native::common::data::files::information::FileInformation;
use wlist_native::common::data::files::options::Duplicate;
use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::download::download_request;
use wlist_native::core::client::trash::{trash_delete, trash_trash};
use wlist_native::core::client::upload::{upload_cancel, upload_confirm, upload_finish, upload_mkdir, upload_request, upload_stream};
use wlist_native::core::helper::hasher::Md5Hasher;

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let md5 = Md5Hasher::new().finalize().await;

    // test_incorrect_storage
    let result = upload_mkdir(c!(guard), root, "directory".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = upload_request(c!(guard), root, "hello.txt".to_string(), 5, md5.clone(), vec![md5.clone()], Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    Ok(())
}

pub async fn upload(guard: &InitializeGuard, parent: FileLocation, name: String, data: Bytes, duplicate: Duplicate) -> anyhow::Result<FileInformation> {
    let len = data.remaining();
    let (md5, md5s) = {
        const CHUNK: usize = 4 << 20;
        let md5 = Md5Hasher::new();
        let mut md5s = Vec::new();
        let mut i = 0;
        loop {
            let l = i * CHUNK;
            let r = min((i + 1) * CHUNK, len);
            if l >= r { break; }
            let chunk = data.slice(l..r);
            let ((), md5) = tokio::join!(
                md5.update(chunk.clone()),
                async {
                    let md5 = Md5Hasher::new();
                    md5.update(chunk).await;
                    md5.finalize().await
                },
            );
            md5s.push(md5);
            i += 1;
        }
        let md5 = md5.finalize().await;
        if md5s.is_empty() { md5s.push(md5.clone()); }
        (md5, md5s)
    };
    let confirmation = upload_request(c!(guard), parent, name, len as u64, md5, md5s, duplicate).await?;
    if !confirmation.done {
        let information = upload_confirm(c!(guard), confirmation.token.clone()).await?;
        let mut set = JoinSet::new();
        for (chunk, id) in information.chunks.into_iter().zip(0..) {
            let l = chunk.start as usize;
            let r = l + chunk.size as usize;
            let data = data.slice(l..r);
            const CHUNK: usize = 1 << 10;
            let mut i = 0;
            loop {
                let l = i * CHUNK;
                let r = min((i + 1) * CHUNK, len);
                if l >= r { break; }
                let guard = unsafe { &*(guard as *const InitializeGuard) }; // Safety: not cancelled.
                let token = confirmation.token.clone();
                let mut chunk = data.slice(l..r); // slice to test upload in chunk
                set.spawn(async move {
                    let (tx, _rx) = channel(0);
                    // TODO: output process?
                    upload_stream(c!(guard), token, id, &mut chunk, tx, channel(true).1).await
                });
                i += 1;
            }
        }
        for r in set.join_all().await { r?; }
    }
    let information = upload_finish(c!(guard), confirmation.token).await?;
    assert_eq!(information.is_directory, false);
    assert_eq!(information.parent_id, parent.file_id);
    assert_eq!(information.size, Some(len as u64));
    assert!(information.create_time.is_some());
    assert!(information.update_time.is_some());
    Ok(information)
}

async fn mkdir_and_delete(guard: &InitializeGuard, root: FileLocation, name: String, duplicate: Duplicate) -> anyhow::Result<()> {
    let file = upload_mkdir(c!(guard), root, name, duplicate).await?;
    let list = super::list::list(guard, file.get_location(root.storage), None).await?;
    assert_eq!(list.total, 0);
    let information = trash_trash(c!(guard), file.get_location(root.storage)).await?;
    trash_delete(c!(guard), information.get_location(root.storage)).await
}

async fn upload_and_delete(guard: &InitializeGuard, root: FileLocation, name: String, data: Bytes, duplicate: Duplicate) -> anyhow::Result<()> {
    let file = upload(guard, root, name, data.clone(), duplicate).await?;
    let confirmation = download_request(c!(guard), file.get_location(root.storage), 0, u64::MAX).await?;
    assert_eq!(confirmation.size, data.remaining() as u64);
    let (downloaded, from, to) = super::download::download0(guard, &confirmation.token).await?;
    assert_eq!(from, 0); assert_eq!(to, data.remaining() as u64);
    assert_eq!(data, downloaded);
    let information = trash_trash(c!(guard), file.get_location(root.storage)).await?;
    trash_delete(c!(guard), information.get_location(root.storage)).await
}

fn generate_md5() -> String {
    const ALL: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
    let mut key = Vec::with_capacity(32);
    let mut rand = rand::thread_rng();
    for _ in 0..32 {
        key.push(ALL.as_bytes()[rand.gen_range(0..ALL.len())]);
    }
    unsafe { String::from_utf8_unchecked(key) }
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    // mkdir_test
    mkdir_and_delete(guard, root, "directory".to_string(), Duplicate::Error).await?;

    // upload_test_same
    upload_and_delete(guard, root, "UploadSame.txt".to_string(), Bytes::from_static(b"hello world!"), Duplicate::Error).await?; // hello.txt

    // upload_test_random
    let mut bytes = BytesMut::new();
    let mut rand = rand::thread_rng();
    for _ in 0..rand.gen_range(128..4<<10) {
        bytes.put_u8(rand.gen());
    }
    upload_and_delete(guard, root, "UploadRandom.txt".to_string(), bytes.freeze(), Duplicate::Error).await?;

    // upload_test_large
    let mut bytes = BytesMut::new();
    let mut rand = rand::thread_rng();
    for _ in 0..rand.gen_range(1<<20..5<<20) {
        bytes.put_u8(rand.gen());
    }
    upload_and_delete(guard, root, "UploadLarge.txt".to_string(), bytes.freeze(), Duplicate::Error).await?;

    // upload_test_cancel
    let md5 = generate_md5();
    let confirmation = upload_request(c!(guard), root, "hello.txt".to_string(), 5, md5.clone(), vec![md5.clone()], Duplicate::Error).await?;
    if confirmation.done {
        warn!(%md5, "upload_test_cancel: uploaded done.");
        let information = upload_finish(c!(guard), confirmation.token).await?;
        let information = trash_trash(c!(guard), information.get_location(root.storage)).await?;
        trash_delete(c!(guard), information.get_location(root.storage)).await?;
    } else {
        upload_cancel(c!(guard), confirmation.token).await?;
    }

    // TODO: test duplicate
    // TODO: test pause

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let md5 = Md5Hasher::new().finalize().await;

    // test_incorrect_parent
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };
    let result = upload_mkdir(c!(guard), file, "directory".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_request(c!(guard), file, "chunk.txt".to_string(), 5, md5.clone(), vec![md5.clone()], Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;

    // test_incorrect_name
    let result = upload_mkdir(c!(guard), root, "".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_request(c!(guard), root, "".to_string(), 5, md5.clone(), vec![md5.clone()], Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_mkdir(c!(guard), root, "a".repeat(32768), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = upload_request(c!(guard), root, "a".repeat(32768), 5, md5.clone(), vec![md5.clone()], Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;

    // test_incorrect_md5
    for invalid_md5 in ["".to_string(), "A".to_string(), "a".repeat(30) + "0A", "A".repeat(32), "-".repeat(32)] {
        let result = upload_request(c!(guard), root, "hello.txt".to_string(), 5, invalid_md5.clone(), vec![invalid_md5.clone()], Duplicate::Error).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        let result = upload_request(c!(guard), root, "hello.txt".to_string(), 5, invalid_md5.clone(), vec![md5.clone()], Duplicate::Error).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        let result = upload_request(c!(guard), root, "hello.txt".to_string(), 5, md5.clone(), vec![md5.clone(); 2], Duplicate::Error).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
        let result = upload_request(c!(guard), root, "hello.txt".to_string(), 5, md5.clone(), vec![], Duplicate::Error).await;
        crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    }
    Ok(())
}
