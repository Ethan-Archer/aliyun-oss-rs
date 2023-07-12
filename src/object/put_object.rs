use crate::{
    common::{
        url_encode, Acl, CacheControl, ContentDisposition, OssInners, PutObjectResult, StorageClass,
    },
    error::{normal_error, Error},
    send::send_to_oss,
    OssObject,
};
use futures_util::StreamExt;
use hyper::{header, Body, Method};
use std::collections::HashMap;
use tokio::{fs::File, io::BufReader};
use tokio_util::io::ReaderStream;

/// 上传文件
///
/// 添加的Object大小不能超过 5GB
///
/// 默认情况下，如果已存在同名Object且对该Object有访问权限，则新添加的Object将覆盖原有的Object
///
/// 在开启版本控制的情况下，上传文件和删除文件的逻辑都变得复杂，建议详细阅读阿里云官方文档
///
/// 具体详情查阅 [阿里云官方文档](https://help.aliyun.com/document_detail/31978.html)
pub struct PutObject {
    object: OssObject,
    headers: OssInners,
    querys: OssInners,
    mime: Option<String>,
    tags: HashMap<String, String>,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync + 'static>>,
}
impl PutObject {
    pub(super) fn new(object: OssObject) -> Self {
        PutObject {
            object,
            headers: OssInners::new(),
            querys: OssInners::new(),
            mime: None,
            tags: HashMap::new(),
            callback: None,
        }
    }
    /// 设置文件的mime类型
    pub fn set_mime(mut self, mime: impl ToString) -> Self {
        self.mime = Some(mime.to_string());
        self
    }
    /// 设置文件的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.headers.insert("x-oss-object-acl", acl);
        self
    }
    /// 设置文件的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.headers.insert("x-oss-storage-class", storage_class);
        self
    }
    /// 文件被下载时网页的缓存行为
    pub fn set_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.headers.insert(header::CACHE_CONTROL, cache_control);
        self
    }
    /// 设置文件的展示形式
    pub fn set_content_disposition(mut self, content_disposition: ContentDisposition) -> Self {
        self.headers
            .insert(header::CONTENT_DISPOSITION, content_disposition);
        self
    }
    /// 不允许覆盖同名文件
    pub fn forbid_overwrite(mut self) -> Self {
        self.headers.insert("x-oss-forbid-overwrite", "true");
        self
    }
    /// 设置需要附加的metadata
    pub fn set_meta(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.headers
            .insert(format!("x-oss-meta-{}", key.to_string()), value);
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
    pub async fn send_file(mut self, file: &str) -> Result<PutObjectResult, Error> {
        //生成文件类型
        let file_type = match self.mime {
            Some(mime) => mime,
            None => match infer::get_from_path(file)? {
                Some(ext) => ext.mime_type().to_owned(),
                None => mime_guess::from_path(&*self.object.object)
                    .first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_owned())
                    .to_string(),
            },
        };
        self.headers.insert(header::CONTENT_TYPE, file_type);
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
        self.headers.insert("x-oss-tagging", tags);
        //打开文件
        let file = File::open(file).await?;
        //读取文件大小
        let file_size = file.metadata().await?.len();
        if file_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
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
        //上传文件
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::PUT,
            Some(&self.querys),
            Some(&self.headers),
            body,
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let e_tag = headers.get("ETag").and_then(|header| {
                    header.to_str().ok().map(|s| s.trim_matches('"').to_owned())
                });
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok(PutObjectResult { e_tag, version_id })
            }
            _ => Err(normal_error(response).await),
        }
    }
    /// 将内存中的数据上传到OSS
    ///
    pub async fn send_content(mut self, content: Vec<u8>) -> Result<PutObjectResult, Error> {
        //生成文件类型
        let content_type = match self.mime {
            Some(mime) => mime,
            None => match infer::get(&content) {
                Some(ext) => ext.mime_type().to_string(),
                None => mime_guess::from_path(&*self.object.object)
                    .first()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_owned())
                    .to_string(),
            },
        };
        self.headers.insert(header::CONTENT_TYPE, content_type);
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
        self.headers.insert("x-oss-tagging", tags);
        //读取大小
        let content_size = content.len() as u64;
        if content_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
        }
        //上传文件
        let response = send_to_oss(
            &self.object.client,
            Some(&self.object.bucket),
            Some(&self.object.object),
            Method::PUT,
            Some(&self.querys),
            Some(&self.headers),
            Body::from(content),
        )?
        .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let e_tag = headers.get("ETag").and_then(|header| {
                    header.to_str().ok().map(|s| s.trim_matches('"').to_owned())
                });
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok(PutObjectResult { e_tag, version_id })
            }
            _ => Err(normal_error(response).await),
        }
    }
}
