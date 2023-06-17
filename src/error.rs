use reqwest::Response;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("文件大小超过5GB，请使用MultipartUpload接口")]
    FileTooBig,
    #[error("{0}")]
    HttpError(#[from] reqwest::Error),
    #[error("OSS返回了成功，但消息体解析失败，请自行解析")]
    OssInvalidResponse(Option<Vec<u8>>),
    #[error("OSS返回了错误，HTTP状态码：{0}，错误内容：\n{1:?}")]
    OssError(reqwest::StatusCode, Option<Vec<u8>>),
    #[error("使用了不符合要求的字符")]
    InvalidCharacter,
}

pub async fn normal_error(response: Response) -> Error {
    let status_code = response.status();
    match response.bytes().await {
        Err(_) => Error::OssError(status_code, None),
        Ok(response_bytes) => Error::OssError(status_code, Some(response_bytes.into())),
    }
}
