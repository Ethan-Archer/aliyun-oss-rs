use crate::{
    common::{
        invalid_metadata_key, url_encode, Acl, CacheControl, ContentDisposition, StorageClass,
    },
    error::{normal_error, Error},
    request::{Oss, OssRequest},
};
use futures_util::StreamExt;
use hyper::{header, Body, Method};
use std::collections::HashMap;
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

/// 追加文件
///
/// 只允许对类型为Appendable的文件进行追加，通过put方法上传的文件不允许追加
///
/// 追加文件时，文件的最终大小不允许超过 5GB
///
/// 追加文件的逻辑和限制较为复杂，建议仔细阅读 [阿里云官方文档](https://help.aliyun.com/document_detail/31978.html)
pub struct AppendObject {
    req: OssRequest,
    mime: Option<String>,
    tags: HashMap<String, String>,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync + 'static>>,
}

impl AppendObject {
    pub(super) fn new(oss: Oss) -> Self {
        let mut req = OssRequest::new(oss, Method::POST);
        req.insert_query("append", "");
        req.insert_query("position", "0");
        AppendObject {
            req,
            mime: None,
            tags: HashMap::new(),
            callback: None,
        }
    }
    /// 设置追加内容的起点
    pub fn set_position(mut self, position: u32) -> Self {
        self.req.insert_query("position", position);
        self
    }
    /// 设置文件的mime类型
    ///
    /// 如果未设置mime类型，请求发送时，会尝试从内容、本地路径、远程路径获取mime，如果依然未获取成功，则使用默认mime类型（application/octet-stream）
    pub fn set_mime(mut self, mime: impl ToString) -> Self {
        self.mime = Some(mime.to_string());
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
    /// 如果设置了上传进度的回调方法，调用者将会实时获得最新的上传进度
    ///
    pub async fn send_file(mut self, file: impl ToString) -> Result<Option<String>, Error> {
        //生成文件类型
        let file_type = match self.mime {
            Some(mime) => mime,
            None => match infer::get_from_path(&file.to_string())? {
                Some(ext) => ext.mime_type().to_owned(),
                None => mime_guess::from_path(
                    &self
                        .req
                        .oss
                        .object
                        .clone()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| String::new()),
                )
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_owned())
                .to_string(),
            },
        };
        self.req.insert_header(header::CONTENT_TYPE, file_type);
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
        //打开文件
        let file = File::open(file.to_string()).await?;
        //读取文件大小
        let file_size = file.metadata().await?.len();
        if file_size >= 5_368_709_120 {
            return Err(Error::InvalidFileSize);
        }
        self.req.insert_header(header::CONTENT_LENGTH, file_size);
        //初始化文件内容读取数据流
        let buf = BufReader::with_capacity(131072, file);
        let stream = ReaderStream::with_capacity(buf, 16384);
        //初始化已上传内容大小
        let mut uploaded_size = 0;
        //创建body对象
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
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let next_position = response
                    .headers()
                    .get("x-oss-next-append-position")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok(next_position)
            }
            _ => Err(normal_error(response).await),
        }
    }
    /// 将内存中的数据上传到OSS
    ///
    pub async fn send_content(mut self, content: Vec<u8>) -> Result<Option<String>, Error> {
        //读取文件大小
        let content_size = content.len();
        if content_size >= 5_368_709_120 {
            return Err(Error::InvalidFileSize);
        }
        self.req.insert_header(header::CONTENT_LENGTH, content_size);
        //生成文件类型
        let content_type = match self.mime {
            Some(mime) => mime,
            None => match infer::get(&content) {
                Some(ext) => ext.mime_type().to_string(),
                None => mime_guess::from_path(
                    self.req
                        .oss
                        .object
                        .clone()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| String::new().into()),
                )
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_owned())
                .to_string(),
            },
        };
        self.req.insert_header(header::CONTENT_TYPE, content_type);
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
        //插入body
        self.req.set_body(content.into());
        //构建http请求
        let response = self.req.send_to_oss()?.await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let next_position = response
                    .headers()
                    .get("x-oss-next-append-position")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok(next_position)
            }
            _ => Err(normal_error(response).await),
        }
    }
}
