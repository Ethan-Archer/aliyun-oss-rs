use crate::{
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;
use std::cmp;

// 返回的内容
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListMultipartUploadsResult {
    pub is_truncated: bool,
    pub next_key_marker: String,
    pub next_upload_id_marker: String,
    pub upload: Option<Vec<Upload>>,
    pub common_prefixes: Option<Vec<CommonPrefixes>>,
}

/// 分组列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommonPrefixes {
    /// 前缀
    pub prefix: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Upload {
    pub key: String,
    pub upload_id: String,
    pub storage_class: String,
    pub initiated: String,
}

/// 列举所有执行中的Multipart Upload事件，即已经初始化但还未完成（Complete）或者还未中止（Abort）的Multipart Upload事件
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31997.html)
pub struct ListUploads {
    req: OssRequest,
}

impl ListUploads {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("uploads", "");
        ListUploads { req }
    }
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    pub fn set_delimiter(mut self, delimiter: impl ToString) -> Self {
        self.req.insert_query("delimiter", delimiter);
        self
    }
    /// 限定返回文件的Key必须以prefix作为前缀。
    pub fn set_prefix(mut self, prefix: impl ToString) -> Self {
        self.req.insert_query("prefix", prefix.to_string());
        self
    }
    /// 设置key-marker
    pub fn set_key_marker(mut self, key_marker: impl ToString) -> Self {
        self.req.insert_query("key-marker", key_marker.to_string());
        self
    }
    /// 设置upload-id-marker
    pub fn set_upload_id_marker(mut self, upload_id_marker: impl ToString) -> Self {
        self.req
            .insert_query("upload-id-marker", upload_id_marker.to_string());
        self
    }
    /// 限定此次返回Multipart Upload事件的最大个数
    ///
    /// 当设置了delimiter时，此参数指的是文件和分组的总和
    ///
    /// 默认值：1000，取值范围：1 - 1000，设置的值如不在这个范围，则会使用默认值
    pub fn set_max_uploads(mut self, max_keys: u32) -> Self {
        let max_keys = cmp::min(1000, cmp::max(1, max_keys));
        self.req.insert_query("max-uploads", max_keys);
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<ListMultipartUploadsResult, Error> {
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let result: ListMultipartUploadsResult =
                    serde_xml_rs::from_reader(&*response_bytes)
                        .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(result)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
