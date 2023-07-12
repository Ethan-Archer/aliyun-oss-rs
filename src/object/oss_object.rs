use super::{
    del_object::DelObject, AppendObject, CopyObject, GetObject, GetObjectMeta, GetObjectTagging,
    GetObjectUrl, PutObject,
};
use crate::{OssBucket, OssClient};
use chrono::NaiveDateTime;
use std::borrow::Cow;

/// OSS文件，实现了上传文件、删除文件等API
#[derive(Debug, Clone)]
pub struct OssObject {
    pub(crate) client: OssClient,
    pub(crate) bucket: Cow<'static, str>,
    pub(crate) object: Cow<'static, str>,
}

impl OssObject {
    pub(crate) fn new(bucket: OssBucket, object: &str) -> Self {
        OssObject {
            client: bucket.client,
            bucket: bucket.bucket,
            object: Cow::Owned(object.to_owned()),
        }
    }
    /// 上传文件到OSS
    pub fn put_object(&self) -> PutObject {
        PutObject::new(self.clone())
    }
    /// 追加文件
    pub fn append_object(&self) -> AppendObject {
        AppendObject::new(self.clone())
    }
    /// 删除文件
    pub fn del_object(&self) -> DelObject {
        DelObject::new(self.clone())
    }
    /// 获取文件访问url
    pub fn get_object_url(&self, expires: NaiveDateTime) -> GetObjectUrl {
        GetObjectUrl::new(self.clone(), expires)
    }
    /// 获取文件的标签信息
    pub fn get_object_tagging(&self) -> GetObjectTagging {
        GetObjectTagging::new(self.clone())
    }
    /// 获取文件的meta信息
    pub fn get_object_meta(&self) -> GetObjectMeta {
        GetObjectMeta::new(self.clone())
    }
    /// 获取文件内容
    pub fn get_object(&self) -> GetObject {
        GetObject::new(self.clone())
    }
    /// 复制文件
    pub fn copy_object(&self, copy_source: &str) -> CopyObject {
        CopyObject::new(self.clone(), copy_source)
    }
}
