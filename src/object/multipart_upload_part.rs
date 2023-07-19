use crate::{
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use futures_util::StreamExt;
use hyper::{header, Body, Method};
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

/// 初始化分片上传
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31993.html)
pub struct UploadPart {
    req: OssRequest,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync + 'static>>,
}
impl UploadPart {
    pub(super) fn new(oss: Oss, part_number: u32, upload_id: impl ToString) -> Self {
        let mut req = OssRequest::new(oss, Method::PUT);
        req.insert_query("partNumber", part_number);
        req.insert_query("uploadId", upload_id);
        UploadPart {
            req,
            callback: None,
        }
    }
    /// 设置文件上传进度的回调方法，此方法仅对send_file()有效
    /// ```
    /// let callback = Box::new(|uploaded_size: u64, total_size: u64| {
    ///     let percentage = if total_size == 0 {
    ///         100.0
    ///     } else {
    ///         (uploaded_size as f64) / (total_size as f64) * 100.00
    ///     };
    ///     println!("{:.2}%", percentage);
    /// });
    /// ```
    pub fn set_callback(mut self, callback: Box<dyn Fn(u64, u64) + Send + Sync + 'static>) -> Self {
        self.callback = Some(callback);
        self
    }
    /// 将磁盘中的文件上传到OSS
    ///
    /// 返回值为ETag
    pub async fn send_file(mut self, file: impl ToString) -> Result<String, Error> {
        //打开文件
        let file = File::open(file.to_string()).await?;
        //读取文件大小
        let file_size = file.metadata().await?.len();
        if file_size >= 5_368_709_120 || file_size < 102_400 {
            return Err(Error::InvalidFileSize);
        }
        //初始化文件内容读取数据流
        let buf = BufReader::with_capacity(131072, file);
        let stream = ReaderStream::with_capacity(buf, 16384);
        //初始化已上传内容大小
        let mut uploaded_size = 0;
        //初始化上传请求
        let body = Body::wrap_stream(stream.map(move |result| match result {
            Ok(chunk) => {
                if let Some(callback) = &self.callback {
                    let upload_size = chunk.len() as u64;
                    uploaded_size = uploaded_size + upload_size;
                    callback(uploaded_size, file_size);
                }
                Ok(chunk)
            }
            Err(err) => Err(err),
        }));
        self.req.set_body(body);
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let e_tag = response
                    .headers()
                    .get("ETag")
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).ok())
                    .flatten()
                    .unwrap_or_else(|| String::new());
                Ok(e_tag)
            }
            _ => Err(normal_error(response).await),
        }
    }
    /// 将内存中的数据上传到OSS
    ///
    /// 返回值为ETag
    pub async fn send_content(mut self, content: Vec<u8>) -> Result<String, Error> {
        //读取大小
        let content_size = content.len() as u64;
        if content_size >= 5_000_000_000 {
            return Err(Error::InvalidFileSize);
        }
        self.req.insert_header(header::CONTENT_LENGTH, content_size);
        //插入body
        self.req.set_body(content.into());
        //上传文件
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let e_tag = response
                    .headers()
                    .get("ETag")
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).ok())
                    .flatten()
                    .unwrap_or_else(|| String::new());
                Ok(e_tag)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
