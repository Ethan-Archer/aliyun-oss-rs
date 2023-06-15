use crate::{
    common::{Acl, CacheControl, ContentDisposition, OssErrorResponse, StorageClass},
    error::Error,
    sign::SignRequest,
    OssObject,
};
use futures::stream::StreamExt;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{header, Body, Client};
use std::collections::{HashMap, HashSet};
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
    mime: Option<String>,
    acl: Option<Acl>,
    storage_class: Option<StorageClass>,
    cache_control: Option<CacheControl>,
    content_disposition: Option<ContentDisposition>,
    forbid_overwrite: bool,
    x_oss_meta: HashMap<String, String>,
    x_oss_tagging: HashSet<String>,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync + 'static>>,
}

impl PutObject {
    pub(super) fn new(object: OssObject) -> Self {
        PutObject {
            object,
            mime: None,
            acl: None,
            storage_class: None,
            cache_control: None,
            content_disposition: None,
            forbid_overwrite: false,
            x_oss_meta: HashMap::new(),
            x_oss_tagging: HashSet::new(),
            callback: None,
        }
    }
    /// 设置对象的mime类型
    pub fn set_mime(mut self, mime: &str) -> Self {
        self.mime = Some(mime.to_owned());
        self
    }
    /// 设置对象的访问权限
    pub fn set_acl(mut self, acl: Acl) -> Self {
        self.acl = Some(acl);
        self
    }
    /// 设置对象的存储类型
    pub fn set_storage_class(mut self, storage_class: StorageClass) -> Self {
        self.storage_class = Some(storage_class);
        self
    }
    /// 对象被下载时网页的缓存行为
    pub fn set_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
    /// 设置对象的展示形式
    pub fn set_content_disposition(mut self, content_disposition: ContentDisposition) -> Self {
        self.content_disposition = Some(content_disposition);
        self
    }
    /// 设置是否允许覆盖同名对象
    pub fn set_forbid_overwrite(mut self, forbid_overwrite: bool) -> Self {
        self.forbid_overwrite = forbid_overwrite;
        self
    }
    /// 设置需要附加的metadata
    pub fn set_meta(mut self, key: &str, value: &str) -> Self {
        self.x_oss_meta.insert(key.to_owned(), value.to_owned());
        self
    }
    /// 设置tag信息
    pub fn set_tagging(mut self, key: &str, value: Option<&str>) -> Self {
        match value {
            Some(v) => self.x_oss_tagging.insert(format!("{}={}", key, v)),
            None => self.x_oss_tagging.insert(key.to_owned()),
        };
        self
    }
    /// 设置对象上传进度的回调方法，此方法仅对send_file()有效
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
    /// - 返回值 0 - OSS返回的Etag标识
    /// - 返回值 1 - 开启版本控制时，OSS会返回的文件版本号
    pub async fn send_file(self, file: &str) -> Result<(Option<String>, Option<String>), Error> {
        //生成文件类型
        let file_type = match self.mime {
            Some(mime) => mime,
            None => infer::get_from_path(file)?
                .map(|val| val.mime_type())
                .unwrap_or_else(|| "application/octet-stream")
                .to_owned(),
        };
        //打开文件
        let file = File::open(file).await?;
        //读取文件大小
        let file_size = file.metadata().await?.len();
        if file_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
        }
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(&self.object.object, NON_ALPHANUMERIC).to_string();
        //构造url
        let url = format!(
            "https://{}.{}/{}",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构造http请求
        let mut req = Client::new()
            .put(url)
            //插入content_type
            .header(header::CONTENT_TYPE, file_type)
            //插入content_length
            .header(header::CONTENT_LENGTH, file_size);
        //插入acl
        if let Some(acl) = self.acl {
            req = req.header("x-oss-object-acl", acl.to_string());
        }
        //插入cache-control
        if let Some(cache_control) = self.cache_control {
            req = req.header(header::CACHE_CONTROL, cache_control.to_string());
        }
        //插入content-disposition
        if let Some(content_disposition) = self.content_disposition {
            req = req.header(header::CONTENT_DISPOSITION, content_disposition.to_string());
        }
        //插入存储类型
        if let Some(storage_class) = self.storage_class {
            req = req.header("x-oss-storage-class", storage_class.to_string());
        }
        //插入不允许覆盖标志
        if self.forbid_overwrite {
            req = req.header("x-oss-forbid-overwrite", "true");
        }
        //插入x-oss-meta
        for (key, value) in self.x_oss_meta {
            req = req.header("x-oss-meta-".to_owned() + &key, value)
        }
        //插入x-oss-tagging
        if !self.x_oss_tagging.is_empty() {
            let tagging_str = self
                .x_oss_tagging
                .into_iter()
                .collect::<Vec<String>>()
                .join("&");
            req = req.header("x-oss-tagging", tagging_str);
        }
        //初始化文件内容读取数据流
        let buf = BufReader::with_capacity(131072, file);
        let stream = ReaderStream::with_capacity(buf, 16384);
        //初始化已上传内容大小
        let mut uploaded_size = 0;
        //初始化上传请求
        let req = req.body(Body::wrap_stream(stream.map(move |result| match result {
            Ok(chunk) => {
                if let Some(callback) = &self.callback {
                    let upload_size = chunk.len() as u64;
                    uploaded_size = uploaded_size + upload_size;
                    callback(uploaded_size, file_size);
                }
                Ok(chunk)
            }
            Err(err) => Err(err),
        })));
        //上传文件
        let response = req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let e_tag = headers
                    .get("ETag")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()))
                    .map(|v| v.replace("\"", ""));
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok((e_tag, version_id))
            }
            _ => {
                let body = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                return Err(Error::OssError(status_code, error_info));
            }
        }
    }
    /// 将内存中的数据上传到OSS
    ///
    /// - 返回值 0 - OSS返回的Etag标识
    /// - 返回值 1 - 开启版本控制时，OSS会返回的文件版本号
    pub async fn send_content(
        self,
        content: &[u8],
    ) -> Result<(Option<String>, Option<String>), Error> {
        //生成文件类型
        let content_type = self.mime.unwrap_or_else(|| {
            infer::get(content)
                .map(|val| val.mime_type())
                .unwrap_or_else(|| "application/octet-stream")
                .to_owned()
        });
        //读取文件大小
        let content_size = content.len() as u64;
        if content_size >= 5_000_000_000 {
            return Err(Error::FileTooBig);
        }
        //对文件名进行urlencode
        let filename_str = utf8_percent_encode(
            &self.object.object.trim().trim_matches('/'),
            NON_ALPHANUMERIC,
        )
        .to_string();
        //构造请求url
        let url = format!(
            "https://{}.{}/{}",
            self.object.bucket, self.object.client.endpoint, filename_str
        );
        //构造http请求
        let mut req = Client::new()
            .put(url)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, content_size)
            .body(content.to_owned());
        //插入acl
        if let Some(acl) = self.acl {
            req = req.header("x-oss-object-acl", acl.to_string());
        }
        //插入cache—control
        if let Some(cache_control) = self.cache_control {
            req = req.header(header::CACHE_CONTROL, cache_control.to_string());
        }
        //插入content-disposition
        if let Some(content_disposition) = self.content_disposition {
            req = req.header(header::CONTENT_DISPOSITION, content_disposition.to_string());
        }
        //插入存储类型
        if let Some(storage_class) = self.storage_class {
            req = req.header("x-oss-storage-class", storage_class.to_string());
        }
        //插入不允许覆盖标志
        if self.forbid_overwrite {
            req = req.header("x-oss-forbid-overwrite", "true");
        }
        //插入x-oss-meta
        for (key, value) in self.x_oss_meta {
            req = req.header("x-oss-meta-".to_owned() + &key, value)
        }
        //插入x-oss-tagging
        if !self.x_oss_tagging.is_empty() {
            let tagging_str = self
                .x_oss_tagging
                .into_iter()
                .collect::<Vec<String>>()
                .join("&");
            req = req.header("x-oss-tagging", tagging_str);
        }
        //签名并发送请求
        let response = req
            .sign(
                &self.object.client.ak_id,
                &self.object.client.ak_secret,
                Some(&self.object.bucket),
                Some(&self.object.object),
            )?
            .send()
            .await?;
        //拆解响应消息
        let status_code = response.status();
        match status_code {
            code if code.is_success() => {
                let headers = response.headers();
                let e_tag = headers
                    .get("ETag")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()))
                    .map(|v| v.replace(r#"\""#, ""));
                let version_id = headers
                    .get("x-oss-version-id")
                    .and_then(|header| header.to_str().ok().map(|s| s.to_owned()));
                Ok((e_tag, version_id))
            }
            _ => {
                let body = response.text().await?;
                let error_info: OssErrorResponse = serde_xml_rs::from_str(&body)?;
                return Err(Error::OssError(status_code, error_info));
            }
        }
    }
}
