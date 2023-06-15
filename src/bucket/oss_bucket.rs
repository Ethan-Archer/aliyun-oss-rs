use super::{DelBucket, GetBucketInfo, GetBucketStat, ListObjects, PutBucket};
use crate::{OssClient, OssObject};
use std::borrow::Cow;

/// OSS存储空间，实现了新建存储空间、获取存储空间信息、文件列表等API
#[derive(Debug, Clone)]
pub struct OssBucket {
    pub(crate) client: OssClient,
    pub(crate) bucket: Cow<'static, str>,
}

impl OssBucket {
    pub(crate) fn new(client: OssClient, bucket: &str) -> Self {
        OssBucket {
            client,
            bucket: Cow::Owned(bucket.to_owned()),
        }
    }
    /// 初始化OssObject
    pub fn object(&self, object: &str) -> OssObject {
        OssObject::new(
            self.client.clone(),
            &self.bucket,
            object.trim().trim_matches('/'),
        )
    }
    /// 创建存储空间
    pub fn put_bucket(&self) -> PutBucket {
        PutBucket::new(self.clone())
    }
    /// 删除存储空间
    pub fn del_bucket(&self) -> DelBucket {
        DelBucket::new(self.clone())
    }
    /// 查询存储空间中全部对象信息
    pub fn list_objects(&self) -> ListObjects {
        ListObjects::new(self.clone())
    }
    /// 查询存储空间详细信息
    pub fn get_bucket_info(&self) -> GetBucketInfo {
        GetBucketInfo::new(self.clone())
    }
    /// 查询存储空间的存储容量和对象数量
    pub fn get_bucket_stat(&self) -> GetBucketStat {
        GetBucketStat::new(self.clone())
    }
}
