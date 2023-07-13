use crate::{
    common::{url_encode, Acl, StorageClass},
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use chrono::NaiveDateTime;
use hyper::Method;
use std::collections::HashMap;

/// 拷贝文件
///
/// 同Bucket内拷贝，文件大小不能超过 5GB ；不同Bucket间拷贝，文件大小不超过 1GB
///
/// 其他较多的限制，具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31979.html)
pub struct CopyObject {
    req: OssRequest,
    tags: HashMap<String, String>,
}

impl CopyObject {
    pub(super) fn new(oss: Oss, copy_source: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_header("x-oss-copy-source", copy_source);
        CopyObject {
            req,
            tags: HashMap::new(),
        }
    }
    /// 设置来源文件的版本id
    pub fn set_suorce_version_id(mut self, version_id: impl ToString) -> Self {
        self.req
            .insert_header("x-oss-copy-source-version-id", version_id);
        self
    }
    /// 设置文件的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.req.insert_header("x-oss-object-acl", acl);
        self
    }
    /// 设置文件的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.req.insert_header("x-oss-storage-class", storage_class);
        self
    }
    /// 设置需要附加的metadata
    pub fn set_meta(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.req
            .insert_header(format!("x-oss-meta-{}", key.to_string()), value);
        self
    }
    /// 如果指定的时间早于文件实际修改时间，则正常拷贝文件。
    ///
    pub fn set_if_modified_since(mut self, if_modified_since: NaiveDateTime) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-modified-since", if_modified_since);
        self
    }
    /// 如果指定的时间等于或者晚于文件实际修改时间，则正常拷贝文件。
    ///
    pub fn set_if_unmodified_since(mut self, if_unmodified_since: NaiveDateTime) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-unmodified-since", if_unmodified_since);
        self
    }
    /// 如果源文件的ETag值和您提供的ETag相等，则执行拷贝操作。
    ///
    /// 文件的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_match(mut self, if_match: impl ToString) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-match", if_match);
        self
    }
    /// 如果源文件的ETag值和您提供的ETag不相等，则执行拷贝操作。
    ///
    /// 文件的ETag值用于验证数据是否发生了更改，您可以基于ETag值验证数据完整性。
    pub fn set_if_none_match(mut self, if_none_match: impl ToString) -> Self {
        self.req
            .insert_header("x-oss-copy-source-if-none-match", if_none_match);
        self
    }
    /// 不允许覆盖同名文件
    pub fn forbid_overwrite(mut self) -> Self {
        self.req.insert_header("x-oss-forbid-overwrite", "true");
        self
    }
    /// 设置标签信息
    pub fn set_tagging(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
    /// 采用请求中指定的元数据，忽略源Object的元数据
    pub fn set_metadata_directive(mut self) -> Self {
        self.req
            .insert_header("x-oss-metadata-directive", "REPLACE");
        self
    }
    /// 直接采用请求中指定的文件标签，忽略源文件的标签
    pub fn set_tagging_directive(mut self) -> Self {
        self.req.insert_header("x-oss-tagging-directive", "Replace");
        self
    }

    /// 复制文件
    ///
    pub async fn send(mut self) -> Result<(), Error> {
        //插入标签
        let tags = self
            .tags
            .into_iter()
            .map(|(key, value)| {
                if value.is_empty() {
                    url_encode(&key.to_string())
                } else {
                    format!(
                        "{}={}",
                        url_encode(&key.to_string()),
                        url_encode(&value.to_string())
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        self.req.insert_header("x-oss-tagging", tags);
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => Ok(()),
            _ => Err(normal_error(response).await),
        }
    }
}
