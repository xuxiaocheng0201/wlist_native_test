use std::cmp::min;
use std::sync::Arc;

use anyhow::Error;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use dashmap::DashMap;
use tokio::sync::watch::channel;
use tokio::task::JoinSet;

use wlist_native::common::data::files::tokens::DownloadToken;
use wlist_native::common::data::files::FileLocation;
use wlist_native::core::client::download::{download_cancel, download_confirm, download_finish, download_request, download_stream};

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };
    let root = FileLocation { storage: 0, file_id: 0, is_directory: true, };

    let result = download_request(c!(guard), file, 0, u64::MAX).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = download_request(c!(guard), root, 0, u64::MAX).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = download_request(c!(guard), file, 0, 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = download_request(c!(guard), file, 1, 0).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    Ok(())
}

pub async fn download0(guard: &InitializeGuard, token: &DownloadToken) -> anyhow::Result<(Bytes, u64, u64)> {
    let information = download_confirm(c!(guard), token.clone()).await?;
    if information.chunks.is_empty() {
        return Ok((Bytes::new(), 0, 0));
    }
    let map = Arc::new(DashMap::new());
    let mut set = JoinSet::new();
    for (chunk, id) in information.chunks.iter().zip(0..) {
        let guard = unsafe { &*(guard as *const InitializeGuard) }; // Safety: not cancelled.
        let token = token.clone();
        let map = Arc::clone(&map);
        let chunk = *chunk;
        set.spawn(async move {
            let buffer = if chunk.range {
                let (tx, _rx) = channel(0);
                // TODO: output process?
                let mut buffer = BytesMut::new().limit(chunk.size as usize);
                download_stream(c!(guard), token, id, 0, &mut buffer, tx, channel(true).1).await?;
                buffer.into_inner()
            } else {
                let mut buffer = BytesMut::new();
                loop {
                    const BUF_CHUNK_SIZE: usize = 1 << 10;
                    let chunk_size = min(BUF_CHUNK_SIZE, chunk.size as usize - buffer.len());
                    if chunk_size == 0 {
                        break buffer;
                    }
                    let mut buf = BytesMut::new().limit(chunk_size);
                    let (tx, rx) = channel(0);
                    // TODO: output process?
                    download_stream(c!(guard), token.clone(), id, 0, &mut buf, tx, channel(true).1).await?;
                    let buf = buf.into_inner().freeze();
                    buffer.put_slice(&buf);
                    if *rx.borrow() < chunk_size {
                        break buffer;
                    }
                }
            };
            map.insert(id, buffer);
            Ok::<_, Error>(())
        });
    }
    for r in set.join_all().await { r?; }
    download_finish(c!(guard), token.clone()).await?;
    let map = Arc::try_unwrap(map).unwrap();
    let mut buffer = BytesMut::new();
    let l = information.chunks[0].start;
    let mut r = l;
    for (chunk, id) in information.chunks.iter().zip(0..) {
        assert_eq!(r, chunk.start);
        let buf = map.remove(&id).unwrap().1;
        r += buf.remaining() as u64;
        buffer.put_slice(&buf);
    }
    let buffer = buffer.freeze();
    assert_eq!(buffer.remaining() as u64, r - l);
    Ok((buffer, l, r))
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let list = super::list::list(guard, root, None).await?;
    let chunk = FileLocation { storage: root.storage, file_id: list.files[0].id, is_directory: false, };
    let large = FileLocation { storage: root.storage, file_id: list.files[3].id, is_directory: false, };

    tokio::try_join!(
        async {
            // download_test_chunk
            let confirmation = download_request(c!(guard), chunk, 0, u64::MAX).await?;
            let (bytes, l, r) = download0(guard, &confirmation.token).await?;
            assert_eq!(l, 0); assert_eq!(r, 4 << 10);
            assert_eq!(bytes, "@wlist small chunk 32 origin len".repeat(128).as_bytes());
            Ok::<_, Error>(())
        },
        async {
            // download_test_large
            let confirmation = download_request(c!(guard), large, 0, u64::MAX).await?;
            let (bytes, l, r) = download0(guard, &confirmation.token).await?;
            assert_eq!(l, 0); assert_eq!(r, 12 << 20);
            assert_eq!(bytes, "@wlist large file 32 origin len\n".repeat(393216).as_bytes());
            Ok::<_, Error>(())
        },
        async {
            // download_test_range
            let confirmation = download_request(c!(guard), chunk, 0, 31).await?;
            let (bytes, l, r) = download0(guard, &confirmation.token).await?;
            assert_eq!(l, 0); assert!(r > 31);
            assert_eq!(&bytes[..32], b"@wlist small chunk 32 origin len");
            Ok::<_, Error>(())
        },
        async {
            // download_test_range_no_head
            let confirmation = download_request(c!(guard), chunk, 1, 1).await?;
            let (bytes, l, r) = download0(guard, &confirmation.token).await?;
            assert!(l <= 1); assert!(r > 1);
            assert_eq!(&bytes[l as usize - 1..l as usize], b"w");
            Ok::<_, Error>(())
        },
        async {
            // download_test_cancel
            let confirmation = download_request(c!(guard), chunk, 0, 0).await?;
            let result = download_stream(c!(guard), confirmation.token.clone(), 0, 0,
                                         &mut BytesMut::new().limit(0), channel(0).0, channel(true).1).await;
            crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
            let result = download_finish(c!(guard), confirmation.token.clone()).await;
            crate::assert_error::<_, wlist_native::common::exceptions::TokenExpiredError>(result)?;
            download_cancel(c!(guard), confirmation.token).await
        },
    )?;

    let special = FileLocation { storage: root.storage, file_id: list.files[5].id, is_directory: true, };
    let special = super::list::list(guard, special, None).await?;
    let empty = special.files.iter().filter(|i| i.name.as_str() == "empty.txt").next()
        .map(|i| FileLocation { storage: root.storage, file_id: i.id, is_directory: false, });

    if let Some(empty) = empty {
        // download_test_empty
        let confirmation = download_request(c!(guard), empty, 0, u64::MAX).await?;
        let (bytes, l, r) = download0(guard, &confirmation.token).await?;
        assert_eq!(l, 0); assert_eq!(r, 0); assert_eq!(bytes, "");
    }

    // TODO: test pause

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    let result = download_request(c!(guard), FileLocation { storage: root.storage, file_id: 0, is_directory: false, }, 0, u64::MAX).await;
    crate::assert_error::<_, wlist_native::common::exceptions::FileNotFoundError>(result)?;
    Ok(())
}
