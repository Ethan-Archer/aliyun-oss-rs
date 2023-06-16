use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("文件大小超过5GB，请使用MultipartUpload接口")]
    FileTooBig,
    #[error("提供的本地文件路径是一个目录")]
    FilePathError,
    #[error("{0}")]
    HttpError(#[from] reqwest::Error),
    #[error("文件上传已取消")]
    UploadCancelled,
    #[error("UrlEncode出错")]
    UrlEncodeError,
    #[error("{0}")]
    ToStrError(#[from] reqwest::header::ToStrError),
    #[error("Header获取失败")]
    HeaderError,
    #[error("上传时未设置本地文件")]
    FileNotFound,
    #[error("HeaderValue转换失败")]
    InvalidHeaderValue,
    #[error("Response.Body转换失败:{0}")]
    XmlDeserializeError(#[from] serde_xml_rs::Error),
    #[error("OSS返回了错误，HTTP状态码：{0}，错误内容：\n{1}")]
    OssError(reqwest::StatusCode, crate::common::OssErrorResponse),
    #[error("使用了不符合要求的字符")]
    InvalidCharacter,
}
