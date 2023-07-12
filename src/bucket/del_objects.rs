use crate::{common::OssInners, error::normal_error, send::send_to_oss, Error, OssBucket};
use base64::{engine::general_purpose, Engine};
use hyper::{body::to_bytes, Body, Method};
use md5::{Digest, Md5};
use serde_derive::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(rename = "DeleteResult")]
pub(crate) struct DeleteObjectsResult {
    #[serde(rename = "Deleted")]
    pub deleted: Vec<DeletedObject>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Deleted")]
pub struct DeletedObject {
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "DeleteMarker")]
    pub delete_marker: Option<String>,
}

/// 批量删除文件
///
/// 删除文件时，不会检查文件是否存在，只要请求合法，都会返回成功
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31983.html)
pub struct DelObjects {
    bucket: OssBucket,
    querys: OssInners,
    objects: HashSet<String>,
}
impl DelObjects {
    pub(super) fn new(bucket: OssBucket, files: Vec<impl ToString>) -> Self {
        let querys = OssInners::from("delete", "");
        let len = files.len();
        let objects = if len == 0 {
            HashSet::new()
        } else {
            let mut objects = HashSet::with_capacity(len);
            for object in files {
                objects.insert(object.to_string());
            }
            objects
        };
        DelObjects {
            bucket,
            querys,
            objects,
        }
    }
    /// 添加要删除的文件
    ///
    pub fn add_files(mut self, files: Vec<impl ToString>) -> Self {
        let len = files.len();
        if len == 0 {
            self
        } else {
            self.objects.reserve(len);
            for object in files {
                self.objects.insert(object.to_string());
            }
            self
        }
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<Vec<DeletedObject>, Error> {
        // 生成body内容
        let body = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Delete><Quiet>false</Quiet>{}</Delete>",
            self.objects
                .iter()
                .map(|v| format!("<Object><Key>{}</Key></Object>", v))
                .collect::<Vec<_>>()
                .join("")
        );
        //计算body长度
        let body_len = body.len();
        //计算body md5值
        let mut hasher = Md5::new();
        hasher.update(&body);
        let result = hasher.finalize();
        let body_md5 = general_purpose::STANDARD.encode(&result);
        //生成header内容
        let mut headers = OssInners::from("Content-Length", body_len);
        headers.insert("Content-MD5", body_md5);
        //构建http请求
        let response = send_to_oss(
            &self.bucket.client,
            Some(&self.bucket.bucket),
            None,
            Method::POST,
            Some(&self.querys),
            Some(&headers),
            Body::from(body),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let response_bytes = to_bytes(response.into_body())
                    .await
                    .map_err(|_| Error::OssInvalidResponse(None))?;
                let result: DeleteObjectsResult = serde_xml_rs::from_reader(&*response_bytes)
                    .map_err(|_| Error::OssInvalidResponse(Some(response_bytes)))?;
                Ok(result.deleted)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
