//! 存储空间是用于存储文件（Object）的容器，所有的文件都必须隶属于某个存储空间。

#[doc(hidden)]
pub use self::oss_bucket::OssBucket;
pub use self::{
    del_bucket::DelBucket, get_bucket_info::GetBucketInfo, get_bucket_stat::GetBucketStat,
    list_objects::ListObjects, put_bucket::PutBucket,
};

mod del_bucket;
mod get_bucket_info;
mod get_bucket_stat;
mod list_objects;
mod oss_bucket;
mod put_bucket;
