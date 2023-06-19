use super::{DescribeRegions, ListBuckets};
use crate::{OssBucket, OssObject};
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
            ak_id: Cow::Owned(ak_id.to_owned()),
            ak_secret: Cow::Owned(ak_secret.to_owned()),
            endpoint: Cow::Owned(endpoint.to_owned()),
        }
    }

    /// 初始化OssBucket
    pub fn bucket(&self, bucket: &str) -> OssBucket {
        OssBucket::new(self.clone(), bucket)
    }

    /// 初始化OssObject
    pub fn object(&self, bucket: &str, object: &str) -> OssObject {
        OssObject::new(self.clone(), bucket, object.trim().trim_matches('/'))
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
