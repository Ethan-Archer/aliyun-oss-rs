use super::{
    del_object::DelObject, AppendObject, CopyObject, GetObject, GetObjectMeta, GetObjectTagging,
    GetObjectUrl, HeadObject, PutObject,
};
use crate::request::Oss;

/// OSS文件，实现了上传文件、删除文件等API
#[derive(Debug, Clone)]
pub struct OssObject {
    oss: Oss,
}

impl OssObject {
    pub(crate) fn new(mut oss: Oss, object: impl ToString) -> Self {
        oss.set_object(object);
        OssObject { oss }
    }
    /// 上传文件到OSS
    pub fn put_object(&self) -> PutObject {
        PutObject::new(self.oss.clone())
    }
    /// 追加文件
    pub fn append_object(&self) -> AppendObject {
        AppendObject::new(self.oss.clone())
    }
    /// 删除文件
    pub fn del_object(&self) -> DelObject {
        DelObject::new(self.oss.clone())
    }
    /// 获取文件访问url
    pub fn get_object_url(&self) -> GetObjectUrl {
        GetObjectUrl::new(self.oss.clone())
    }
    /// 获取文件的标签信息
    pub fn get_object_tagging(&self) -> GetObjectTagging {
        GetObjectTagging::new(self.oss.clone())
    }
    /// 获取文件完整元信息
    pub fn head_object(&self) -> HeadObject {
        HeadObject::new(self.oss.clone())
    }
    /// 获取文件的meta信息
    pub fn get_object_meta(&self) -> GetObjectMeta {
        GetObjectMeta::new(self.oss.clone())
    }
    /// 获取文件内容
    pub fn get_object(&self) -> GetObject {
        GetObject::new(self.oss.clone())
    }
    /// 复制文件
    pub fn copy_object(&self, copy_source: &str) -> CopyObject {
        CopyObject::new(self.oss.clone(), copy_source)
    }
}
