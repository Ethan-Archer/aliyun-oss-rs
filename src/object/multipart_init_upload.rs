use crate::{
    common::{
        invalid_metadata_key, url_encode, Acl, CacheControl, ContentDisposition, StorageClass,
    },
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use hyper::{body::to_bytes, header, Method};
use serde_derive::Deserialize;
use std::collections::HashMap;

// 返回内容
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InitiateMultipartUploadResult {
    upload_id: String,
}

/// 初始化分片上传
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31992.html)
pub struct InitUpload {
    req: OssRequest,
    tags: HashMap<String, String>,
}
impl InitUpload {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::POST);
        req.insert_query("uploads", "");
        InitUpload {
            req,
            tags: HashMap::new(),
        }
    }
    /// 设置文件的mime类型
    ///
    /// 如果未设置mime类型，则使用默认mime类型（application/octet-stream）
    pub fn set_mime(mut self, mime: impl ToString) -> Self {
        self.req.insert_header(header::CONTENT_TYPE, mime);
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
    /// 文件被下载时网页的缓存行为
    pub fn set_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.req.insert_header(header::CACHE_CONTROL, cache_control);
        self
    }
    /// 设置文件的展示形式
    pub fn set_content_disposition(mut self, content_disposition: ContentDisposition) -> Self {
        self.req
            .insert_header(header::CONTENT_DISPOSITION, content_disposition);
        self
    }
    /// 不允许覆盖同名文件
    pub fn forbid_overwrite(mut self) -> Self {
        self.req.insert_header("x-oss-forbid-overwrite", "true");
        self
    }
    /// 设置需要附加的metadata
    ///
    /// key只允许存在英文字母、数字、连字符，如果存在其他字符，则metadata将直接被抛弃
    pub fn set_meta(mut self, key: impl ToString, value: impl ToString) -> Self {
        let key = key.to_string();
        if !invalid_metadata_key(&key) {
            self.req
                .insert_header(format!("x-oss-meta-{}", key.to_string()), value);
        }
        self
    }
    /// 设置标签信息
    pub fn set_tagging(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.tags.insert(key.to_string(), value.to_string());
        self
    }
    /// 将磁盘中的文件上传到OSS
    ///
    pub async fn send(mut self) -> Result<String, Error> {
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
        if !tags.is_empty() {
            self.req.insert_header("x-oss-tagging", tags);
        }
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let result: InitiateMultipartUploadResult =
                    serde_xml_rs::from_reader(&*response_bytes)
                        .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(result.upload_id)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
