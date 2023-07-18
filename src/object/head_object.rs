use crate::{
    error::OssError,
    request::{Oss, OssRequest},
    Error,
};
use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use chrono::NaiveDateTime;
use hyper::Method;
use std::collections::HashMap;

/// 获取文件的元信息
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31984.html)
pub struct HeadObject {
    req: OssRequest,
}
impl HeadObject {
    pub(super) fn new(oss: Oss) -> Self {
        HeadObject {
            req: OssRequest::new(oss, Method::HEAD),
        }
    }
    /// 如果传入参数中的时间早于实际修改时间，则正常返回
    ///
    pub fn set_if_modified_since(mut self, if_modified_since: NaiveDateTime) -> Self {
        self.req.insert_header(
            "If-Modified-Since",
            if_modified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 限制必须指定的时间等于或者晚于文件实际修改时间
    ///
    pub fn set_if_unmodified_since(mut self, if_unmodified_since: NaiveDateTime) -> Self {
        self.req.insert_header(
            "If-Unmodified-Since",
            if_unmodified_since.format("%a, %e %b %Y %H:%M:%S GMT"),
        );
        self
    }
    /// 限制必须源文件的ETag值和提供的ETag相等
    ///
    /// 文件的ETag值用于验证数据是否发生了更改，可以基于ETag值验证数据完整性。
    pub fn set_if_match(mut self, if_match: impl ToString) -> Self {
        self.req.insert_header("If-Match", if_match);
        self
    }
    /// 限制必须源文件的ETag值和您提供的ETag不相等
    ///
    pub fn set_if_none_match(mut self, if_none_match: impl ToString) -> Self {
        self.req.insert_header("If-None-Match", if_none_match);
        self
    }
    /// 发送请求
    ///
    pub async fn send(self) -> Result<HashMap<String, String>, Error> {
        //构建http请求
        let mut response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers_mut();
                headers.remove("server");
                headers.remove("date");
                headers.remove("content-type");
                headers.remove("content-length");
                headers.remove("connection");
                headers.remove("x-oss-request-id");
                headers.remove("accept-ranges");
                let result = headers
                    .into_iter()
                    .map(|(key, value)| {
                        let key = key.to_string();
                        let mut value = String::from_utf8(value.as_bytes().to_vec())
                            .unwrap_or_else(|_| String::new());
                        if &key == "etag" {
                            value = value.trim_matches('"').to_owned();
                        }
                        (key, value)
                    })
                    .collect::<HashMap<String, String>>();
                Ok(result)
            }
            _ => {
                let x_oss_error = response.headers().get("x-oss-err").and_then(|header| {
                    general_purpose::STANDARD
                        .decode(header)
                        .ok()
                        .map(|v| Bytes::from(v))
                });
                match x_oss_error {
                    None => Err(Error::OssInvalidError(status_code, Bytes::new())),
                    Some(response_bytes) => {
                        let oss_error =
                            serde_xml_rs::from_reader::<&[u8], OssError>(&*response_bytes);
                        match oss_error {
                            Ok(oss_error) => Err(Error::OssError(status_code, oss_error)),
                            Err(_) => Err(Error::OssInvalidError(status_code, response_bytes)),
                        }
                    }
                }
            }
        }
    }
}
