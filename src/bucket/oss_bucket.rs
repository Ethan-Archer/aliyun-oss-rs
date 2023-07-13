use super::{DelBucket, DelObjects, GetBucketInfo, GetBucketStat, ListObjects, PutBucket};
use crate::{request::Oss, OssObject};

/// OSS存储空间，实现了新建存储空间、获取存储空间信息、文件列表等API
#[derive(Debug, Clone)]
pub struct OssBucket {
    pub(crate) oss: Oss,
}

impl OssBucket {
    pub(crate) fn new(mut oss: Oss, bucket: impl ToString, endpoint: impl ToString) -> Self {
        oss.set_bucket(bucket);
        oss.set_endpoint(endpoint);
        OssBucket { oss }
    }
    /// 设置自定义域名
    ///
    pub fn set_custom_domain(mut self, custom_domain: impl ToString, enable_https: bool) -> Self {
        self.oss.set_endpoint(custom_domain);
        self.oss.set_https(enable_https);
        self
    }
    /// 初始化OssObject
    pub fn object(&self, object: impl ToString) -> OssObject {
        OssObject::new(self.oss.clone(), object)
    }
    /// 创建存储空间
    pub fn put_bucket(&self) -> PutBucket {
        PutBucket::new(self.oss.clone())
    }
    /// 删除存储空间
    pub fn del_bucket(&self) -> DelBucket {
        DelBucket::new(self.oss.clone())
    }
    /// 查询存储空间中全部文件信息
    pub fn list_objects(&self) -> ListObjects {
        ListObjects::new(self.oss.clone())
    }
    /// 查询存储空间详细信息
    pub fn get_bucket_info(&self) -> GetBucketInfo {
        GetBucketInfo::new(self.oss.clone())
    }
    /// 查询存储空间的存储容量和文件数量
    pub fn get_bucket_stat(&self) -> GetBucketStat {
        GetBucketStat::new(self.oss.clone())
    }
    /// 查询存储空间的存储容量和文件数量
    pub fn del_objects(&self, files: Vec<impl ToString>) -> DelObjects {
        DelObjects::new(self.oss.clone(), files)
    }
}
