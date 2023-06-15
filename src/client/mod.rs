//! 包括AccessKey和EndPoint信息的基础服务

pub use self::describe_regions::DescribeRegions;
pub use self::list_buckets::ListBuckets;
pub use self::oss_client::OssClient;

mod describe_regions;
mod list_buckets;
mod oss_client;
