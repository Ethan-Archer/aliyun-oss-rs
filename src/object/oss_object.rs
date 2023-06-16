use chrono::NaiveDateTime;

use super::{del_object::DelObject, get_url::GetUrl, GetObjectTagging, PutObject};
use crate::OssClient;
use std::borrow::Cow;

/// OSS对象，实现了上传文件、删除文件等API
#[derive(Debug, Clone)]
pub struct OssObject {
    pub(crate) client: OssClient,
    pub(crate) bucket: Cow<'static, str>,
    pub(crate) object: Cow<'static, str>,
}

impl OssObject {
    pub(crate) fn new(client: OssClient, bucket: &str, object: &str) -> Self {
        OssObject {
            client,
            bucket: Cow::Owned(bucket.to_owned()),
            object: Cow::Owned(object.to_owned()),
        }
    }
    /// 上传文件到OSS
    pub fn put_object(self) -> PutObject {
        PutObject::new(self.clone())
    }
    /// 删除文件
    pub fn del_object(self) -> DelObject {
        DelObject::new(self.clone())
    }
    /// 获取文件访问url
    pub fn get_url(self, expires: NaiveDateTime) -> GetUrl {
        GetUrl::new(self.clone(), expires)
    }
    /// 获取文件的标签信息
    pub fn get_object_tagging(self) -> GetObjectTagging {
        GetObjectTagging::new(self.clone())
    }
}
