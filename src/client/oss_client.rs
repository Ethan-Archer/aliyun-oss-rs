use super::{DescribeRegions, ListBuckets};
use crate::{request::Oss, OssBucket};

/// OSS容器入口，实现了查询OSS开服地域信息和查询存储空间列表两个API
#[derive(Debug, Clone)]
pub struct OssClient {
    pub(crate) oss: Oss,
}

impl OssClient {
    /// 初始化一个OssClient容器，以便后续使用
    ///
    /// - ak_id ： 阿里云AccessKey ID
    /// - ak_secret：阿里云AccessKey Secret
    ///
    pub fn new(ak_id: &str, ak_secret: &str) -> Self {
        OssClient {
            oss: Oss::new(ak_id, ak_secret),
        }
    }
    /// 禁用https
    pub fn disable_https(mut self) -> Self {
        self.oss.set_https(false);
        self
    }
    /// 初始化OssBucket
    pub fn bucket(&self, bucket: &str, endpoint: &str) -> OssBucket {
        OssBucket::new(self.oss.clone(), bucket, endpoint)
    }
    /// 查询所有地域的Endpoint信息
    pub fn describe_regions(&self) -> DescribeRegions {
        DescribeRegions::new(self.oss.clone())
    }
    /// 查询已创建的所有存储空间
    pub fn list_buckets(&self) -> ListBuckets {
        ListBuckets::new(self.oss.clone())
    }
}
