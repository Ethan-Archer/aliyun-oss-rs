use crate::{
    error::normal_error,
    request::{Oss, OssRequest},
    Error,
};
use base64::{engine::general_purpose, Engine};
use hyper::{body::to_bytes, Method};
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
    req: OssRequest,
    objects: HashSet<String>,
}
impl DelObjects {
    pub(super) fn new(oss: Oss, files: Vec<impl ToString>) -> Self {
        let mut req = OssRequest::new(oss, Method::GET);
        req.insert_query("delete", "");
        let len = files.len();
        if len == 0 {
            DelObjects {
                req,
                objects: HashSet::new(),
            }
        } else {
            let mut objects = HashSet::with_capacity(len);
            for object in files {
                objects.insert(object.to_string());
            }
            DelObjects { req, objects }
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
    pub async fn send(mut self) -> Result<Vec<DeletedObject>, Error> {
        //生成body
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
        //插入body内容
        self.req.set_body(body.into());
        //插入header内容
        self.req.insert_header("Content-Length", body_len);
        self.req.insert_header("Content-MD5", body_md5);
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
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
