use bytes::Bytes;
use hyper::{body::to_bytes, Body, Response};
use serde_derive::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("不支持网络路径")]
    PathNotSupported,
    #[error("文件大小超过5GB，请使用MultipartUpload接口")]
    FileTooBig,
    #[error("{0}")]
    HttpError(#[from] hyper::http::Error),
    #[error("{0}")]
    HyperError(#[from] hyper::Error),
    #[error("OSS返回了成功，但消息体结构解析失败，请尝试自行解析")]
    OssInvalidResponse(Option<Bytes>),
    #[error("{0} \n {1:#?}")]
    OssError(hyper::StatusCode, OssError),
    #[error("OSS返回了错误，HTTP状态码：{0}，错误内容请自行解析")]
    OssInvalidError(hyper::StatusCode, Bytes),
    #[error("使用了不符合要求的字符")]
    InvalidCharacter,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Error")]
pub struct OssError {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(rename = "EC")]
    pub ec: String,
}

pub async fn normal_error(response: Response<Body>) -> Error {
    let status_code = response.status();
    let response_bytes = to_bytes(response.into_body()).await;
    match response_bytes {
        Err(e) => Error::HyperError(e),
        Ok(response_bytes) => {
            let oss_error = serde_xml_rs::from_reader::<&[u8], OssError>(&*response_bytes);
            match oss_error {
                Ok(oss_error) => Error::OssError(status_code, oss_error),
                Err(_) => Error::OssInvalidError(status_code, response_bytes),
            }
        }
    }
}
