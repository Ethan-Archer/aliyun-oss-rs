use super::{DescribeRegions, ListBuckets};
use crate::OssBucket;
use std::borrow::Cow;

/// OSS容器入口，可以转换为OssBucket和OssObject，同时也实现了查询地域信息和查询bucket列表两个API
// #[doc(hidden)]
#[derive(Debug, Clone)]
pub struct OssClient {
    /// 阿里云AccessKey ID
    pub(crate) ak_id: Cow<'static, str>,
    /// 阿里云AccessKey Secret
    pub(crate) ak_secret: Cow<'static, str>,
    /// 地域的OSS endpoint
    pub(crate) endpoint: Cow<'static, str>,
}

impl OssClient {
    /// 初始化一个OssClient容器，以便后续使用
    ///
    /// - ak_id ： 阿里云AccessKey ID
    /// - ak_secret：阿里云AccessKey Secret
    /// - endpoint：地域的OSS endpoint
    ///
    pub fn new(ak_id: &str, ak_secret: &str, endpoint: &str) -> Self {
        OssClient {
            ak_id: ak_id.to_owned().into(),
            ak_secret: ak_secret.to_owned().into(),
            endpoint: endpoint.to_owned().into(),
        }
    }

    /// 初始化OssBucket
    pub fn bucket(&self, bucket: &str) -> OssBucket {
        OssBucket::new(self.clone(), bucket)
    }

    /// 查询所有地域的Endpoint信息
    pub fn describe_regions(&self) -> DescribeRegions {
        DescribeRegions::new(self.clone())
    }

    /// 查询已创建的所有存储空间
    pub fn list_buckets(&self) -> ListBuckets {
        ListBuckets::new(self.clone())
    }
}
