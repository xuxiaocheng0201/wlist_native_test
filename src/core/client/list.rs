use indexmap::IndexMap;

use wlist_native::common::data::files::information::FileListInformation;
use wlist_native::common::data::files::options::{Duplicate, FilesFilter, FilesOrder, ListFileOptions};
use wlist_native::common::data::files::FileLocation;
use wlist_native::common::data::Direction;
use wlist_native::core::client::files::{files_copy, files_list, files_move, files_rename};

use crate::core::{c, InitializeGuard};

pub async fn test_none(guard: &InitializeGuard) -> anyhow::Result<()> {
    let file = FileLocation { storage: 0, file_id: 0, is_directory: false, };
    let directory = FileLocation { storage: 0, file_id: 0, is_directory: true, };
    let options = ListFileOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 0,
    };

    let result = files_list(c!(guard), file, options.clone()).await;
    crate::assert_error::<_, wlist_native::common::exceptions::IncorrectArgumentError>(result)?;
    let result = files_list(c!(guard), directory, options).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = files_copy(c!(guard), file, directory, "none".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_copy(c!(guard), directory, directory, "none".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = files_move(c!(guard), file, directory, Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_move(c!(guard), directory, directory, Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    let result = files_rename(c!(guard), file, "none".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;
    let result = files_rename(c!(guard), directory, "none".to_string(), Duplicate::Error).await;
    crate::assert_error::<_, wlist_native::common::exceptions::StorageNotFoundError>(result)?;

    Ok(())
}

pub async fn list(guard: &InitializeGuard, directory: FileLocation, options: Option<ListFileOptions>) -> anyhow::Result<FileListInformation> {
    let options = options.unwrap_or(ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Name, Direction::ASCEND)]), offset: 0, limit: 10,
    });
    let confirmation = match files_list(c!(guard), directory, options.clone()).await? {
        either::Either::Left(list) => return Ok(list),
        either::Either::Right(c) => c,
    };
    super::refresh::refresh(guard, confirmation.token).await?;
    Ok(files_list(c!(guard), directory, options).await?.unwrap_left())
}

pub async fn test_normal(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    // normal_test
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 7,
    }).await?.unwrap_left(); // this is tested after refresh, so needn't refresh.
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 6);

    // normal_test_limit
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Name, Direction::ASCEND)]), offset: 0, limit: 4,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 4);
    assert_eq!(list.files[0].name.as_str(), "chunk.txt");
    assert_eq!(list.files[1].name.as_str(), "empty");
    assert_eq!(list.files[2].name.as_str(), "hello");
    assert_eq!(list.files[3].name.as_str(), "large.txt");

    // normal_test_offset
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Name, Direction::ASCEND)]), offset: 4, limit: 3,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 2);
    assert_eq!(list.files[0].name.as_str(), "recursion");
    assert_eq!(list.files[1].name.as_str(), "special");


    // filter_test_directory
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::OnlyDirectories, orders: Default::default(), offset: 0, limit: 5,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 4);

    // filter_test_file
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::OnlyFiles, orders: Default::default(), offset: 0, limit: 3,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 2);

    // filter_test_count
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::OnlyFiles, orders: Default::default(), offset: 0, limit: 0,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 0);


    // order_test_name
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Name, Direction::ASCEND)]), offset: 0, limit: 7,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 6);
    assert_eq!(list.files[0].name.as_str(), "chunk.txt");
    assert_eq!(list.files[1].name.as_str(), "empty");
    assert_eq!(list.files[2].name.as_str(), "hello");
    assert_eq!(list.files[3].name.as_str(), "large.txt");
    assert_eq!(list.files[4].name.as_str(), "recursion");
    assert_eq!(list.files[5].name.as_str(), "special");

    // order_test_suffix
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Suffix, Direction::ASCEND), (FilesOrder::Name, Direction::DESCEND)]), offset: 0, limit: 7,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 6);
    assert_eq!(list.files[0].name.as_str(), "special");
    assert_eq!(list.files[1].name.as_str(), "recursion");
    assert_eq!(list.files[2].name.as_str(), "hello");
    assert_eq!(list.files[3].name.as_str(), "empty");
    assert_eq!(list.files[4].name.as_str(), "large.txt");
    assert_eq!(list.files[5].name.as_str(), "chunk.txt");

    // order_test_directory
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: IndexMap::from([(FilesOrder::Directory, Direction::ASCEND), (FilesOrder::Name, Direction::ASCEND)]), offset: 0, limit: 7,
    }).await?.unwrap_left();
    assert_eq!(list.total_file, 2);
    assert_eq!(list.total_directory, 4);
    assert_eq!(list.files.len(), 6);
    assert_eq!(list.files[0].name.as_str(), "empty");
    assert_eq!(list.files[1].name.as_str(), "hello");
    assert_eq!(list.files[2].name.as_str(), "recursion");
    assert_eq!(list.files[3].name.as_str(), "special");
    assert_eq!(list.files[4].name.as_str(), "chunk.txt");
    assert_eq!(list.files[5].name.as_str(), "large.txt");


    let files = list.files;
    let list = super::list::list;
    // all_test
    let empty = list(guard, files[0].get_location(root.storage), None).await?;
    assert_eq!(empty.total_file, 0);
    assert_eq!(empty.total_directory, 0);
    assert_eq!(empty.files.len(), 0);
    let hello = list(guard, files[1].get_location(root.storage), None).await?;
    assert_eq!(hello.total_file, 1);
    assert_eq!(hello.total_directory, 0);
    assert_eq!(hello.files.len(), 1);
    assert_eq!(hello.files[0].name.as_str(), "hello.txt");
    let recursion = list(guard, files[2].get_location(root.storage), None).await?;
    assert_eq!(recursion.total_file, 0);
    assert_eq!(recursion.total_directory, 1);
    assert_eq!(recursion.files.len(), 1);
    assert_eq!(recursion.files[0].name.as_str(), "inner");
    let recursion = list(guard, recursion.files[0].get_location(root.storage), None).await?;
    assert_eq!(recursion.total_file, 1);
    assert_eq!(recursion.total_directory, 0);
    assert_eq!(recursion.files.len(), 1);
    assert_eq!(recursion.files[0].name.as_str(), "recursion.txt");
    list(guard, files[3].get_location(root.storage), None).await?;

    let special = list(guard, files[3].get_location(root.storage), None).await?;
    assert!(special.files.iter().any(|i| i.name.as_str() == "中文.zip"), "{:?}", special);

    Ok(())
}

pub async fn test_empty(guard: &InitializeGuard, root: FileLocation) -> anyhow::Result<()> {
    // normal_test
    let list = files_list(c!(guard), root, ListFileOptions {
        filter: FilesFilter::Both, orders: Default::default(), offset: 0, limit: 1,
    }).await?.unwrap_left(); // this is tested after refresh, so needn't refresh.
    assert_eq!(list.total_file, 0);
    assert_eq!(list.total_directory, 0);
    assert_eq!(list.files.len(), 0);
    Ok(())
}
