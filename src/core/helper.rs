#[tokio::test]
async fn read_buffer() {
    let data = vec![1, 2, 3];
    let buffer = unsafe { wlist_native::core::helper::buffer::new_read_buffer(data.as_ptr(), 3) };
    let mut iter = buffer.into_iter();
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
}

#[tokio::test]
async fn write_buffer() {
    use bytes::BufMut;
    let mut data = vec![0; 3];
    let mut buffer = unsafe { wlist_native::core::helper::buffer::new_write_buffer(data.as_mut_ptr(), 3) };
    buffer.put_u8(1);
    buffer.put_u8(2);
    buffer.put_u8(3);
    drop(buffer);
    assert_eq!(&data, &[1, 2, 3]);
}

#[tokio::test]
#[should_panic]
async fn write_buffer_panic() {
    use bytes::BufMut;
    let mut data = vec![0; 3];
    let mut buffer = unsafe { wlist_native::core::helper::buffer::new_write_buffer(data.as_mut_ptr(), 0) };
    buffer.put_u8(1);
}

#[tokio::test]
async fn md5() {
    let md5 = wlist_native::core::helper::hasher::Md5Hasher::new();
    md5.update(bytes::Bytes::from_static("hello world".as_bytes())).await;
    assert_eq!(&md5.finalize().await, "5eb63bbbe01eeed093cb22bb8f5acdc3");
}

#[tokio::test]
async fn sha256() {
    let sha256 = wlist_native::core::helper::hasher::Sha256Hasher::new();
    sha256.update(bytes::Bytes::from_static("hello world".as_bytes())).await;
    assert_eq!(&sha256.finalize().await, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
}
