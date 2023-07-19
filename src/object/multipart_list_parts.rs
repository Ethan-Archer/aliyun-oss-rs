use crate::{
    common::StorageClass,
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use hyper::{body::to_bytes, Method};
use serde_derive::Deserialize;
use std::cmp;

// 返回的内容
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListPartsResult {
    pub storage_class: StorageClass,
    pub part_number_marker: u32,
    pub next_part_number_marker: u32,
    pub is_truncated: bool,
    pub part: Option<Vec<Part>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Part {
    pub part_number: u32,
    pub last_modified: String,
    pub e_tag: String,
    pub hash_crc64ecma: u64,
    pub size: u64,
}

/// 列举指定Upload ID所属的所有已经上传成功Part
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31998.html)
pub struct ListParts {
    req: OssRequest,
}

impl ListParts {
    pub(super) fn new(oss: Oss, upload_id: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("uploadId", upload_id);
        ListParts { req }
    }
    /// 限定此次返回分片数据的最大个数
    ///
    /// 默认值：1000，取值范围：1 - 1000，设置的值如不在这个范围，则会使用默认值
    pub fn set_max_parts(mut self, max_keys: u32) -> Self {
        let max_keys = cmp::min(1000, cmp::max(1, max_keys));
        self.req.insert_query("max-uploads", max_keys);
        self
    }
    /// 指定List的起始位置
    ///
    pub fn set_part_number_marker(mut self, part_number_marker: u32) -> Self {
        self.req
            .insert_query("part-number-marker", part_number_marker);
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<ListPartsResult, Error> {
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let result: ListPartsResult = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(result)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
